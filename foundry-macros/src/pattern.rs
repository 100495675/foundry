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

    let test_capture_name =
        syn::Ident::new(&format!("__foundry_capture_for_{}", name), name.span());

    // 🛠️ FIX DEFINITIVO: Fabricamos los identificadores como tokens puros
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
            pub fn __foundry_get_matrix(&self, target_ptr: usize) -> Option<&'static [u8]> {
                let current_ptr = #name as fn() -> #output_type as usize;
                if target_ptr == current_ptr {
                    let manifest_dir = env!("CARGO_MANIFEST_DIR");
                    let matrix_path = std::path::Path::new(manifest_dir)
                        .join("target")
                        .join("foundry_data")
                        .join(concat!(stringify!(#name), ".matrix"));

                    if matrix_path.exists() {
                        if let Ok(bytes) = std::fs::read(matrix_path) {
                            return Some(Box::leak(bytes.into_boxed_slice()));
                        }
                    }
                }
                None
            }
        }

        // 3. Extensión local acoplada al puntero primitivo usando el token precalculado
        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        pub trait #discovery_trait_name {
            fn __foundry_discover(&self) -> #wrapper_struct_name { #wrapper_struct_name }
        }

        // Conexión del puente limpio
        impl #discovery_trait_name for fn() -> #output_type {}

        #[cfg(test)]
        #[test]
        fn #test_capture_name() {
            use ::std::io::Write as _;

            let object = #name();
            let serializer = ::foundry::prelude::rkyv::to_bytes::<_, 256>(&object)
                .expect("foundry: Failed to serialize matrix data via rkyv");
            let payload = serializer.into_vec();

            let out_dir = std::env::var("FOUNDRY_OUT_DIR_INJECT").expect("Missing FOUNDRY_OUT_DIR_INJECT");
            let data_dir = std::path::Path::new(&out_dir);
            let matrix_path = data_dir.join(format!("{}.matrix", stringify!(#name)));

            std::fs::create_dir_all(&data_dir).unwrap();
            let mut file = std::fs::File::create(&matrix_path).unwrap();
            file.write_all(&payload).unwrap();
        }
    };

    TokenStream::from(expanded)
}
