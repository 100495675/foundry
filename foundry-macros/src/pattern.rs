use proc_macro::TokenStream;
use quote::quote;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use syn::{parse_macro_input, ItemFn, ReturnType};

pub fn pattern_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let name = &input.sig.ident;
    let vis = &input.vis;
    let body = &input.block;
    let attrs = &input.attrs;
    let sig = &input.sig;

    let output_type: syn::Type = match &input.sig.output {
        ReturnType::Type(_, ty) => *ty.clone(),
        ReturnType::Default => syn::parse_quote!(()),
    };

    // 🛠️ RESTAURACIÓN DE METADATOS 1: Calcular hash estático del AST de la función
    let body_tokens = quote!(#body);
    let body_str = body_tokens.to_string();
    let mut ast_hasher = DefaultHasher::new();
    body_str.hash(&mut ast_hasher);
    let ast_hash = ast_hasher.finish();

    // 🛠️ RESTAURACIÓN DE METADATOS 2: Calcular hash del tipo de retorno (FNV-1a simplificado)
    let type_str = quote!(#output_type).to_string().replace(" ", "");
    let type_bytes = type_str.as_bytes();
    let mut type_hash = 0xcbf29ce484222325u64;
    for &b in type_bytes {
        type_hash ^= b as u64;
        type_hash = type_hash.wrapping_mul(0x100000001b3u64);
    }

    let test_capture_name =
        syn::Ident::new(&format!("__foundry_capture_for_{}", name), name.span());

    let wrapper_struct_name = syn::Ident::new(&format!("__FoundryWrapper_{}", name), name.span());
    let discovery_trait_name =
        syn::Ident::new(&format!("__FoundryDiscovery_{}", name), name.span());

    let expanded = quote! {
        #(#attrs)*
        #vis #sig #body

        // 1. Tipo local único para esquivar la regla de orfandad
        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        pub struct #wrapper_struct_name;

        // 2. Implementación inherente libre de traits y restricciones
        impl #wrapper_struct_name {
            #[inline(always)]
            pub fn __foundry_get_matrix(&self) -> Option<&'static [u8]> {
                static CACHE: ::std::sync::OnceLock<Option<&'static [u8]>> = ::std::sync::OnceLock::new();

                *CACHE.get_or_init(|| {
                    let manifest_dir = env!("CARGO_MANIFEST_DIR");
                    let matrix_path = ::std::path::Path::new(manifest_dir)
                        .join("target")
                        .join("foundry_data")
                        .join(concat!(stringify!(#name), ".matrix"));

                    if matrix_path.exists() {
                        if let Ok(bytes) = ::std::fs::read(matrix_path) {
                            return Some(::std::boxed::Box::leak(bytes.into_boxed_slice()));
                        }
                    }
                    None
                })
            }
        }

        // 3. Extensión local acoplada al puntero primitivo usando el token precalculado
        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        pub trait #discovery_trait_name {
            fn __foundry_discover(&self) -> #wrapper_struct_name { #wrapper_struct_name }
        }

        impl #discovery_trait_name for fn() -> #output_type {}

        #[cfg(test)]
        #[test]
        fn #test_capture_name() {
            use ::std::io::Write as _;

            let object = #name();
            let serializer = ::foundry::prelude::rkyv::to_bytes::<_, 256>(&object)
                .expect("foundry: Fallo al serializar la matriz en la fase de captura");

            let payload = serializer.into_vec();

            let out_dir = ::std::env::var("FOUNDRY_OUT_DIR_INJECT")
                .expect("foundry: Variable FOUNDRY_OUT_DIR_INJECT no definida en el script de build");
            let data_dir = ::std::path::Path::new(&out_dir);
            let matrix_path = data_dir.join(format!("{}.matrix", stringify!(#name)));

            ::std::fs::create_dir_all(&data_dir).expect("foundry: No se pudo crear el directorio de captura de datos");
            let mut file = ::std::fs::File::create(&matrix_path).expect("foundry: No se pudo crear el archivo de matriz");

            // ESCRITURA DE CABECERA ALINEADA (Total: 40 Bytes con el padding de hardware)
            file.write_all(b"MATR").unwrap();                              // 4 bytes
            file.write_all(&[0x02]).unwrap();                             // 1 byte
            file.write_all(&1u16.to_le_bytes()).unwrap();                 // 2 bytes
            file.write_all(&#ast_hash.to_le_bytes()).unwrap();            // 8 bytes
            file.write_all(&0u64.to_le_bytes()).unwrap();                 // 8 bytes
            file.write_all(&#type_hash.to_le_bytes()).unwrap();           // 8 bytes
            file.write_all(&(payload.len() as u64).to_le_bytes()).unwrap(); // 8 bytes
            file.write_all(&[0x00]).unwrap();                             // 1 byte de padding físico

            file.write_all(&payload).expect("foundry: Error al escribir los bytes serializados en disco");
        }
    };

    TokenStream::from(expanded)
}
