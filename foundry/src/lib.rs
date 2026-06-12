pub mod mold;
pub mod runtime;

// Re-export internal components required for macro syntax expansion
#[doc(hidden)]
pub mod internal {
    pub use crate::runtime::{bincode_options, FoundryFallbackRouter};
    pub use bincode;
}

pub use foundry_macros::pattern;

pub mod prelude {
    pub use crate::mold;
    pub use crate::mold::Mold;
    pub use crate::pattern; // Declarative macro export
}

/// Master declarative macro of the forge based on the function type.
#[macro_export]
macro_rules! mold {
    ($function:expr) => {
        {
            // Import the global fallback router from internal plumbing
            use $crate::runtime::FoundryFallbackRouter as _;

            // Coerce the input into a standard flat function pointer
            let function_ptr: fn() -> _ = $function;

            // Invoke injecting the hardware memory address as a static discriminator.
            // Double ampersand (&&function_ptr) orchestrates the Method Resolution Order (MRO):
            // 1. If decorated with #[pattern], resolves local extension trait on `&fn() -> T` (Max Priority).
            // 2. If it is a common function, scales down to global FoundryFallbackRouter on `&&fn() -> T`.
            let bytes = (&&function_ptr).__foundry_get_matrix(function_ptr as usize);

            $crate::mold::Mold::new_internal(function_ptr, bytes)
        }
    };
}
