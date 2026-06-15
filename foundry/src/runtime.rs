use rkyv::check_archived_root;
use rkyv::validation::validators::DefaultValidator;

#[inline(always)]
#[doc(hidden)]
pub fn access_matrix<T>(bytes: &'static [u8]) -> &'static rkyv::Archived<T>
where
    T: rkyv::Archive,
    rkyv::Archived<T>: for<'a> rkyv::CheckBytes<DefaultValidator<'a>>,
{
    let payload = &bytes[40..];

    #[cfg(debug_assertions)]
    {
        check_archived_root::<T>(payload)
            .expect("foundry: Corrupted structural binary payload detected")
    }

    #[cfg(not(debug_assertions))]
    {
        unsafe { rkyv::archived_root::<T>(payload) }
    }
}

#[doc(hidden)]
pub trait FoundryFallbackRouter {
    #[inline(always)]
    fn __foundry_get_matrix(&self, _target_ptr: usize) -> Option<&'static [u8]> {
        None
    }
}

#[doc(hidden)]
pub struct DefaultRouter;

impl FoundryFallbackRouter for DefaultRouter {
    #[inline(always)]
    fn __foundry_get_matrix(&self, _target_ptr: usize) -> Option<&'static [u8]> {
        None
    }
}
