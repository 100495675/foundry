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

    let body_tokens = quote!(#body);
    let body_str = body_tokens.to_string();
    let mut ast_hasher = DefaultHasher::new();
    body_str.hash(&mut ast_hasher);
    let ast_hash = ast_hasher.finish();

    let dep_hash = compute_dependency_hash();
    let test_capture_name =
        syn::Ident::new(&format!("__foundry_capture_for_{}", name), name.span());
    let wrapper_trait_name = syn::Ident::new(&format!("__FoundryTrait_{}", name), name.span());

    let type_str = quote!(#output_type).to_string().replace(" ", "");
    let type_bytes = type_str.as_bytes();
    let mut type_hash = 0xcbf29ce484222325u64;
    for &b in type_bytes {
        type_hash ^= b as u64;
        type_hash = type_hash.wrapping_mul(0x100000001b3u64);
    }

    let mut matrix_bytes_token = quote! { None };
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let matrix_path = std::path::Path::new(&manifest_dir)
            .join("target")
            .join("foundry_data")
            .join(format!("{}.matrix", name));

        if matrix_path.exists() {
            if let Ok(bytes) = std::fs::read(&matrix_path) {
                if bytes.len() >= 39 {
                    let mut saved_ast_bytes = [0u8; 8];
                    saved_ast_bytes.copy_from_slice(&bytes[7..15]);
                    let saved_ast_hash = u64::from_le_bytes(saved_ast_bytes);

                    let mut saved_type_bytes = [0u8; 8];
                    saved_type_bytes.copy_from_slice(&bytes[23..31]);
                    let saved_type_hash = u64::from_le_bytes(saved_type_bytes);

                    if saved_ast_hash == ast_hash && saved_type_hash == type_hash {
                        let len = bytes.len() + 1;
                        matrix_bytes_token = quote! {
                            {
                                #[repr(align(4))]
                                struct AlignedBytes<const N: usize>([u8; N]);

                                // Inyectamos el byte de alineación de hardware
                                static ALIGNED_STATIC: AlignedBytes<#len> = AlignedBytes([0u8, #(#bytes),*]);
                                Some(&ALIGNED_STATIC.0 as &[u8])
                            }
                        };
                    }
                }
            }
        }
    }

    let expanded = quote! {
        #(#attrs)*
        #vis #sig #body

        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        pub trait #wrapper_trait_name {
            fn __foundry_get_matrix(&self, target_ptr: usize) -> Option<&'static [u8]>;
        }

        impl #wrapper_trait_name for &fn() -> #output_type {
            #[inline(always)]
            fn __foundry_get_matrix(&self, target_ptr: usize) -> Option<&'static [u8]> {
                let current_ptr = #name as fn() -> #output_type as usize;
                if target_ptr == current_ptr {
                    #matrix_bytes_token
                } else {
                    None
                }
            }
        }

        #[cfg(test)]
        #[test]
        fn #test_capture_name() {
            use ::std::io::Write as _;

            let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
            let data_dir = std::path::Path::new(&manifest).join("target").join("foundry_data");
            let matrix_path = data_dir.join(format!("{}.matrix", stringify!(#name)));

            if let Ok(bytes) = std::fs::read(&matrix_path) {
                if bytes.len() >= 39 {
                    let mut ast_buf = [0u8; 8];
                    ast_buf.copy_from_slice(&bytes[7..15]);
                    let saved_ast = u64::from_le_bytes(ast_buf);

                    let mut type_buf = [0u8; 8];
                    type_buf.copy_from_slice(&bytes[23..31]);
                    let saved_type = u64::from_le_bytes(type_buf);

                    if saved_ast == #ast_hash && saved_type == #type_hash {
                        return;
                    }
                }
            }

            let object = #name();
            let serializer = ::foundry::internal::rkyv::to_bytes::<_, 256>(&object)
                .expect("foundry: Failed to serialize matrix data via rkyv");
            let payload = serializer.into_vec();

            std::fs::create_dir_all(&data_dir).unwrap();
            let mut file = std::fs::File::create(&matrix_path).unwrap();
            file.write_all(b"MATR").unwrap();
            file.write_all(&[0x02]).unwrap();
            file.write_all(&1u16.to_le_bytes()).unwrap();
            file.write_all(&#ast_hash.to_le_bytes()).unwrap();
            file.write_all(&#dep_hash.to_le_bytes()).unwrap();
            file.write_all(&#type_hash.to_le_bytes()).unwrap();
            file.write_all(&(payload.len() as u64).to_le_bytes()).unwrap();
            file.write_all(&payload).unwrap();
        }
    };

    TokenStream::from(expanded)
}

fn compute_dependency_hash() -> u64 {
    let mut hasher = DefaultHasher::new();
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let lock_path = std::path::Path::new(&manifest_dir).join("Cargo.lock");
        if let Ok(content) = std::fs::read_to_string(&lock_path) {
            content.hash(&mut hasher);
            return hasher.finish();
        }
    }
    hasher.finish()
}
