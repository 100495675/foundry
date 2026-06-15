pub mod mold;
pub mod runtime;
pub mod vista;

pub use foundry_macros::pattern;
pub use crate::mold as macro_mold;

pub mod prelude {
    pub use crate::mold;
    pub use crate::mold::Mold;
    pub use crate::pattern;
    pub use crate::vista::Pipeline;
    pub use rkyv;
}

#[macro_export]
macro_rules! mold {
    ($function:expr) => {
        {
            let function_ptr = $function;

            struct InferenceAnchor<R>(std::marker::PhantomData<R>);
            impl<R: $crate::prelude::rkyv::Archive + 'static> InferenceAnchor<R> {
                #[inline(always)]
                fn anchor(function_ptr: fn() -> R) -> $crate::vista::Pipeline<R>
                where
                    R: $crate::prelude::rkyv::Serialize<$crate::prelude::rkyv::ser::serializers::AllocSerializer<256>>
                {
                    // 🏎️ SI EL SCRIPT DE BUILD GENERÓ EL ARCHIVO:
                    // include_bytes! incrusta el archivo directamente en la sección .rodata
                    // del binario en tiempo de compilación. Coste de I/O en Runtime: CERO ABSOLUTO.
                    #[cfg(foundry_forged)]
                    {
                        static MATRIX_BYTES: &[u8] = include_bytes!(concat!(
                            env!("CARGO_MANIFEST_DIR"),
                            "/target/foundry_data/load_massive_array.matrix"
                        ));
                        $crate::vista::Pipeline::<R>::Forged(MATRIX_BYTES, std::marker::PhantomData)
                    }

                    // 💻 SI NO EXISTE EL ARCHIVO (Fase de captura o desarrollo inicial):
                    #[cfg(not(foundry_forged))]
                    {
                        let live_data = function_ptr();
                        let serialized_bytes = $crate::prelude::rkyv::to_bytes::<_, 256>(&live_data).unwrap();
                        $crate::vista::Pipeline::<R>::Live(serialized_bytes.to_vec(), std::marker::PhantomData)
                    }
                }
            }

            fn infer_target_type<R: $crate::prelude::rkyv::Archive>(_: fn() -> R) -> std::marker::PhantomData<R> {
                std::marker::PhantomData
            }

            let _ = infer_target_type(function_ptr);
            InferenceAnchor::anchor(function_ptr)
        }
    };
}