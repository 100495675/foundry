pub mod mold;
pub mod runtime;

#[doc(hidden)]
pub mod internal {
    pub use crate::runtime::FoundryFallbackRouter;
    pub use rkyv;
}

pub use foundry_macros::pattern;

pub mod prelude {
    pub use crate::mold;
    pub use crate::mold::Mold;
    pub use crate::pattern;
}

#[macro_export]
macro_rules! mold {
    ($function:expr) => {{
        use $crate::runtime::FoundryFallbackRouter as _;
        let function_ptr: fn() -> _ = $function;
        let bytes = (&&function_ptr).__foundry_get_matrix(function_ptr as usize);
        $crate::mold::Mold::new_internal(function_ptr, bytes)
    }};
}
