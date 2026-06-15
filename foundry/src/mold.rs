// foundry/src/mold.rs

#[macro_export]
macro_rules! mold {
    ($function:expr) => {{
        // 1. Mantenemos la identidad exacta del ítem de función original para la inferencia
        let original_fn = $function;

        // 2. Coaccionamos a puntero plano de forma higiénica para el Autoref de pattern.rs
        let function_ptr: fn() -> _ = original_fn;

        // Extraemos los metadatos y bytes usando el puente por Autoref legítimo
        let matrix_bytes = {
            let wrapper = (&&function_ptr).__foundry_extract_wrapper();
            wrapper.__foundry_get_matrix_live()
        };

        let expected_meta = {
            let wrapper = (&&function_ptr).__foundry_extract_wrapper();
            wrapper.__foundry_metadata()
        };

        // 3. LA SOLUCIÓN TÉCNICA LIMPIA (Inferencia unificada al instante):
        // Creamos una pequeña función interna en el bloque que acepta la función original.
        // Al usar la firma `fn() -> R` y devolver un `Pipeline<R>`, Rust se ve obligado a
        // extraer matemáticamente el tipo de retorno exacto (ej. String o u64) y ligarlo
        // a la estructura de datos final del Pipeline. Así el .map() sabrá el tipo real siempre.
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

        // Invocamos pasándole el puntero ya coaccionado para fijar de golpe los tipos del layout
        inferir_y_construir(function_ptr, matrix_bytes, expected_meta)
    }};
}
