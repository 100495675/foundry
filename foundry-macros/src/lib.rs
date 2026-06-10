extern crate proc_macro;

mod derive_shape;
mod pattern;

/// Macro de derivación automática para el layout de memoria seguro.
#[proc_macro_derive(Shape)]
pub fn derive_shape(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_shape::derive_shape_impl(input)
}

/// Macro de atributo atómica para registrar patrones en la fragua.
#[proc_macro_attribute]
pub fn pattern(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    pattern::pattern_impl(attr, item)
}
