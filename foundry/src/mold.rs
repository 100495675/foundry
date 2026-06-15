use crate::runtime::access_matrix;
use rkyv::validation::validators::DefaultValidator;
use rkyv::Deserialize;
use std::sync::atomic::{AtomicPtr, Ordering};

pub trait Pour {
    type Output: rkyv::Archive;
    fn pour(&self) -> Self::Output;
}

impl<R, G> Pour for G
where
    G: Fn() -> R + Copy,
    R: rkyv::Archive,
{
    type Output = R;

    #[inline(always)]
    fn pour(&self) -> R {
        (self)()
    }
}

pub struct Mold<F, Target> {
    fallback_function: F,
    injected_bytes: Option<&'static [u8]>,
    storage: AtomicPtr<Target>,
}

impl<F: Clone, Target> Clone for Mold<F, Target> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            fallback_function: self.fallback_function.clone(),
            injected_bytes: self.injected_bytes,
            // Clonamos el valor del puntero atómico actual de forma segura
            storage: AtomicPtr::new(self.storage.load(Ordering::Relaxed)),
        }
    }
}

// ELIMINADO: impl Copy for Mold. Se acabó el error E0204 para siempre.

impl<F, T> Mold<F, T>
where
    F: Pour<Output = T> + Copy,
    T: rkyv::Archive,
    rkyv::Archived<T>: for<'a> rkyv::CheckBytes<DefaultValidator<'a>>
        + rkyv::Deserialize<T, rkyv::Infallible>
        + 'static,
{
    #[inline(always)]
    #[doc(hidden)]
    pub const fn new_internal(fallback_function: F, injected_bytes: Option<&'static [u8]>) -> Self {
        Self {
            fallback_function,
            injected_bytes,
            storage: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    #[inline(always)]
    pub const fn is_forged(&self) -> bool {
        self.injected_bytes.is_some()
    }

    #[inline(always)]
    pub fn cast(&self) -> &'static T {
        let current_ptr = self.storage.load(Ordering::Acquire);
        if !current_ptr.is_null() {
            return unsafe { &*current_ptr };
        }

        let value = if let Some(matrix_bytes) = self.injected_bytes {
            let archived = access_matrix::<T>(matrix_bytes);
            archived.deserialize(&mut rkyv::Infallible).unwrap()
        } else {
            self.fallback_function.pour()
        };

        let leaked_ref = Box::leak(Box::new(value));
        let allocated_ptr = leaked_ref as *mut T;

        match self.storage.compare_exchange(
            std::ptr::null_mut(),
            allocated_ptr,
            Ordering::Release,
            Ordering::Acquire,
        ) {
            Ok(_) => leaked_ref,
            Err(existing_ptr) => unsafe {
                let _reclaimed_box = Box::from_raw(allocated_ptr);
                &*existing_ptr
            },
        }
    }
}

impl<T> PartialEq for Mold<fn() -> T, T>
where
    T: rkyv::Archive,
    rkyv::Archived<T>: for<'a> rkyv::CheckBytes<DefaultValidator<'a>>
        + rkyv::Deserialize<T, rkyv::Infallible>
        + 'static,
{
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        std::ptr::fn_addr_eq(self.fallback_function, other.fallback_function)
    }
}

impl<T> Eq for Mold<fn() -> T, T>
where
    T: rkyv::Archive,
    rkyv::Archived<T>: for<'a> rkyv::CheckBytes<DefaultValidator<'a>>
        + rkyv::Deserialize<T, rkyv::Infallible>
        + 'static,
{
}
