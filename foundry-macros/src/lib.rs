use proc_macro::TokenStream;

mod pattern;

#[proc_macro_attribute]
pub fn pattern(attr: TokenStream, item: TokenStream) -> TokenStream {
    pattern::pattern_impl(attr, item)
}
