// foundry/src/mold.rs

#[macro_export]
macro_rules! mold {
    ($function:expr) => {{
        let original_fn = $function;
        let function_ptr: fn() -> _ = original_fn;

        #[allow(dead_code)]
        struct __FoundryFallbackWrapper;
        impl __FoundryFallbackWrapper {
            #[inline(always)]
            fn __foundry_get_matrix_live(&self) -> Option<&'static [u8]> {
                None
            }
            #[inline(always)]
            fn __foundry_metadata(&self) -> $crate::core::PatternMetadata {
                $crate::core::PatternMetadata::default()
            }
        }

        #[allow(non_camel_case_types)]
        trait __FoundryUniversalFallback {
            fn __foundry_extract_wrapper(&self) -> __FoundryFallbackWrapper;
        }

        // 🛠️ CORRECCIÓN DE PRIORIDAD DE AUTOREF:
        // Implementamos el fallback para `&&&F`. Al requerir más ampersands,
        // Rust siempre preferirá el trait de `pattern.rs` (que pide menos) si está presente.
        impl<F> __FoundryUniversalFallback for &&&F {
            #[inline(always)]
            fn __foundry_extract_wrapper(&self) -> __FoundryFallbackWrapper {
                __FoundryFallbackWrapper
            }
        }

        // Pasamos tres ampersands para activar la cadena de desempate en cascada
        let matrix_bytes = {
            let wrapper = (&&&function_ptr).__foundry_extract_wrapper();
            wrapper.__foundry_get_matrix_live()
        };

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
                $crate::vista::validar_matriz_auditoria(bytes, &expected_meta);
                $crate::vista::Pipeline::Forged(bytes, expected_meta, std::marker::PhantomData)
            } else {
                let live_data = _f();
                let serialized_bytes =
                    $crate::prelude::rkyv::to_bytes::<_, 256>(&live_data).unwrap();
                $crate::vista::Pipeline::Live(serialized_bytes.to_vec(), std::marker::PhantomData)
            }
        }

        inferir_y_construir(function_ptr, matrix_bytes, expected_meta)
    }};
}
