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
    let raw_name = syn::Ident::new(&format!("__foundry_raw_{}", name), name.span());

    // Sincronización Clave 1: El nombre del test debe coincidir con el filtro de foundry-build
    let test_capture_name =
        syn::Ident::new(&format!("__foundry_capture_for_{}", name), name.span());

    // --- 1. ESCÁNER DE COMPILACIÓN CON FILTRADO DE SEGURIDAD ---
    let mut matrix_bytes_token = quote! { None };
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let matrix_path = std::path::Path::new(&manifest_dir)
            .join("foundry_data")
            .join(format!("{}.matrix", name));

        if matrix_path.exists() {
            if let Ok(bytes) = std::fs::read(&matrix_path) {
                if bytes.len() >= 31 {
                    let mut saved_ast_bytes = [0u8; 8];
                    saved_ast_bytes.copy_from_slice(&bytes[7..15]);
                    let saved_ast_hash = u64::from_le_bytes(saved_ast_bytes);

                    // Si el código NO ha cambiado, preparamos los bytes para la inyección
                    if saved_ast_hash == ast_hash {
                        matrix_bytes_token = quote! { Some(&[#(#bytes),*]) };
                    }
                }
            }
        }
    }

    // --- 2. EXPANSIÓN SINTÁCTICA ---
    let expanded = quote! {
        #[inline(never)]
        #(#attrs)*
        #vis fn #raw_name() -> #output_type {
            #body
        }

        #[allow(non_camel_case_types)]
        #[derive(Copy, Clone)]
        #vis struct #name;

        impl ::foundry::internal::Pattern for #name {
            type Output = #output_type;
            const AST_HASH: u64 = #ast_hash;
            const DEPENDENCY_HASH: u64 = #dep_hash;

            // Sincronización Clave 2: Solo si foundry-build valida el proceso (foundry_baked),
            // inyectamos los bytes evaluados en la constante. Si no, forzamos None.
            const BAKED_TEMPLATE: Option<&'static [u8]> = {
                #[cfg(foundry_baked)]
                { #matrix_bytes_token }
                #[cfg(not(foundry_baked))]
                { None }
            };

            #[inline(always)]
            fn execute() -> Self::Output {
                #raw_name()
            }
        }

        impl ::std::ops::Deref for #name {
            type Target = fn() -> #output_type;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                static TARGET_PTR: fn() -> #output_type = #raw_name;
                &TARGET_PTR
            }
        }

        // --- TEST DE FRAGUA AUTOMATIZADO ---
        #[cfg(feature = "foundry-capture")]
        #[test]
        fn #test_capture_name() {
            use ::std::io::Write as _;

            let objeto = #raw_name();
            let payload = ::foundry::internal::bincode::serialize(&objeto).unwrap();

            let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
            let carpeta = std::path::Path::new(&manifest).join("foundry_data");
            std::fs::create_dir_all(&carpeta).unwrap();

            let ruta_archivo = carpeta.join(format!("{}.matrix", stringify!(#name)));
            let mut file = std::fs::File::create(&ruta_archivo).unwrap();

            file.write_all(b"MATR").unwrap();
            file.write_all(&[0x01]).unwrap(); // Little Endian
            file.write_all(&1u16.to_le_bytes()).unwrap();
            file.write_all(&#ast_hash.to_le_bytes()).unwrap();
            file.write_all(&#dep_hash.to_le_bytes()).unwrap();
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
