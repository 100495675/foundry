pub mod mold;
pub mod mold_runtime;

pub mod internal {
    pub use crate::mold_runtime::bincode_options;
    pub use bincode;

    /// Trait fallback global por defecto para la especialización por Autoref.
    pub trait FoundryFallbackRouter {
        #[inline(always)]
        fn __foundry_obtener_matriz(&self) -> Option<&'static [u8]> {
            None
        }
    }

    /// Implementación fallback de última opción sobre la referencia de un puntero.
    impl<T> FoundryFallbackRouter for &fn() -> T {}
}

pub use foundry_macros::pattern;

pub mod prelude {
    pub use crate::mold;
    pub use crate::mold::Mold;
    pub use crate::pattern;
}

/// Macro maestra universal de un solo genérico basado en la función.
#[macro_export]
macro_rules! mold {
    ($funcion:expr) => {{
        use $crate::internal::FoundryFallbackRouter as _;

        // Forzamos la degradación implícita a puntero estándar
        let ptr_funcion: fn() -> _ = $funcion;

        // Invocación explícita por referencia para activar el Autoref condicional
        let bytes = (&ptr_funcion).__foundry_obtener_matriz();

        // Retornamos el Struct con tipado estricto por función
        $crate::mold::Mold::new_internal(ptr_funcion, bytes)
    }};
}
