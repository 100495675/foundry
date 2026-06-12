use crate::runtime::cast_from_matrix;

/// Internal infrastructure trait to statically resolve the evaluation
/// of any pure executable logic (Function Items or function pointers) during fallback.
pub trait Pour {
    type Output;
    fn pour(&self) -> Self::Output;
}

// Universal Blanket Implementation for anything that satisfies `Fn() -> R + Copy`.
// Seamlessly covers standard function pointers and clean Function Items without overlapping.
impl<R, G> Pour for G
where
    G: Fn() -> R + Copy,
{
    type Output = R;

    #[inline(always)]
    fn pour(&self) -> R {
        (self)()
    }
}

/// Universal zero-cost runtime wrapper.
///
/// Occupies exactly 24 bytes on the Stack and is as lightweight as a primitive type.
/// The `F` type mathematically maps the unique identity of the function in the foundry.
pub struct Mold<F> {
    fallback_function: F,
    injected_bytes: Option<&'static [u8]>,
}

impl<F: Clone> Clone for Mold<F> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            fallback_function: self.fallback_function.clone(),
            injected_bytes: self.injected_bytes,
        }
    }
}

impl<F: Copy> Copy for Mold<F> {}

impl<F> Mold<F>
where
    F: Pour + Copy,
    <F as Pour>::Output: serde::de::DeserializeOwned,
{
    /// Internal constructor for zero-allocation on the Heap.
    #[inline(always)]
    #[doc(hidden)]
    pub const fn new_internal(fallback_function: F, injected_bytes: Option<&'static [u8]>) -> Self {
        Self {
            fallback_function,
            injected_bytes,
        }
    }

    /// Deserializes from the static injected bytes (Forged Phase) or pours the original function (Dynamic Phase).
    #[inline(always)]
    pub fn cast(&self) -> <F as Pour>::Output {
        if let Some(matrix_bytes) = self.injected_bytes {
            match cast_from_matrix::<<F as Pour>::Output>(matrix_bytes) {
                Ok(object) => return object,
                Err(e) => panic!("foundry: cast_from_matrix failed: {:?}", e),
            }
        }

        // Pours the original function: zero Box, zero vtable, 100% inlinable.
        self.fallback_function.pour()
    }

    /// Strict binary guarantee: returns `true` only if the physical binary
    /// carries the pre-calculated bytes injected from the `.matrix` file.
    #[inline(always)]
    pub const fn is_forged(&self) -> bool {
        self.injected_bytes.is_some()
    }
}

// --- SAFE ALGEBRAIC IDENTITY WITHOUT UB ---
// We restrict strict equality to named function pointers (`Mold<fn() -> T>`).
impl<T> PartialEq for Mold<fn() -> T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        std::ptr::fn_addr_eq(self.fallback_function, other.fallback_function)
    }
}

impl<T> Eq for Mold<fn() -> T> {}
