pub mod mold;
pub mod vista;

pub use foundry_core as core;
pub use foundry_macros::pattern;

pub mod prelude {
    pub use crate::pattern;
    pub use crate::vista::Pipeline;
    pub use rkyv;
}
