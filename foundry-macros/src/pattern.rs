use proc_macro::TokenStream;
use quote::quote;
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

    // Algoritmo FNV-1a estático para el name_hash basado en el Identificador Inmune
    let name_str = name.to_string();
    let mut name_hash = 0xcbf29ce484222325u64;
    for &b in name_str.as_bytes() {
        name_hash ^= b as u64;
        name_hash = name_hash.wrapping_mul(0x100000001b3u64);
    }

    // Algoritmo FNV-1a estático para el type_hash
    let type_str = quote!(#output_type).to_string().replace(" ", "");
    let mut type_hash = 0xcbf29ce484222325u64;
    for &b in type_str.as_bytes() {
        type_hash ^= b as u64;
        type_hash = type_hash.wrapping_mul(0x100000001b3u64);
    }

    let test_capture_name =
        syn::Ident::new(&format!("__foundry_capture_for_{}", name), name.span());
    let wrapper_struct_name = syn::Ident::new(&format!("__FoundryWrapper_{}", name), name.span());

    // Trait local único por función para el descubrimiento directo por Autoref
    let discovery_trait_name =
        syn::Ident::new(&format!("__FoundryLocalDiscovery_{}", name), name.span());

    let expanded = quote! {
        #(#attrs)*
        #vis #sig #body

        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        pub struct #wrapper_struct_name;

        impl #wrapper_struct_name {
            #[inline(always)]
            pub fn __foundry_get_matrix_live(&self) -> Option<&'static [u8]> {
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

            #[inline(always)]
            pub fn __foundry_metadata(&self) -> ::foundry::core::PatternMetadata {
                ::foundry::core::PatternMetadata {
                    name_hash: #name_hash, // 🛠️ CORRECCIÓN: Campo real de foundry-core
                    type_hash: #type_hash,
                    payload_len: 0,
                    reserved: 0,
                    magic: *::foundry::core::MATRIX_MAGIC,
                    schema_ver: 1,
                    version: ::foundry::core::MATRIX_VERSION,
                    padding: 0,
                }
            }
        }

        // El puente de descubrimiento por Autoref local
        #[allow(non_camel_case_types)]
        pub trait #discovery_trait_name {
            type Wrapper;
            fn __foundry_extract_wrapper(&self) -> Self::Wrapper;
        }

        impl #discovery_trait_name for &fn() -> #output_type {
            type Wrapper = #wrapper_struct_name;
            #[inline(always)]
            fn __foundry_extract_wrapper(&self) -> Self::Wrapper {
                #wrapper_struct_name
            }
        }

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

            let header = ::foundry::core::PatternMetadata {
                name_hash: #name_hash, // 🛠️ CORRECCIÓN: Campo real de foundry-core
                type_hash: #type_hash,
                payload_len: payload.len() as u64,
                reserved: 0,
                magic: *::foundry::core::MATRIX_MAGIC,
                schema_ver: 1,
                version: ::foundry::core::MATRIX_VERSION,
                padding: 0,
            };

            let header_bytes: &[u8; 40] = unsafe { &*(&header as *const ::foundry::core::PatternMetadata as *const [u8; 40]) };

            file.write_all(header_bytes).unwrap();
            file.write_all(&payload).expect("foundry: Error al escribir los bytes serializados en disco");
        }
    };

    TokenStream::from(expanded)
}
