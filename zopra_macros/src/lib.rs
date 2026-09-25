use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, Punct, Spacing, Span, TokenStream as TokenStream2, TokenTree};
use std::collections::HashSet;
use syn::{
    parse_macro_input, parse_quote,
    visit::{self, Visit},
    visit_mut::{self, VisitMut},
    Expr, ExprCall, ExprClosure, ExprForLoop, ExprIf, ExprMatch, ExprWhile, ItemFn, Local, Macro,
    Pat,
};

const SKIP_FNS: &[&str] = &["use_state", "use_effect"];

// hooks where cx must be injected into the closure argument's own params
const CLOSURE_CX_FNS: &[&str] = &["use_callback", "use_event"];
const ASYNC_CX_FNS: &[&str] = &["use_async"];

/// true if the expression is exactly the identifier `name`
fn is_bare_ident(arg: &Expr, name: &str) -> bool {
    matches!(arg, Expr::Path(p) if p.path.segments.last().is_some_and(|s| s.ident == name))
}

/// true if the call path refers to a zopra hook: either unqualified
fn is_hook_path(path: &syn::Path, fns: &[&str]) -> bool {
    let Some(seg) = path.segments.last() else {
        return false;
    };
    if !fns.contains(&seg.ident.to_string().as_str()) {
        return false;
    }
    path.segments.len() == 1 || path.segments.first().is_some_and(|s| s.ident == "zopra")
}

/// Traverses any complex `syn::Pat` to harvest every bound identifier,
/// so nested destructures like `let ((count, set_count), _) = use_state(..)`
/// are handled, not just flat tuples.
#[derive(Default)]
struct PatIdentCollector {
    idents: Vec<syn::Ident>,
}

impl<'ast> Visit<'ast> for PatIdentCollector {
    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        self.idents.push(node.ident.clone());
        visit::visit_pat_ident(self, node);
    }
}

fn collect_idents_from_pat(pat: &Pat) -> Vec<syn::Ident> {
    let mut collector = PatIdentCollector::default();
    collector.visit_pat(pat);
    collector.idents
}

#[derive(Default)]
struct Scope {
    signals: HashSet<String>,
    plain: HashSet<String>,
}

#[derive(Default)]
struct InjectCx {
    scopes: Vec<Scope>,
}

impl InjectCx {
    fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn is_signal(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.plain.contains(name) {
                return false;
            }
            if scope.signals.contains(name) {
                return true;
            }
        }
        false
    }

    fn register_pat(&mut self, pat: &Pat, as_signal: bool) {
        let names = collect_idents_from_pat(pat)
            .into_iter()
            .map(|ident| ident.to_string())
            .collect::<Vec<_>>();
        if let Some(scope) = self.scopes.last_mut() {
            for name in names {
                if as_signal {
                    scope.signals.insert(name);
                } else {
                    scope.plain.insert(name);
                }
            }
        }
    }

    fn init_is_hook_call(local: &Local) -> bool {
        local.init.as_ref().is_some_and(|init| {
            matches!(&*init.expr, Expr::Call(ExprCall { func, .. })
                if matches!(&**func, Expr::Path(p) if is_hook_path(&p.path, SKIP_FNS)))
        })
    }

    fn rewrite_signal_calls(&self, tokens: &mut TokenStream2) {
        let toks: Vec<TokenTree> = tokens.clone().into_iter().collect();
        let mut out: Vec<TokenTree> = Vec::with_capacity(toks.len());
        let mut i = 0;

        while i < toks.len() {
            match &toks[i] {
                TokenTree::Group(group) => {
                    let mut inner = group.stream();
                    self.rewrite_signal_calls(&mut inner);
                    out.push(TokenTree::Group(rebuild_group(group, inner, group.delimiter())));
                    i += 1;
                }
                TokenTree::Ident(ident)
                    if self.is_signal(&ident.to_string())
                        && let Some(TokenTree::Group(g)) = toks.get(i + 1)
                        && g.delimiter() == Delimiter::Parenthesis =>
                {
                    let is_method_or_path = i > 0
                        && matches!(
                            &toks[i - 1],
                            TokenTree::Punct(p) if p.as_char() == '.' || p.as_char() == ':'
                        );

                    if is_method_or_path {
                        out.push(TokenTree::Ident(ident.clone()));
                        i += 1;
                        continue;
                    }

                    let group = g.clone();
                    let mut inner = group.stream();

                    let ends_with_cx = matches!(
                        inner.clone().into_iter().last(),
                        Some(TokenTree::Ident(id)) if id == "cx"
                    );

                    self.rewrite_signal_calls(&mut inner);

                    if !ends_with_cx {
                        if !inner.is_empty() {
                            let mut comma = Punct::new(',', Spacing::Alone);
                            comma.set_span(group.span());
                            inner.extend(std::iter::once(TokenTree::Punct(comma)));
                        }
                        inner.extend(std::iter::once(TokenTree::Ident(
                            proc_macro2::Ident::new("cx", Span::call_site()),
                        )));
                    }

                    out.push(TokenTree::Ident(ident.clone()));
                    out.push(TokenTree::Group(rebuild_group(
                        &group,
                        inner,
                        Delimiter::Parenthesis,
                    )));
                    i += 2;
                }
                other => {
                    out.push(other.clone());
                    i += 1;
                }
            }
        }
        *tokens = out.into_iter().collect();
    }
}

