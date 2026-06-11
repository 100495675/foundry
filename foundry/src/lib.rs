pub mod mold;
pub mod mold_runtime;

pub mod internal {
    pub use bincode;
    pub use crate::mold_runtime::bincode_options;
}

pub use foundry_macros::pattern;
pub use foundry_core::Pattern;

pub mod prelude {
    pub use crate::mold::Mold;
    pub use crate::pattern;
}