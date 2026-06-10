pub mod mold;
pub mod mold_runtime;
pub mod shape;

pub mod internal {
    pub use foundry_internal::Pattern;
    // Hacemos que bincode sea visible públicamente para el código inyectado por la macro
    pub use bincode;
}

pub use foundry_macros::pattern;
pub use foundry_macros::Shape;

pub mod prelude {
    pub use crate::mold::Mold;
    pub use crate::pattern;
    pub use crate::shape::Shape as TraitShape;
    pub use crate::Shape;
}