impl VisitMut for InjectCx {
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        self.push_scope();
        visit_mut::visit_block_mut(self, block);
        self.pop_scope();
    }

    fn visit_expr_closure_mut(&mut self, closure: &mut ExprClosure) {
        self.push_scope();
        for input in &closure.inputs {
            self.register_pat(input, false);
        }
        visit_mut::visit_expr_closure_mut(self, closure);
        self.pop_scope();
    }

    fn visit_local_mut(&mut self, local: &mut Local) {
        visit_mut::visit_local_mut(self, local);
        let as_signal = Self::init_is_hook_call(local);
        self.register_pat(&local.pat, as_signal);
    }

    fn visit_expr_match_mut(&mut self, expr_match: &mut ExprMatch) {
        self.visit_expr_mut(&mut expr_match.expr);
        for arm in &mut expr_match.arms {
            self.push_scope();
            self.register_pat(&arm.pat, false);
            if let Pat::Guard(guard_pat) = &mut arm.pat {
                self.visit_expr_mut(&mut guard_pat.guard);
            }
            self.visit_expr_mut(&mut arm.body);
            self.pop_scope();
        }
    }

    fn visit_expr_if_mut(&mut self, expr_if: &mut ExprIf) {
        self.visit_expr_mut(&mut expr_if.cond);

        self.push_scope();
        if let Expr::Let(expr_let) = &*expr_if.cond {
            self.register_pat(&expr_let.pat, false);
        }
        self.visit_block_mut(&mut expr_if.then_branch);
        self.pop_scope();

        if let Some((_, else_expr)) = &mut expr_if.else_branch {
            self.visit_expr_mut(else_expr);
        }
    }

    fn visit_expr_while_mut(&mut self, expr_while: &mut ExprWhile) {
        self.visit_expr_mut(&mut expr_while.cond);

        self.push_scope();
        if let Expr::Let(expr_let) = &*expr_while.cond {
            self.register_pat(&expr_let.pat, false);
        }
        self.visit_block_mut(&mut expr_while.body);
        self.pop_scope();
    }

    fn visit_expr_for_loop_mut(&mut self, expr_for_loop: &mut ExprForLoop) {
        self.visit_expr_mut(&mut expr_for_loop.expr);

        self.push_scope();
        self.register_pat(&expr_for_loop.pat, false);
        self.visit_block_mut(&mut expr_for_loop.body);
        self.pop_scope();
    }

    fn visit_expr_call_mut(&mut self, call: &mut ExprCall) {
        visit_mut::visit_expr_call_mut(self, call);

        let Expr::Path(path_expr) = &*call.func else {
            return;
        };
        let name = path_expr
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        let is_hook = is_hook_path(&path_expr.path, SKIP_FNS)
            || is_hook_path(&path_expr.path, CLOSURE_CX_FNS)
            || is_hook_path(&path_expr.path, ASYNC_CX_FNS);

        if is_hook || self.is_signal(&name) {
            let ends_with_window_cx = call.args.len() >= 2
                && is_bare_ident(call.args.last().unwrap(), "cx")
                && is_bare_ident(&call.args[call.args.len() - 2], "window");
            if name == "use_state" && !ends_with_window_cx {
                call.args.push(parse_quote!(window));
            }
            if !call.args.last().is_some_and(|a| is_bare_ident(a, "cx")) {
                call.args.push(parse_quote!(cx));
            }
            return;
        }

        if let Some(Expr::Closure(closure)) = call.args.last_mut() {
            if is_hook_path(&path_expr.path, CLOSURE_CX_FNS)
                || is_hook_path(&path_expr.path, ASYNC_CX_FNS)
            {
                inject_cx_param(closure);
            }
        }
    }

    fn visit_macro_mut(&mut self, mac: &mut Macro) {
        if mac
            .path
            .segments
            .last()
            .is_some_and(|s| s.ident == "signals")
        {
            if let Ok(names) = syn::parse2::<SignalNames>(mac.tokens.clone()) {
                if let Some(scope) = self.scopes.last_mut() {
                    for ident in names.idents {
                        scope.signals.insert(ident.to_string());
                    }
                }
            }
            mac.tokens = TokenStream2::new();
            return;
        }

        let mut tokens = mac.tokens.clone();
        self.rewrite_signal_calls(&mut tokens);
        mac.tokens = tokens;
    }
}

