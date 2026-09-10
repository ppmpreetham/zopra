use proc_macro::TokenStream;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{
    ExprCall, ExprPath, Local, LocalInit, Pat,
    visit_mut::{self, VisitMut},
};
use syn::{ItemFn, parse_macro_input, parse_quote};

#[derive(Default)]
struct InjectCx {
    signal_idents: HashSet<syn::Ident>,
}

// TODO: add more hooks to skip
static SKIP_FNS: &[&str] = &["use_state", "use_effect"];

/// goes through every single function call in the AST and injects `cx` into closures wherever required
impl VisitMut for InjectCx {
    fn visit_local_mut(&mut self, local: &mut Local) {
        visit_mut::visit_local_mut(self, local);

        if let Some(LocalInit { expr, .. }) = &local.init
          && let syn::Expr::Call(ExprCall { func, .. }) = &**expr
          && let syn::Expr::Path(ExprPath { path, .. }) = &**func
          // only last cuz people use smtg::create_signal
          && let Some(last_seg) = path.segments.last()
          && SKIP_FNS.iter().any(|f| last_seg.ident == f)
          && let Pat::Tuple(tuple) = &local.pat
        {
            self.signal_idents
                .extend(tuple.elems.iter().filter_map(|elem| match elem {
                    Pat::Ident(id) => Some(id.ident.clone()),
                    _ => None,
                }));
        }
    }

    fn visit_expr_call_mut(&mut self, call: &mut ExprCall) {
        visit_mut::visit_expr_call_mut(self, call);

        if let syn::Expr::Path(ExprPath { path, .. }) = &*call.func
            && let Some(seg) = path.segments.last()
            && (SKIP_FNS.iter().any(|f| seg.ident == f) || self.signal_idents.contains(&seg.ident))
        {
            if seg.ident == "use_state" {
                call.args.push(syn::parse_quote!(window));
            }
            call.args.push(syn::parse_quote!(cx));
        }
    }
}

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
    input
        .sig
        .inputs
        .push(parse_quote!(window: &mut gpui::Window));
    input.sig.inputs.push(parse_quote!(cx: &mut gpui::App));

    // injects cx into functions which need it
    let mut rewriter = InjectCx::default();
    rewriter.visit_block_mut(&mut input.block);

    TokenStream::from(input.to_token_stream())
}
