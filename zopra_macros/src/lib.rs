use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{ItemFn, parse_macro_input, parse_quote};

/// Converts a function into a GPUI component
///
/// # Example
/// ```rust
/// #[component]
/// fn button(text: 'static str) {
///
/// }
/// ```
#[proc_macro_attribute]
pub fn component(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemFn);
    input.sig.output = parse_quote!(-> impl gpui::IntoElement);
    input.sig.inputs.push(parse_quote!(cx: &mut gpui::App));

    TokenStream::from(input.to_token_stream())
}