struct SignalNames {
    idents: Vec<syn::Ident>,
}

impl syn::parse::Parse for SignalNames {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut idents = Vec::new();
        while !input.is_empty() {
            idents.push(input.parse::<syn::Ident>()?);
            if input.is_empty() {
                break;
            }
            input.parse::<syn::Token![,]>()?;
        }
        Ok(Self { idents })
    }
}

/// Declares identifiers as zopra signal getters/setters for the enclosing
/// `#[component]`. Use this when a signal is created by code the macro
/// rewriter cannot see through — e.g. a `macro_rules!` expansion:
///
/// ```ignore
/// macro_rules! counter {
///     () => { let (count, set_count) = use_state(0, window, cx); };
/// }
/// counter!();
/// signals!(count, set_count); // must come AFTER the macro that binds them
/// ```
#[proc_macro]
pub fn signals(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

fn inject_cx_param(closure: &mut ExprClosure) {
    let already_has_cx = closure
        .inputs
        .iter()
        .any(|p| matches!(p, Pat::Ident(id) if id.ident == "cx"));
    if !already_has_cx {
        closure.inputs.push(syn::parse_quote!(cx));
    }
}

fn rebuild_group(group: &Group, stream: TokenStream2, delimiter: Delimiter) -> Group {
    let mut rebuilt = Group::new(delimiter, stream);
    rebuilt.set_span(group.span());
    rebuilt
}

/// Converts a function into a GPUI component.
///
/// # Example
/// ```ignore
/// #[component]
/// pub fn greet(name: String) {
///     div().child(format!("Hello, {name}"))
/// }
///
/// // direct:      greet(window, cx)
/// // builder:     GreetProps::new("hi".to_string()).render(window, cx)
/// // via rsx!:    rsx! { <Greet name={"hi".to_string()} /> }
/// ```
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let expanded = match generate_component(input) {
        Ok(tokens) => tokens,
        Err(err) => return err.to_compile_error().into(),
    };

    TokenStream::from(expanded)
}

