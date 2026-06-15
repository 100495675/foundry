// foundry/src/mold.rs

#[macro_export]
macro_rules! mold {
    ($function:ident) => {{
        let original_fn = $function;
        let function_ptr: fn() -> _ = original_fn;

        #[cfg(test)]
        mod __foundry_env_bridge {
            #[allow(dead_code)]
            pub fn get_matrix_bytes(_name: &str) -> Option<&'static [u8]> {
                None
            }
        }

        #[cfg(not(test))]
        mod __foundry_env_bridge {
            include!(concat!(env!("OUT_DIR"), "/foundry_env.rs"));
        }

        #[allow(dead_code)]
        struct __FoundryFallbackWrapper;
        impl __FoundryFallbackWrapper {
            #[inline(always)]
            fn __foundry_metadata(&self) -> $crate::core::PatternMetadata {
                $crate::core::PatternMetadata::default()
            }
        }

        #[allow(non_camel_case_types)]
        trait __FoundryUniversalFallback {
            fn __foundry_extract_wrapper(&self) -> __FoundryFallbackWrapper;
        }

        impl<F> __FoundryUniversalFallback for &&&F {
            #[inline(always)]
            fn __foundry_extract_wrapper(&self) -> __FoundryFallbackWrapper {
                __FoundryFallbackWrapper
            }
        }

        let matrix_bytes = __foundry_env_bridge::get_matrix_bytes(stringify!($function));

        let expected_meta = {
            let wrapper = (&&&function_ptr).__foundry_extract_wrapper();
            wrapper.__foundry_metadata()
        };

        #[inline(always)]
        fn inferir_y_construir<R: $crate::prelude::rkyv::Archive>(
            _f: fn() -> R,
            matrix_bytes: Option<&'static [u8]>,
            expected_meta: $crate::core::PatternMetadata,
        ) -> $crate::vista::Pipeline<R>
        where
            R: $crate::prelude::rkyv::Serialize<
                $crate::prelude::rkyv::ser::serializers::AllocSerializer<256>,
            >,
        {
            if let Some(bytes) = matrix_bytes {
                if $crate::vista::validar_matriz_auditoria(bytes, &expected_meta) {
                    return $crate::vista::Pipeline::Forged(
                        bytes,
                        expected_meta,
                        std::marker::PhantomData,
                    );
                }
            }

            let live_data = _f();
            let serialized_bytes = $crate::prelude::rkyv::to_bytes::<_, 256>(&live_data).unwrap();
            $crate::vista::Pipeline::Live(serialized_bytes.to_vec(), std::marker::PhantomData)
        }

        inferir_y_construir(function_ptr, matrix_bytes, expected_meta)
    }};
}
