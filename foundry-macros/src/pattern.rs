// foundry-macros/src/pattern.rs
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

    let name_str = name.to_string();
    let test_capture_name =
        syn::Ident::new(&format!("__foundry_capture_for_{}", name), name.span());
    let wrapper_struct_name = syn::Ident::new(&format!("__FoundryWrapper_{}", name), name.span());
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
            pub fn __foundry_metadata(&self) -> ::foundry::core::PatternMetadata {
                ::foundry::core::PatternMetadata {
                    name_hash: 0,
                    type_hash: 0,
                    payload_len: 0,
                    reserved_1: 0,
                    magic: *::foundry::core::MATRIX_MAGIC, // 🛠️ FIJADO: Ahora el control espera b"MATR"
                    schema_ver: 1,
                    version: ::foundry::core::MATRIX_VERSION, // 🛠️ FIJADO: Ahora el control espera la versión real (2)
                    padding: 0,
                }
            }
        }

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

        #[test]
        #[doc(hidden)]
        fn #test_capture_name() {
            use ::std::io::Write as _;

            let object = #name();
            let serializer = ::foundry::prelude::rkyv::to_bytes::<_, 256>(&object)
                .expect("foundry: Fallo al serializar la matriz en la fase de captura");

            let payload = serializer.into_vec();

            let raw_out_dir = ::std::env::var("FOUNDRY_OUT_DIR_INJECT")
                .unwrap_or_else(|_| "target/foundry_data".to_string());

            let base_path = ::std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            let data_dir = if ::std::path::Path::new(&raw_out_dir).is_absolute() {
                ::std::path::PathBuf::from(raw_out_dir)
            } else {
                base_path.join(raw_out_dir)
            };

            ::std::fs::create_dir_all(&data_dir).expect("foundry: No se pudo crear el directorio de captura");
            let matrix_path = data_dir.join(format!("{}.matrix", #name_str));
            let mut file = ::std::fs::File::create(&matrix_path).expect("foundry: No se pudo crear el archivo de matriz");

            let header = ::foundry::core::PatternMetadata {
                name_hash: 0,
                type_hash: 0,
                payload_len: payload.len() as u64,
                reserved_1: 0,
                magic: *::foundry::core::MATRIX_MAGIC,
                schema_ver: 1,
                version: ::foundry::core::MATRIX_VERSION,
                padding: 0,
            };

            let mut header_bytes = [0u8; 40];
            unsafe {
                ::std::ptr::write_unaligned(header_bytes.as_mut_ptr() as *mut ::foundry::core::PatternMetadata, header);
            }

            file.write_all(&header_bytes).unwrap();
            file.write_all(&payload).expect("foundry: Error al escribir los bytes serializados");
        }
    };

    TokenStream::from(expanded)
}
