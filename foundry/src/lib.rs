pub mod mold;
pub mod runtime;
pub mod vista;

pub use crate::mold as macro_mold;
pub use foundry_macros::pattern;

pub mod prelude {
    pub use crate::mold;
    pub use crate::mold::Mold;
    pub use crate::pattern;
    pub use crate::vista::Pipeline;
    pub use rkyv;
}

#[macro_export]
macro_rules! mold {
    ($function:expr) => {{
        let function_ptr = $function;

        struct InferenceAnchor<R>(std::marker::PhantomData<R>);
        impl<R: $crate::prelude::rkyv::Archive + 'static> InferenceAnchor<R> {
            #[inline(always)]
            fn anchor(function_ptr: fn() -> R) -> $crate::vista::Pipeline<R>
            where
                R: $crate::prelude::rkyv::Serialize<
                    $crate::prelude::rkyv::ser::serializers::AllocSerializer<256>,
                >,
            {
                // SECCIÓN METAL CON ALINEACIÓN FORZADA
                #[cfg(foundry_forged)]
                {
                    #[repr(align(8))]
                    struct AlignedData<const N: usize>([u8; N]);

                    static FORGED_PAYLOAD: &AlignedData<
                        {
                            include_bytes!(concat!(
                                env!("CARGO_MANIFEST_DIR"),
                                "/target/foundry_data/load_massive_array.matrix"
                            ))
                            .len()
                        },
                    > = &AlignedData(*include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/target/foundry_data/load_massive_array.matrix"
                    )));

                    $crate::vista::Pipeline::<R>::Forged(
                        &FORGED_PAYLOAD.0,
                        std::marker::PhantomData,
                    )
                }

                // MODO DESARROLLO / CAPTURA INITIAL
                #[cfg(not(foundry_forged))]
                {
                    // Trait de desvío universal por si la función no tiene #[pattern]
                    trait __FoundryGlobalFallback {
                        fn __foundry_discover(&self) -> $crate::runtime::DefaultRouter {
                            $crate::runtime::DefaultRouter
                        }
                    }
                    impl<F> __FoundryGlobalFallback for F {}

                    // Resolvemos el llamador de forma estática
                    let router = function_ptr.__foundry_discover();
                    let bytes = router.__foundry_get_matrix(function_ptr as usize);

                    if let Some(matrix_bytes) = bytes {
                        $crate::vista::Pipeline::<R>::Forged(matrix_bytes, std::marker::PhantomData)
                    } else {
                        let live_data = function_ptr();
                        let serialized_bytes =
                            $crate::prelude::rkyv::to_bytes::<_, 256>(&live_data).unwrap();
                        $crate::vista::Pipeline::<R>::Live(
                            serialized_bytes.to_vec(),
                            std::marker::PhantomData,
                        )
                    }
                }
            }
        }

        fn infer_target_type<R: $crate::prelude::rkyv::Archive>(
            _: fn() -> R,
        ) -> std::marker::PhantomData<R> {
            std::marker::PhantomData
        }

        let _ = infer_target_type(function_ptr);
        InferenceAnchor::anchor(function_ptr)
    }};
}