/// Transforms a `#[component]` fn into:
///
/// - the original fn, rewritten to `fn name(window, cx) -> impl IntoElement`
///   with the cx-injection pass applied (the immediate-call form);
/// - a passive props builder `NameProps` with `new()` (one arg per prop, in
///   declaration order) and an inherent `render(window, cx)` that forwards to
///   the component fn with the ambient window/cx.
fn generate_component(mut input: ItemFn) -> syn::Result<TokenStream2> {
    use quote::quote;
    use syn::{FnArg, Pat, PatType};

    let prop_count = input.sig.inputs.len();
    let mut props: Vec<(syn::Ident, syn::Type)> = Vec::new();
    for arg in input.sig.inputs.iter().take(prop_count) {
        match arg {
            FnArg::Typed(PatType { pat, ty, .. }) => match &**pat {
                Pat::Ident(pat_ident) => props.push((pat_ident.ident.clone(), (**ty).clone())),
                _ => {
                    return Err(syn::Error::new_spanned(
                        pat,
                        "component props must be plain identifiers: `count: u32`, not destructuring patterns like `(a, b): (u32, u32)`",
                    ));
                }
            },
            FnArg::Receiver(receiver) => {
                return Err(syn::Error::new_spanned(
                    receiver,
                    "methods (self) are not supported in components",
                ));
            }
        }
    }

    input.sig.output = parse_quote!(-> impl gpui_kit::IntoElement + use<>);
    input
        .sig
        .inputs
        .push(parse_quote!(window: &mut gpui_kit::Window));
    input.sig.inputs.push(parse_quote!(cx: &mut gpui_kit::App));

    let mut rewriter = InjectCx::default();
    rewriter.push_scope();
    rewriter.visit_block_mut(&mut input.block);
    rewriter.pop_scope();

    let vis = &input.vis;
    let name = &input.sig.ident;
    // `greet_click` -> `GreetClickProps` so the rsx tag `<GreetClick>` resolves
    let mut props_str = String::with_capacity(name.to_string().len() + 5);
    for part in name.to_string().split('_') {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            props_str.extend(first.to_uppercase());
            props_str.push_str(chars.as_str());
        }
    }
    props_str.push_str("Props");
    let props_ident = syn::Ident::new(&props_str, name.span());

    let mut prop_fields = TokenStream2::new();
    let mut setters = TokenStream2::new();
    let mut prop_args = TokenStream2::new();
    let mut prop_extracts = TokenStream2::new();
    let mut prop_defaults = TokenStream2::new();
    for (prop, ty) in &props {
        prop_fields.extend(quote! { #prop: ::std::option::Option<#ty>, });
        setters.extend(quote! {
            #vis fn #prop(mut self, #prop: #ty) -> Self {
                self.#prop = ::std::option::Option::Some(#prop);
                self
            }
        });
        prop_args.extend(quote! { self.#prop, });
        prop_extracts.extend(quote! { #prop, });
        prop_defaults.extend(quote! { #prop: ::std::option::Option::None, });
    }
    let mut prop_unwraps = TokenStream2::new();
    for (prop, _) in &props {
        let err = format!(
            "missing required prop `{prop}` for component `{name}` (rsx tag `<{} ...>`)",
            name
        );
        prop_unwraps.extend(quote! {
            let #prop = #prop.expect(#err);
        });
    }

    let tag_alias = {
        let mut tag_str = String::new();
        for part in name.to_string().split('_') {
            let mut chars = part.chars();
            if let Some(first) = chars.next() {
                tag_str.extend(first.to_uppercase());
                tag_str.push_str(chars.as_str());
            }
        }
        syn::Ident::new(&tag_str, name.span())
    };

    Ok(quote! {
        #input

        #[doc = concat!("rsx! tag alias: `<", stringify!(#tag_alias), " prop={..} />` resolves to the props builder.")]
        #[allow(missing_docs, non_snake_case, non_camel_case_types)]
        #vis type #tag_alias = #props_ident;

        #[doc = "Props builder for the "]
        #[doc = concat!(stringify!(#name), " component (immediate-call model — no RenderOnce).")]
        #[allow(missing_docs, non_snake_case, non_camel_case_types)]
        #vis struct #props_ident {
            #prop_fields
        }

        impl #props_ident {
            #vis fn new() -> Self {
                Self { #prop_defaults }
            }

            #setters

            #[track_caller]
            #vis fn render(
                self,
                window: &mut gpui_kit::Window,
                cx: &mut gpui_kit::App,
            ) -> impl gpui_kit::IntoElement {
                let site = core::panic::Location::caller();
                self.render_at(gpui_kit::ElementId::CodeLocation(*site), window, cx)
            }

            #vis fn render_at(
                self,
                site: gpui_kit::ElementId,
                window: &mut gpui_kit::Window,
                cx: &mut gpui_kit::App,
            ) -> impl gpui_kit::IntoElement {
                let #props_ident { #prop_extracts } = self;
                #prop_unwraps
                window.with_id(site, |window| {
                    #name(#prop_extracts window, cx)
                })
            }
        }
    })
}
