pub mod mold;
pub mod mold_runtime;

pub mod internal {
    pub use bincode;
    pub use crate::mold_runtime::bincode_options;

    // Trait fallback global por defecto
    pub trait FoundryFallbackRouter {
        #[inline(always)]
        fn __foundry_obtener_matriz(&self) -> Option<&'static [u8]> {
            None
        }
    }

    // Se aplica a la referencia de un puntero de función plano de forma exacta
    impl<T> FoundryFallbackRouter for &fn() -> T {}
}

pub use foundry_macros::pattern;

pub mod prelude {
    pub use crate::mold::Mold;
    pub use crate::pattern;
    pub use crate::mold;
}

/// Macro maestra universal definitiva.
#[macro_export]
macro_rules! mold {
    ($funcion:expr) => {
        {
            // Importamos el router fallback global
            use $crate::internal::FoundryFallbackRouter as _;

            // Estabilizamos el tipo a un puntero de función plano
            let ptr_funcion: fn() -> _ = $funcion;

            // ¡EL CAMBIO AQUÍ!: Le metemos el ampersand explícito.
            // Esto cumple de forma exacta con lo que el compilador exige cuando no hay macro.
            let bytes = (&ptr_funcion).__foundry_obtener_matriz();

            let funcion_capturada = $funcion;
            let clausura_fallback = move || funcion_capturada();

            $crate::mold::Mold::new_internal(Box::new(clausura_fallback), bytes)
        }
    };
}