extern crate proc_macro;

mod pattern;

#[proc_macro_attribute]
pub fn pattern(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    pattern::pattern_impl(attr, item)
}
