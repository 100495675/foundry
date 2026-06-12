pub mod mold;
pub mod mold_runtime;

pub mod internal {
    pub use crate::mold_runtime::bincode_options;
    pub use bincode;

    // Trait fallback global por defecto para el Autoref
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
    pub use crate::mold;
    pub use crate::mold::Mold;
    pub use crate::pattern; // Exportamos la macro maestra mold!
}

/// Macro maestra universal purista de un solo genérico compatible con Rust Estable.
#[macro_export]
macro_rules! mold {
    ($funcion:expr) => {{
        use $crate::internal::FoundryFallbackRouter as _;

        // Estabilizamos el tipo al puntero de función plano nativo
        let ptr_funcion: fn() -> _ = $funcion;

        // Buscamos el método en el trait de extensión local o global
        let bytes = (&ptr_funcion).__foundry_obtener_matriz();

        // Pasamos la función pura por valor. Cero coste en el Heap.
        $crate::mold::Mold::new_internal(ptr_funcion, bytes)
    }};
}
