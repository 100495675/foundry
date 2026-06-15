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

    let _output_type: syn::Type = match &input.sig.output {
        ReturnType::Type(_, ty) => *ty.clone(),
        ReturnType::Default => syn::parse_quote!(()),
    };

    let test_capture_name =
        syn::Ident::new(&format!("__foundry_capture_for_{}", name), name.span());

    let expanded = quote! {
        #(#attrs)*
        #vis #sig #body

        // El test de captura intacto que tu foundry-build ejecutará
        #[cfg(test)]
        #[test]
        fn #test_capture_name() {
            use ::std::io::Write as _;

            let object = #name();
            let serializer = ::foundry::prelude::rkyv::to_bytes::<_, 256>(&object)
                .expect("foundry: Failed to serialize matrix data via rkyv");
            let payload = serializer.into_vec();

            // Guardamos el payload crudo de rkyv directamente en la carpeta OUT_DIR de Cargo
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
