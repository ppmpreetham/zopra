use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, Span, TokenStream as TokenStream2, TokenTree};
use quote::{ToTokens, format_ident, quote};
use std::collections::HashMap;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};
use syn::{
    Expr, ExprAsync, ExprCall, ExprClosure, ExprForLoop, ExprIf, ExprLoop, ExprMatch,
    ExprMethodCall, ExprWhile, FnArg, ItemFn, Local, Macro, Pat, PatType, Stmt, Token,
    parse_macro_input, parse_quote,
};

const HOOKS: &[&str] = &[
    "use_state",
    "use_signal",
    "use_resource",
    "use_window",
    "use_webview",
    "use_input",
    "use_editor",
    "use_event",
    "use_context",
    "try_use_context",
    "provide_context",
];

/// use_state(0) -> use_state(|| 0).
const LAZY_HOOKS: &[&str] = &["use_state", "use_signal"];
/// Signal methods that take `cx` as their only argument.
const SIGNAL_READS: &[&str] = &["get", "read", "try_read"];
/// Signal methods that take a value or closure, then `cx`.
const SIGNAL_WRITES: &[&str] = &["set", "update", "try_update", "update_untracked"];
/// GPUI events whose listeners take `(event, window, cx)`.
const EVENTS: &[&str] = &[
    "on_click",
    "on_mouse_down",
    "on_mouse_up",
    "on_mouse_move",
    "on_mouse_exit",
    "on_mouse_pressure",
    "on_pinch",
    "on_mouse_down_out",
    "on_mouse_up_out",
    "on_any_mouse_down",
    "on_any_mouse_up",
    "on_key_down",
    "on_key_up",
    "on_modifiers_changed",
    "on_hover",
    "on_aux_click",
    "on_scroll_wheel",
    "on_drag",
    "on_drag_move",
    "on_drop",
    "on_action",
    "on_boxed_action",
    "on_double_click",
    "on_a11y_action",
    "on_resize",
];

/// What the macro knows about a binding.
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    /// `Copy` signal: gets the `x()` / `x(v)` sugar and `cx` on its methods.
    Signal,
    /// `Copy` window launcher: `.open(..)` gets `window, cx`.
    Launcher,
    /// Refcounted handle (`Arc`, `Entity`, ...): cloned into closures that capture it.
    Handle,
    /// The `children` prop: `{children}` renders it in place.
    Children,
    Controller,
}

/// Marks a closure whose GPUI context parameter zopra knows: `#[zopra_cx(name)]`.
const CX_MARKER: &str = "zopra_cx";

fn is_context_type(ty: &syn::Type) -> bool {
    let ty = match ty {
        syn::Type::Reference(reference) => &*reference.elem,
        ty => ty,
    };
    matches!(ty, syn::Type::Path(path) if path.path.segments.last().is_some_and(|segment| {
        ["App", "Context", "AsyncApp", "AsyncWindowContext"].iter().any(|name| segment.ident == name)
    }))
}

/// A closure parameter that is a GPUI context: named `cx`, or typed `&mut App` / `&mut Context<_>` / ...
fn context_ident(pat: &Pat) -> Option<syn::Ident> {
    match pat {
        Pat::Ident(pat) if pat.ident == "cx" => Some(pat.ident.clone()),
        Pat::Type(PatType { pat, ty, .. }) => match &**pat {
            Pat::Ident(name) if name.ident == "cx" || is_context_type(ty) => Some(name.ident.clone()),
            _ => None,
        },
        _ => None,
    }
}

/// Names the parameter at `index` (renaming `_` and other patterns) and marks it as the context.
fn mark_context_param(closure: &mut ExprClosure, index: usize) {
    let hidden = format_ident!("__zopra_cx");
    let ident = match closure.inputs.get_mut(index) {
        Some(Pat::Ident(pat)) => pat.ident.clone(),
        Some(Pat::Type(PatType { pat, .. })) => match &mut **pat {
            Pat::Ident(pat) => pat.ident.clone(),
            other => {
                *other = parse_quote!(#hidden);
                hidden
            }
        },
        Some(other) => {
            *other = parse_quote!(#hidden);
            hidden
        }
        None => return,
    };
    closure.attrs.push(parse_quote!(#[zopra_cx(#ident)]));
}

fn type_kind(ty: &syn::Type) -> Option<Kind> {
    let syn::Type::Path(path) = ty else {
        return None;
    };
    match path.path.segments.last()?.ident.to_string().as_str() {
        "Signal" => Some(Kind::Signal),
        "WindowLauncher" => Some(Kind::Launcher),
        "Arc" | "Rc" | "Entity" | "WeakEntity" | "SharedString" => Some(Kind::Handle),
        _ => None,
    }
}

fn hook_kind(hook: &str) -> Option<Kind> {
    match hook {
        "use_state" | "use_signal" | "use_resource" => Some(Kind::Signal),
        "use_window" => Some(Kind::Launcher),
        "use_input" | "use_editor" => Some(Kind::Handle),
        hook if !HOOKS.contains(&hook) => Some(Kind::Controller),
        _ => None,
    }
}

/// `x`, `zopra::..::x` or `crate::hooks::x` -> `x`, if `x` is one of `names`.
fn zopra_fn(path: &syn::Path, names: &[&str]) -> Option<String> {
    let name = path.segments.last()?.ident.to_string();
    let first = &path.segments.first()?.ident;
    let ours = path.segments.len() == 1
        || first == "zopra"
        || (first == "crate" && path.segments.len() == 3 && path.segments[1].ident == "hooks");
    (ours && names.contains(&name.as_str())).then_some(name)
}

fn hook_name(path: &syn::Path) -> Option<String> {
    zopra_fn(path, HOOKS).or_else(|| {
        let name = path.get_ident()?.to_string();
        (name.starts_with("use_") && !HOOKS.contains(&name.as_str())).then_some(name)
    })
}

fn ident_of(expr: &Expr) -> Option<&syn::Ident> {
    match expr {
        Expr::Path(path) => path.path.get_ident(),
        _ => None,
    }
}

fn is_ident(expr: &Expr, name: &str) -> bool {
    ident_of(expr).is_some_and(|ident| ident == name)
}

fn pat_idents(pat: &Pat) -> Vec<syn::Ident> {
    struct Idents(Vec<syn::Ident>);
    impl Visit<'_> for Idents {
        fn visit_pat_ident(&mut self, node: &syn::PatIdent) {
            self.0.push(node.ident.clone());
            visit::visit_pat_ident(self, node);
        }
    }
    let mut idents = Idents(Vec::new());
    idents.visit_pat(pat);
    idents.0
}

fn to_snake(name: &str) -> String {
    name.chars().fold(String::new(), |mut out, c| {
        if c.is_ascii_uppercase() {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
        out
    })
}

fn to_pascal(name: &str) -> String {
    name.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + chars.as_str()
            })
        })
        .collect()
}

fn rebuild_group(group: &Group, stream: TokenStream2) -> Group {
    let mut rebuilt = Group::new(group.delimiter(), stream);
    rebuilt.set_span(group.span());
    rebuilt
}

/// `a, cx` / `cx` -> `a` / ``: drops a context the user passed explicitly.
fn strip_cx(args: TokenStream2, cx: &syn::Ident) -> TokenStream2 {
    let mut args = args.into_iter().collect::<Vec<_>>();
    if matches!(args.last(), Some(TokenTree::Ident(last)) if last == "cx" || last == cx) {
        args.pop();
        if matches!(args.last(), Some(TokenTree::Punct(p)) if p.as_char() == ',') {
            args.pop();
        }
    }
    args.into_iter().collect()
}

/// `a` -> `a, cx`, without doubling an explicit context.
fn with_cx(args: TokenStream2, cx: &syn::Ident) -> TokenStream2 {
    let args = strip_cx(args, cx);
    if args.is_empty() { quote!(#cx) } else { quote!(#args, #cx) }
}

fn is_cx(expr: &Expr, cx: &syn::Ident) -> bool {
    is_ident(expr, "cx") || ident_of(expr).is_some_and(|ident| ident == cx)
}

fn starts_closure(tokens: &TokenStream2) -> bool {
    match tokens.clone().into_iter().next() {
        Some(TokenTree::Punct(p)) => p.as_char() == '|',
        Some(TokenTree::Ident(ident)) => ident == "move" || ident == "async",
        _ => false,
    }
}

/// Rewrites a `#[component]` body: injects `window` / `cx`, adds the signal sugar
/// and copies refcounted handles into the closures that capture them.
#[derive(Default)]
struct Rewriter {
    /// innermost scope last; `None` marks a plain binding that shadows an outer one
    scopes: Vec<HashMap<String, Option<Kind>>>,
    /// the GPUI context in scope, innermost last, with the closure depth it belongs to
    contexts: Vec<(syn::Ident, usize)>,
    /// whether each enclosing closure is `move`, outermost first
    moves: Vec<bool>,
    cond_depth: usize,
    closure_depth: usize,
    block_depth: usize,
    error: Option<syn::Error>,
}

impl Rewriter {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            contexts: vec![(format_ident!("cx"), 0)],
            ..Self::default()
        }
    }

    /// The context to pass for `name`. A `move` closure can't borrow an outer context,
    /// so using one from inside it is reported here instead of as a lifetime error.
    fn cx(&mut self, name: &syn::Ident) -> syn::Ident {
        let (cx, depth) = self.contexts.last().cloned().expect("rewriter has a context");
        if self.error.is_none() && self.moves[depth..].iter().any(|is_move| *is_move) {
            let message = format!(
                "`{name}` needs a GPUI context, but this `move` closure has none; \
                 add one as a parameter (e.g. `|_, _, cx|`) or pass it explicitly (`{name}.get(cx)`)"
            );
            self.error = Some(syn::Error::new(name.span(), message));
        }
        cx
    }

    fn kind(&self, name: &str) -> Option<Kind> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name)).copied().flatten()
    }

    fn bind(&mut self, pat: &Pat, kind: Option<Kind>) {
        let scope = self.scopes.last_mut().expect("rewriter has a scope");
        for ident in pat_idents(pat) {
            scope.insert(ident.to_string(), kind);
        }
    }

    fn scoped(&mut self, f: impl FnOnce(&mut Self)) {
        self.scopes.push(HashMap::new());
        f(self);
        self.scopes.pop();
    }

    fn conditional(&mut self, f: impl FnOnce(&mut Self)) {
        self.cond_depth += 1;
        self.scoped(f);
        self.cond_depth -= 1;
    }

    fn require_top_level(&mut self, what: &str, span: Span) {
        if self.error.is_some() {
            return;
        }
        let place = if self.closure_depth > 0 {
            "not inside a closure or event handler; declare it first, then use its handle in the closure"
        } else if self.cond_depth > 0 {
            "unconditionally, not inside if/match/loop; declare it first, then use its value in the branch"
        } else if self.block_depth > 1 {
            "not inside a nested block"
        } else {
            return;
        };
        let message = format!("{what} must be called at the top level of a #[component] body, {place}");
        self.error = Some(syn::Error::new(span, message));
    }

    /// The kind a `let` binding gets from its initializer.
    fn init_kind(&self, expr: &Expr) -> Option<Kind> {
        match expr {
            Expr::Call(call) => {
                let Expr::Path(func) = &*call.func else {
                    return None;
                };
                if let Some(hook) = hook_name(&func.path) {
                    let explicit = call.args.iter().any(|arg| is_ident(arg, "window") || is_ident(arg, "cx"));
                    return if !HOOKS.contains(&hook.as_str()) && explicit { None } else { hook_kind(&hook) };
                }
                // `Arc::clone(&handle)`
                match call.args.first() {
                    Some(Expr::Reference(reference)) if func.path.segments.last()?.ident == "clone" => {
                        self.init_kind(&reference.expr)
                    }
                    _ => None,
                }
            }
            Expr::MethodCall(call) if call.method == "clone" => self.init_kind(&call.receiver),
            _ => self.kind(&ident_of(expr)?.to_string()),
        }
    }

    /// Makes `closure` own copies of the refcounted handles it captures.
    fn capture(&self, mut closure: ExprClosure) -> Expr {
        #[derive(Default)]
        struct Used(Vec<syn::Ident>);
        impl Visit<'_> for Used {
            fn visit_expr_path(&mut self, path: &syn::ExprPath) {
                if let Some(ident) = path.path.get_ident()
                    && !self.0.contains(ident)
                {
                    self.0.push(ident.clone());
                }
            }
            fn visit_macro(&mut self, mac: &Macro) {
                for token in mac.tokens.clone() {
                    if let TokenTree::Ident(ident) = token
                        && !self.0.contains(&ident)
                    {
                        self.0.push(ident);
                    }
                }
            }
        }

        closure.capture = Some(parse_quote!(move));
        let params = closure.inputs.iter().flat_map(pat_idents).collect::<Vec<_>>();
        let mut used = Used::default();
        used.visit_expr(&closure.body);
        let handles = used
            .0
            .into_iter()
            .filter(|ident| !params.contains(ident) && self.kind(&ident.to_string()) == Some(Kind::Handle))
            .collect::<Vec<_>>();
        if handles.is_empty() {
            Expr::Closure(closure)
        } else {
            parse_quote!({ #(let #handles = #handles.clone();)* #closure })
        }
    }

    /// `on_click={|| ..}` -> a `move` listener with GPUI's `(event, window, cx)` parameters.
    fn event_listener(&mut self, attr: &str, value: &Group) -> Option<Group> {
        let attr = to_snake(attr);
        if !attr.starts_with("on_") {
            return None;
        }
        let mut closure = match syn::parse2::<Expr>(value.stream()).ok()? {
            Expr::Closure(closure) => closure,
            Expr::Path(path) if self.kind(&path.path.get_ident()?.to_string()) == Some(Kind::Signal) => {
                parse_quote!(|__value| #path(__value))
            }
            _ => return None,
        };
        if EVENTS.contains(&attr.as_str()) {
            let params: [Pat; 3] = [parse_quote!(__event), parse_quote!(window), parse_quote!(cx)];
            for param in params.into_iter().skip(closure.inputs.len()) {
                closure.inputs.push(param);
            }
            mark_context_param(&mut closure, 2);
        }
        let mut listener = self.capture(closure);
        self.visit_expr_mut(&mut listener);
        Some(rebuild_group(value, quote!(#listener)))
    }

    /// `{expr}` / `{...expr}` inside markup: rewritten as real Rust, so shadowing is respected.
    /// Falls back to tokens for what isn't an expression, like `{for x in xs { <li/> }}`.
    fn rewrite_braced(&mut self, group: &Group) -> Group {
        let tokens = group.stream().into_iter().collect::<Vec<_>>();
        let spread = tokens.len() > 3
            && tokens[..3].iter().all(|token| matches!(token, TokenTree::Punct(p) if p.as_char() == '.'));
        let (prefix, body) = tokens.split_at(if spread { 3 } else { 0 });
        let prefix = prefix.iter().cloned().collect::<TokenStream2>();
        let body = body.iter().cloned().collect::<TokenStream2>();
        // `{children}` renders the children here
        if !spread
            && let Ok(children) = syn::parse2::<syn::Ident>(body.clone())
            && self.kind(&children.to_string()) == Some(Kind::Children)
        {
            let cx = self.cx(&children);
            return rebuild_group(group, quote!(...#children(window, #cx)));
        }
        let body = match syn::parse2::<Expr>(body.clone()) {
            Ok(mut expr) => {
                self.visit_expr_mut(&mut expr);
                quote!(#expr)
            }
            Err(_) => self.rewrite_tokens(body),
        };
        rebuild_group(group, quote!(#prefix #body))
    }

    /// Token-level rewrite for markup and other macro bodies that aren't plain expressions.
    fn rewrite_tokens(&mut self, tokens: TokenStream2) -> TokenStream2 {
        let tokens = tokens.into_iter().collect::<Vec<_>>();
        let mut out = TokenStream2::new();
        let mut i = 0;
        while i < tokens.len() {
            // `a.x` / `a::x` continue a path; `..x` (range) and `...x` (spread) start an expression
            let punct = |back: usize| match i.checked_sub(back).map(|at| &tokens[at]) {
                Some(TokenTree::Punct(p)) => Some(p.as_char()),
                _ => None,
            };
            let after_path = punct(1) == Some(':') || (punct(1) == Some('.') && punct(2) != Some('.'));
            match (&tokens[i], tokens.get(i + 1), tokens.get(i + 2), tokens.get(i + 3)) {
                // attr={value}
                (TokenTree::Ident(attr), Some(TokenTree::Punct(eq)), Some(TokenTree::Group(value)), _)
                    if eq.as_char() == '='
                        && eq.spacing() == proc_macro2::Spacing::Alone
                        && value.delimiter() == Delimiter::Brace
                        && !after_path =>
                {
                    let value = self.event_listener(&attr.to_string(), value).unwrap_or_else(|| {
                        match syn::parse2::<syn::Ident>(value.stream()) {
                            // a handle passed as a prop is copied, not moved
                            Ok(ident) if self.kind(&ident.to_string()) == Some(Kind::Handle) => {
                                rebuild_group(value, quote!(#ident.clone()))
                            }
                            _ => self.rewrite_braced(value),
                        }
                    });
                    out.extend([tokens[i].clone(), tokens[i + 1].clone(), TokenTree::Group(value)]);
                    i += 3;
                }
                // signal(..)
                (TokenTree::Ident(signal), Some(TokenTree::Group(args)), _, _)
                    if args.delimiter() == Delimiter::Parenthesis
                        && !after_path
                        && self.kind(&signal.to_string()) == Some(Kind::Signal) =>
                {
                    let cx = self.cx(signal);
                    let args = strip_cx(self.rewrite_tokens(args.stream()), &cx);
                    out.extend(if args.is_empty() {
                        quote!(#signal.get(#cx))
                    } else if starts_closure(&args) {
                        quote!(#signal.update(#args, #cx))
                    } else {
                        quote!(#signal.set(#args, #cx))
                    });
                    i += 2;
                }
                // signal.method(..)
                (
                    TokenTree::Ident(signal),
                    Some(TokenTree::Punct(dot)),
                    Some(TokenTree::Ident(method)),
                    Some(TokenTree::Group(args)),
                ) if dot.as_char() == '.'
                    && args.delimiter() == Delimiter::Parenthesis
                    && !after_path
                    && matches!(self.kind(&signal.to_string()), Some(Kind::Signal | Kind::Controller)) =>
                {
                    let method_name = method.to_string();
                    let mut args_tokens = self.rewrite_tokens(args.stream());
                    let takes_cx = if self.kind(&signal.to_string()) == Some(Kind::Controller) {
                        method_name != "clone"
                    } else {
                        (SIGNAL_READS.contains(&method_name.as_str()) && args_tokens.is_empty())
                            || (SIGNAL_WRITES.contains(&method_name.as_str()) && !args_tokens.is_empty())
                    };
                    if takes_cx {
                        let cx = self.cx(signal);
                        args_tokens = with_cx(args_tokens, &cx);
                    }
                    out.extend([tokens[i].clone(), tokens[i + 1].clone(), tokens[i + 2].clone()]);
                    out.extend([TokenTree::Group(rebuild_group(args, args_tokens))]);
                    i += 4;
                }
                (TokenTree::Group(group), ..) => {
                    let group = if group.delimiter() == Delimiter::Brace {
                        self.rewrite_braced(group)
                    } else {
                        rebuild_group(group, self.rewrite_tokens(group.stream()))
                    };
                    out.extend([TokenTree::Group(group)]);
                    i += 1;
                }
                (token, ..) => {
                    out.extend([token.clone()]);
                    i += 1;
                }
            }
        }
        out
    }

    /// Calls on a signal or the children:
    /// `signal()` -> `signal.get(cx)`, `signal(v)` -> `.set(v, cx)`, `signal(|v| ..)` -> `.update(.., cx)`,
    /// `children()` -> `children(window, cx)`
    fn signal_call(&mut self, call: &ExprCall) -> Option<Expr> {
        let Expr::Path(path) = &*call.func else {
            return None;
        };
        let name = path.path.get_ident()?;
        let kind = self.kind(&name.to_string())?;
        let cx = self.cx(name);
        let mut args = call.args.iter().collect::<Vec<_>>();
        if args.last().is_some_and(|arg| is_cx(arg, &cx)) {
            args.pop();
        }
        Some(match (kind, args.as_slice()) {
            (Kind::Children, []) => parse_quote!(#path(window, #cx)),
            (Kind::Signal, []) => parse_quote!(#path.get(#cx)),
            (Kind::Signal, [f @ Expr::Closure(_)]) => parse_quote!(#path.update(#f, #cx)),
            (Kind::Signal, [value]) => parse_quote!(#path.set(#value, #cx)),
            _ => return None,
        })
    }

    /// Before the arguments are visited: closures whose context zopra supplies get marked.
    fn prepare_hook_call(&self, call: &mut ExprCall, hook: &str) {
        match hook {
            "use_event" => {
                if let Some(Expr::Closure(handler)) = call.args.last_mut() {
                    if handler.inputs.len() < 2 {
                        handler.inputs.push(parse_quote!(cx));
                    }
                    mark_context_param(handler, 1);
                    let handler = self.capture(handler.clone());
                    *call.args.last_mut().unwrap() = handler;
                }
            }
            "use_window" => {
                if let Some(Expr::Closure(options)) = call.args.first_mut() {
                    options.capture = Some(parse_quote!(move));
                }
            }
            _ => {}
        }
    }

    fn hook_call(&mut self, call: &mut ExprCall, hook: &str) {
        self.require_top_level("zopra hooks", call.func.span());

        if LAZY_HOOKS.contains(&hook)
            && call.args.len() == 1
            && !matches!(call.args[0], Expr::Closure(_))
        {
            let value = &call.args[0];
            call.args[0] = match value {
                Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(lit), .. }) => {
                    parse_quote!(|| ::gpui_kit::SharedString::new_static(#lit))
                }
                value => parse_quote!(|| #value),
            };
        }

        // `hook(.., cx)` / `hook(.., window, cx)` -> `hook(.., window, cx)`
        while call.args.last().is_some_and(|arg| is_ident(arg, "cx") || is_ident(arg, "window")) {
            call.args.pop();
        }
        call.args.push(parse_quote!(window));
        call.args.push(parse_quote!(cx));
    }

    /// Before the arguments are visited: `launcher.open(|| ..)` gets `window, cx` parameters.
    fn prepare_method_call(&self, call: &mut ExprMethodCall) {
        let Some(receiver) = ident_of(&call.receiver) else {
            return;
        };
        if self.kind(&receiver.to_string()) == Some(Kind::Launcher)
            && call.method == "open"
            && let Some(Expr::Closure(render)) = call.args.first_mut()
        {
            if render.inputs.is_empty() {
                render.inputs.push(parse_quote!(window));
                render.inputs.push(parse_quote!(cx));
            }
            render.capture = Some(parse_quote!(move));
            mark_context_param(render, 1);
        }
    }

    fn method_call(&mut self, call: &mut ExprMethodCall) {
        let Some(receiver) = ident_of(&call.receiver).cloned() else {
            return;
        };
        let method = call.method.to_string();
        let takes_cx = match self.kind(&receiver.to_string()) {
            Some(Kind::Signal) => {
                (SIGNAL_READS.contains(&method.as_str()) && call.args.is_empty())
                    || (SIGNAL_WRITES.contains(&method.as_str()) && call.args.len() == 1)
            }
            Some(Kind::Launcher) => method == "open" && call.args.len() == 1,
            Some(Kind::Controller) => {
                method != "clone" && !call.args.last().is_some_and(|arg| is_ident(arg, "cx"))
            }
            _ => false,
        };
        if takes_cx {
            let cx = self.cx(&receiver);
            call.args.push(parse_quote!(#cx));
        }
    }

    /// `use_effect!(|| .., [deps])` -> `use_effect!(move |_window, cx| .., [deps], cx, window, __zopra_component)`
    fn effect_macro(&mut self, mac: &mut Macro) {
        self.require_top_level("use_effect!", mac.path.span());
        let Ok(args) = mac.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated) else {
            return; // the declarative macro reports the syntax error
        };
        let mut args = args.into_iter();
        let (Some(Expr::Closure(mut effect)), Some(mut deps), None) = (args.next(), args.next(), args.next()) else {
            return;
        };
        let params: [Pat; 2] = [parse_quote!(_window), parse_quote!(cx)];
        effect.inputs = params.into_iter().collect();
        mark_context_param(&mut effect, 1);
        let mut effect = self.capture(effect);
        self.visit_expr_mut(&mut effect);
        self.visit_expr_mut(&mut deps);
        // `__zopra_component` tells the declarative macro it was rewritten by a component
        mac.tokens = quote!(#effect, #deps, cx, window, __zopra_component);
    }
}

impl VisitMut for Rewriter {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        visit_mut::visit_expr_mut(self, expr);
        if let Expr::Call(call) = expr
            && let Some(rewritten) = self.signal_call(call)
        {
            *expr = rewritten;
        }
    }

    fn visit_expr_call_mut(&mut self, call: &mut ExprCall) {
        let hook = match &*call.func {
            Expr::Path(func) => hook_name(&func.path),
            _ => None,
        }
        .filter(|hook| {
            HOOKS.contains(&hook.as_str())
                || !call.args.iter().any(|arg| is_ident(arg, "window") || is_ident(arg, "cx"))
        });
        if let Some(hook) = &hook {
            self.prepare_hook_call(call, hook);
        }
        visit_mut::visit_expr_call_mut(self, call);
        if let Some(hook) = &hook {
            self.hook_call(call, hook);
        }
    }

    fn visit_expr_method_call_mut(&mut self, call: &mut ExprMethodCall) {
        self.prepare_method_call(call);
        visit_mut::visit_expr_method_call_mut(self, call);
        self.method_call(call);
    }

    fn visit_macro_mut(&mut self, mac: &mut Macro) {
        if mac.path.is_ident("macro_rules") {
            return;
        }
        if mac.path.is_ident("signals") {
            let names = mac
                .parse_body_with(Punctuated::<syn::Ident, Token![,]>::parse_terminated)
                .unwrap_or_default();
            let scope = self.scopes.last_mut().expect("rewriter has a scope");
            for name in names {
                scope.insert(name.to_string(), Some(Kind::Signal));
            }
            mac.tokens = TokenStream2::new();
        } else if zopra_fn(&mac.path, &["use_effect"]).is_some() {
            self.effect_macro(mac);
        } else if let Ok(mut args) = mac.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated) {
            // `format!("..", a, b)`, `vec![..]`, ...: rewritten as real Rust
            for arg in &mut args {
                self.visit_expr_mut(arg);
            }
            mac.tokens = quote!(#args);
        } else {
            mac.tokens = self.rewrite_tokens(mac.tokens.clone());
        }
    }

    fn visit_local_mut(&mut self, local: &mut Local) {
        let kind = match &local.pat {
            Pat::Type(PatType { ty, .. }) => type_kind(ty),
            _ => None,
        }
        .or_else(|| self.init_kind(&local.init.as_ref()?.expr));
        visit_mut::visit_local_mut(self, local);
        self.bind(&local.pat, kind);
    }

    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        self.block_depth += 1;
        self.scoped(|this| visit_mut::visit_block_mut(this, block));
        self.block_depth -= 1;
    }

    fn visit_expr_async_mut(&mut self, expr: &mut ExprAsync) {
        self.closure_depth += 1;
        self.moves.push(expr.capture.is_some());
        visit_mut::visit_expr_async_mut(self, expr);
        self.moves.pop();
        self.closure_depth -= 1;
    }

    fn visit_expr_closure_mut(&mut self, closure: &mut ExprClosure) {
        // the context: one zopra marked, else a parameter named `cx` or typed as a GPUI context
        let marked = closure
            .attrs
            .iter()
            .position(|attr| attr.path().is_ident(CX_MARKER))
            .map(|at| closure.attrs.remove(at).parse_args::<syn::Ident>().expect("zopra marks an ident"));
        let context = marked.or_else(|| closure.inputs.iter().find_map(context_ident));

        self.closure_depth += 1;
        self.moves.push(closure.capture.is_some());
        if let Some(context) = &context {
            self.contexts.push((context.clone(), self.closure_depth));
        }
        self.scoped(|this| {
            for input in &closure.inputs {
                this.bind(input, None);
            }
            visit_mut::visit_expr_closure_mut(this, closure);
        });
        if context.is_some() {
            self.contexts.pop();
        }
        self.moves.pop();
        self.closure_depth -= 1;
    }

    fn visit_expr_match_mut(&mut self, expr: &mut ExprMatch) {
        self.visit_expr_mut(&mut expr.expr);
        for arm in &mut expr.arms {
            self.conditional(|this| {
                this.bind(&arm.pat, None);
                if let Pat::Guard(pat) = &mut arm.pat {
                    this.visit_expr_mut(&mut pat.guard);
                }
                this.visit_expr_mut(&mut arm.body);
            });
        }
    }

    fn visit_expr_if_mut(&mut self, expr: &mut ExprIf) {
        self.visit_expr_mut(&mut expr.cond);
        self.conditional(|this| {
            if let Expr::Let(expr_let) = &*expr.cond {
                this.bind(&expr_let.pat, None);
            }
            this.visit_block_mut(&mut expr.then_branch);
        });
        if let Some((_, else_branch)) = &mut expr.else_branch {
            self.visit_expr_mut(else_branch);
        }
    }

    fn visit_expr_while_mut(&mut self, expr: &mut ExprWhile) {
        self.visit_expr_mut(&mut expr.cond);
        self.conditional(|this| {
            if let Expr::Let(expr_let) = &*expr.cond {
                this.bind(&expr_let.pat, None);
            }
            this.visit_block_mut(&mut expr.body);
        });
    }

    fn visit_expr_for_loop_mut(&mut self, expr: &mut ExprForLoop) {
        self.visit_expr_mut(&mut expr.expr);
        self.conditional(|this| {
            this.bind(&expr.pat, None);
            this.visit_block_mut(&mut expr.body);
        });
    }

    fn visit_expr_loop_mut(&mut self, expr: &mut ExprLoop) {
        self.conditional(|this| this.visit_block_mut(&mut expr.body));
    }
}

/// Declares signal names the macro can't see, e.g. ones bound inside a `macro_rules!` expansion.
#[proc_macro]
pub fn signals(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    generate_component(input).unwrap_or_else(syn::Error::into_compile_error).into()
}

fn option_inner(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(tp) = ty else {
        return None;
    };
    let seg = tp.path.segments.last()?;
    if seg.ident != "Option" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    match args.args.first()? {
        syn::GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

/// The elided `&T` / `Option<&T>` reference in a prop type, if any.
fn elided_prop_ref(ty: &mut syn::Type) -> Option<&mut syn::TypeReference> {
    let reference = match ty {
        syn::Type::Reference(reference) => reference,
        syn::Type::Path(path) => {
            let segment = path.path.segments.last_mut()?;
            if segment.ident != "Option" {
                return None;
            }
            let syn::PathArguments::AngleBracketed(args) = &mut segment.arguments else {
                return None;
            };
            match args.args.first_mut()? {
                syn::GenericArgument::Type(syn::Type::Reference(reference)) => reference,
                _ => return None,
            }
        }
        _ => return None,
    };
    reference.lifetime.is_none().then_some(reference)
}

/// `Children` / `Children<T>` -> the child type `T` (`AnyElement` by default).
fn children_item(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != "Children" {
        return None;
    }
    match &segment.arguments {
        syn::PathArguments::None => Some(parse_quote!(::zopra::gpui_kit::AnyElement)),
        syn::PathArguments::AngleBracketed(args) => match args.args.first()? {
            syn::GenericArgument::Type(item) => Some(item.clone()),
            _ => None,
        },
        syn::PathArguments::Parenthesized(_) => None,
    }
}


/// A generic parameter as an argument: `T` / `'a` / `N`.
fn generic_arg(param: &syn::GenericParam) -> TokenStream2 {
    match param {
        syn::GenericParam::Type(param) => param.ident.to_token_stream(),
        syn::GenericParam::Lifetime(param) => param.lifetime.to_token_stream(),
        syn::GenericParam::Const(param) => param.ident.to_token_stream(),
    }
}

/// A generic parameter without bounds or defaults, for a type alias.
fn plain_param(param: &syn::GenericParam) -> TokenStream2 {
    match param {
        syn::GenericParam::Const(param) => {
            let (ident, ty) = (&param.ident, &param.ty);
            quote!(const #ident: #ty)
        }
        param => generic_arg(param),
    }
}

enum Role {
    /// must be set before the component renders; tracked in the builder's type
    Required,
    /// `Option<T>`: the setter takes a `T`
    Optional(syn::Type),
    /// `Option<F>` with `F: Fn(..)`: tracked like children, so an unset callback costs nothing
    Callback(Callback),
    /// `children: Children<T>`: the content between the tags
    Children(syn::Type),
    Key,
}

#[derive(Clone)]
struct Callback {
    /// the component's generic parameter, e.g. `F`
    param: syn::Ident,
    /// its bounds, e.g. `Fn(&ClickEvent, &mut Window, &mut App) + 'static`
    bounds: Vec<syn::TypeParamBound>,
    /// the type of an unset callback: a `fn` pointer with the same signature
    unset: syn::Type,
}

/// Generic parameters bounded by `Fn` / `FnMut` / `FnOnce`, which can back an optional callback prop.
fn callback_params(generics: &syn::Generics) -> HashMap<String, Callback> {
    let mut bounds = HashMap::<syn::Ident, Vec<syn::TypeParamBound>>::new();
    for param in generics.type_params() {
        bounds.entry(param.ident.clone()).or_default().extend(param.bounds.iter().cloned());
    }
    for predicate in generics.where_clause.iter().flat_map(|clause| &clause.predicates) {
        if let syn::WherePredicate::Type(predicate) = predicate
            && let syn::Type::Path(path) = &predicate.bounded_ty
            && let Some(ident) = path.path.get_ident()
            && let Some(param) = bounds.get_mut(ident)
        {
            param.extend(predicate.bounds.iter().cloned());
        }
    }
    bounds
        .into_iter()
        .filter_map(|(param, bounds)| {
            let unset = bounds.iter().find_map(|bound| {
                let syn::TypeParamBound::Trait(bound) = bound else {
                    return None;
                };
                let segment = bound.path.segments.last()?;
                let syn::PathArguments::Parenthesized(signature) = &segment.arguments else {
                    return None;
                };
                ["Fn", "FnMut", "FnOnce"].iter().any(|name| segment.ident == name).then(|| {
                    let (inputs, output) = (&signature.inputs, &signature.output);
                    parse_quote!(fn(#inputs) #output)
                })
            })?;
            Some((param.to_string(), Callback { param, bounds, unset }))
        })
        .collect()
}

/// A setter's parameter type and how it stores the value: strings and elements convert.
fn setter_param(ty: &syn::Type, field: &syn::Ident, into: bool) -> (TokenStream2, TokenStream2) {
    if into {
        return (quote!(impl ::std::convert::Into<#ty>), quote!(#field.into()));
    }
    let last = match ty {
        syn::Type::Path(path) => path.path.segments.last().map(|segment| segment.ident.to_string()),
        _ => None,
    };
    match last.as_deref() {
        Some("String" | "SharedString" | "ElementId") => (quote!(impl ::std::convert::Into<#ty>), quote!(#field.into())),
        Some("AnyElement") => (
            quote!(impl ::zopra::gpui_kit::IntoElement),
            quote!(::zopra::gpui_kit::IntoElement::into_any_element(#field)),
        ),
        _ => (ty.to_token_stream(), field.to_token_stream()),
    }
}

struct Prop {
    /// the builder field and setter (`__zopra_propN` for a pattern parameter)
    name: syn::Ident,
    pat: Option<Pat>,
    ty: syn::Type,
    role: Role,
    into: bool,
}

#[allow(clippy::too_many_lines)]
fn generate_component(mut input: ItemFn) -> syn::Result<TokenStream2> {
    let callbacks = callback_params(&input.sig.generics);
    let mut props = Vec::new();
    let mut borrows_props = false;
    let mut children_bound = None;
    for arg in &mut input.sig.inputs {
        let FnArg::Typed(PatType { pat, ty, attrs, .. }) = arg else {
            return Err(syn::Error::new_spanned(arg, "methods (self) are not supported in components"));
        };
        let into = attrs.iter().any(|attr| attr.path().is_ident("into"));
        attrs.retain(|attr| !attr.path().is_ident("into"));
        // props borrowed by the builder share one named lifetime
        if let Some(reference) = elided_prop_ref(ty) {
            reference.lifetime = Some(parse_quote!('__zopra_p));
            borrows_props = true;
        }
        let (name, pat) = match &**pat {
            Pat::Ident(pat) => (pat.ident.clone(), None),
            other => (format_ident!("__zopra_prop{}", props.len(), span = other.span()), Some(other.clone())),
        };
        let is_key = name == "key"
            && option_inner(ty).is_some_and(|inner| {
                matches!(inner, syn::Type::Path(path) if path.path.segments.last().is_some_and(|s| s.ident == "ElementId"))
            });
        if !is_key && ["new", "key", "render", "render_at"].iter().any(|reserved| name == reserved) {
            let message = format!("`{name}` can't be a prop name: the props builder uses it; rename the prop");
            return Err(syn::Error::new(name.span(), message));
        }
        let role = match children_item(ty) {
            _ if is_key => Role::Key,
            Some(item) if name == "children" => {
                // the content is a generic closure, so it is never boxed; its bound is the
                // `Children` trait exactly as the component named it
                children_bound = Some((**ty).clone());
                **ty = parse_quote!(__ZopraChildren);
                Role::Children(item)
            }
            _ => {
                let callback = match option_inner(ty) {
                    Some(syn::Type::Path(inner)) => {
                        inner.path.get_ident().and_then(|ident| callbacks.get(&ident.to_string()))
                    }
                    _ => None,
                };
                match (callback, option_inner(ty)) {
                    (Some(callback), _) => Role::Callback(callback.clone()),
                    (None, Some(inner)) => Role::Optional(inner.clone()),
                    (None, None) => Role::Required,
                }
            }
        };
        props.push(Prop { name, pat, ty: (**ty).clone(), role, into });
    }
    if borrows_props {
        input.sig.generics.params.insert(0, parse_quote!('__zopra_p));
    }
    // the builder takes the component's generics, minus the children closure and the
    // callback types, which it tracks itself
    let mut generics = input.sig.generics.clone();
    let tracked_params = props
        .iter()
        .filter_map(|prop| match &prop.role {
            Role::Callback(callback) => Some(callback.param.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    generics.params = std::mem::take(&mut generics.params)
        .into_iter()
        .filter(|param| !matches!(param, syn::GenericParam::Type(param) if tracked_params.contains(&param.ident)))
        .collect();
    if let Some(clause) = &mut generics.where_clause {
        clause.predicates = std::mem::take(&mut clause.predicates)
            .into_iter()
            .filter(|predicate| {
                !matches!(predicate, syn::WherePredicate::Type(predicate)
                    if matches!(&predicate.bounded_ty, syn::Type::Path(path)
                        if path.path.get_ident().is_some_and(|ident| tracked_params.contains(ident))))
            })
            .collect();
    }
    if let Some(bound) = &children_bound {
        input.sig.generics.params.push(parse_quote!(__ZopraChildren: #bound));
    }

    // a component without a return type renders an element; one with a type (e.g. a `Tab`) returns it
    let returns_element = matches!(input.sig.output, syn::ReturnType::Default);
    if returns_element {
        let params = input.sig.generics.params.iter().map(generic_arg);
        input.sig.output = parse_quote!(-> impl ::zopra::gpui_kit::IntoElement + use<#(#params),*>);
    }
    let (output, finish) = match &input.sig.output {
        syn::ReturnType::Type(_, ty) if !returns_element => (ty.to_token_stream(), quote!(element)),
        _ => (
            quote!(::zopra::gpui_kit::AnyElement),
            quote!(::zopra::gpui_kit::IntoElement::into_any_element(element)),
        ),
    };

    let mut rewriter = Rewriter::new();
    for prop in &props {
        let kind = match prop.role {
            Role::Children(_) => Some(Kind::Children),
            _ => type_kind(&prop.ty),
        };
        match &prop.pat {
            Some(pat) => rewriter.bind(pat, kind),
            None => {
                rewriter.scopes[0].insert(prop.name.to_string(), kind);
            }
        }
    }
    input.sig.inputs.push(parse_quote!(window: &mut ::zopra::gpui_kit::Window));
    input.sig.inputs.push(parse_quote!(cx: &mut ::zopra::gpui_kit::App));
    rewriter.visit_block_mut(&mut input.block);
    if let Some(err) = rewriter.error {
        return Err(err);
    }

    // a component may return a fragment: `view! { <>..</> }` -> `view! { <div>..</div> }`
    if let Some(Stmt::Expr(Expr::Macro(syn::ExprMacro { mac, .. }), None) | Stmt::Macro(syn::StmtMacro { mac, .. })) =
        input.block.stmts.last_mut()
        && mac.path.is_ident("view")
    {
        let mut tokens = mac.tokens.clone().into_iter();
        if matches!(
            (tokens.next(), tokens.next()),
            (Some(TokenTree::Punct(a)), Some(TokenTree::Punct(b))) if a.as_char() == '<' && b.as_char() == '>'
        ) {
            let inner = mac.tokens.clone();
            mac.tokens = quote!(<div> #inner </div>);
        }
    }

    let vis = &input.vis;
    let name = &input.sig.ident;
    let tag = to_pascal(&name.to_string());
    let props_ident = format_ident!("{}Props", tag, span = name.span());
    let tag_alias = format_ident!("{}", tag, span = name.span());
    let window_ty = quote!(::zopra::gpui_kit::Window);
    let app_ty = quote!(::zopra::gpui_kit::App);

    // the builder's type tracks every required prop and the children: `Missing` until set
    let params = generics.params.iter().collect::<Vec<_>>();
    let args = params.iter().map(|param| generic_arg(param)).collect::<Vec<_>>();
    let plain = params.iter().map(|param| plain_param(param)).collect::<Vec<_>>();
    let where_clause = &generics.where_clause;
    let marker = params.iter().filter_map(|param| match param {
        syn::GenericParam::Type(param) => Some(param.ident.to_token_stream()),
        syn::GenericParam::Lifetime(param) => {
            let lifetime = &param.lifetime;
            Some(quote!(&#lifetime ()))
        }
        syn::GenericParam::Const(_) => None,
    });
    let tracked = props
        .iter()
        .filter(|prop| !matches!(prop.role, Role::Optional(_) | Role::Key))
        .map(|prop| (prop, format_ident!("__Zopra_{}", prop.name)))
        .collect::<Vec<_>>();
    let state_of = |field: &syn::Ident| {
        tracked.iter().find(|(prop, _)| prop.name == *field).map(|(_, state)| state.clone())
    };
    let states = tracked.iter().map(|(_, state)| state).collect::<Vec<_>>();
    let unset = tracked
        .iter()
        .map(|(prop, _)| match &prop.role {
            Role::Children(item) => {
                quote!(::zopra::__private::Set<fn(&mut #window_ty, &mut #app_ty) -> ::std::vec::Vec<#item>>)
            }
            Role::Callback(callback) => {
                let unset = &callback.unset;
                quote!(::std::option::Option<#unset>)
            }
            _ => quote!(::zopra::__private::Missing),
        })
        .collect::<Vec<_>>();

    let fields = props.iter().filter(|prop| !matches!(prop.role, Role::Key)).map(|prop| &prop.name).collect::<Vec<_>>();
    let all_names = props.iter().map(|prop| &prop.name).collect::<Vec<_>>();
    let call_args = props
        .iter()
        .map(|prop| match prop.role {
            Role::Key => quote!(__zopra_given),
            _ => prop.name.to_token_stream(),
        })
        .collect::<Vec<_>>();
    let field_types = props
        .iter()
        .filter(|prop| !matches!(prop.role, Role::Key))
        .map(|prop| match &prop.role {
            Role::Optional(inner) => quote!(::std::option::Option<#inner>),
            _ => state_of(&prop.name).to_token_stream(),
        })
        .collect::<Vec<_>>();
    let initial = props.iter().filter(|prop| !matches!(prop.role, Role::Key)).map(|prop| match &prop.role {
        Role::Required => quote!(::zopra::__private::Missing),
        Role::Optional(_) | Role::Callback(_) | Role::Key => quote!(::std::option::Option::None),
        Role::Children(item) => quote! {
            ::zopra::__private::Set(
                ::zopra::__private::no_children::<#item> as fn(&mut #window_ty, &mut #app_ty) -> ::std::vec::Vec<#item>
            )
        },
    });

    // setters: optional ones keep the type, tracked ones move the builder to a `Set` state
    let mut setters = TokenStream2::new();
    let mut required_traits = TokenStream2::new();
    let mut bounds = Vec::new();
    let mut take_values = TokenStream2::new();
    for prop in &props {
        let field = &prop.name;
        let ty = &prop.ty;
        let set_state = |value: TokenStream2| {
            let states = tracked.iter().map(|(other, state)| {
                if other.name == prop.name { value.clone() } else { state.to_token_stream() }
            });
            quote!(#props_ident<#(#args,)* #(#states,)*>)
        };
        let others = fields.iter().filter(|other| **other != field).collect::<Vec<_>>();
        match &prop.role {
            Role::Optional(inner) => {
                let (param, value) = setter_param(inner, field, prop.into);
                setters.extend(quote! {
                    #vis fn #field(mut self, #field: #param) -> Self {
                        self.#field = ::std::option::Option::Some(#value);
                        self
                    }
                });
            }
            Role::Callback(callback) => {
                let bounds_of = &callback.bounds;
                let next = set_state(quote!(::std::option::Option<__ZopraCallback>));
                setters.extend(quote! {
                    #vis fn #field<__ZopraCallback: #(#bounds_of)+*>(self, #field: __ZopraCallback) -> #next {
                        #props_ident {
                            __zopra_key: self.__zopra_key,
                            __zopra_marker: ::std::marker::PhantomData,
                            #(#others: self.#others,)*
                            #field: ::std::option::Option::Some(#field),
                        }
                    }
                });
                let state = state_of(field);
                bounds.push(quote!(#state: ::zopra::__private::Optional));
                bounds.push(quote!(<#state as ::zopra::__private::Optional>::Inner: #(#bounds_of)+*));
                take_values.extend(quote!(let #field = ::zopra::__private::Optional::into_option(#field);));
            }
            Role::Required => {
                let (param, value) = setter_param(ty, field, prop.into);
                let next = set_state(quote!(::zopra::__private::Set<#ty>));
                setters.extend(quote! {
                    #vis fn #field(self, #field: #param) -> #next {
                        #props_ident {
                            __zopra_key: self.__zopra_key,
                            __zopra_marker: ::std::marker::PhantomData,
                            #(#others: self.#others,)*
                            #field: ::zopra::__private::Set(#value),
                        }
                    }
                });

                // a missing required prop fails `render_at`'s bound with this message
                let required = format_ident!("__{}Requires_{}", props_ident, field);
                let message = format!("`<{tag}>` is missing the required prop `{field}`");
                let label = format!("add `{field}={{{{..}}}}` to this tag");
                required_traits.extend(quote! {
                    #[doc(hidden)]
                    #[allow(non_camel_case_types)]
                    #[diagnostic::on_unimplemented(message = #message, label = #label)]
                    #vis trait #required {
                        type Value;
                        fn value(self) -> Self::Value;
                    }

                    impl<V> #required for ::zopra::__private::Set<V> {
                        type Value = V;
                        fn value(self) -> V {
                            self.0
                        }
                    }
                });
                let state = state_of(field);
                bounds.push(quote!(#state: #required<Value = #ty>));
                take_values.extend(quote!(let #field = #required::value(#field);));
            }
            Role::Children(item) => {
                let next = set_state(quote!(::zopra::__private::Set<__ZopraContent>));
                setters.extend(quote! {
                    #vis fn children<__ZopraContent: ::zopra::Children<#item>>(self, children: __ZopraContent) -> #next {
                        #props_ident {
                            __zopra_key: self.__zopra_key,
                            __zopra_marker: ::std::marker::PhantomData,
                            #(#others: self.#others,)*
                            children: ::zopra::__private::Set(children),
                        }
                    }
                });
                let state = state_of(field);
                bounds.push(quote!(#state: ::zopra::__private::Filled));
                bounds.push(quote!(<#state as ::zopra::__private::Filled>::Value: ::zopra::Children<#item>));
                take_values.extend(quote!(let children = ::zopra::__private::Filled::value(children);));
            }
            Role::Key => {}
        }
    }
    let impl_where = generics.where_clause.as_ref().map(|clause| &clause.predicates).into_iter().flatten().collect::<Vec<_>>();
    let clone_bounds = field_types.iter().map(|ty| quote!(for<'__zopra_b> #ty: ::std::clone::Clone));
    let eq_bounds = field_types.iter().map(|ty| quote!(for<'__zopra_b> #ty: ::std::cmp::PartialEq));
    let component_impl = returns_element.then(|| {
        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#params,)* #(#states,)*> ::zopra::Component for #props_ident<#(#args,)* #(#states,)*>
            where
                #(#impl_where,)*
                #(#bounds,)*
            {
                fn render_component(self, window: &mut #window_ty, cx: &mut #app_ty) -> ::zopra::gpui_kit::AnyElement {
                    self.render_at(::zopra::gpui_kit::ElementId::Name(::std::stringify!(#props_ident).into()), window, cx)
                }
            }
        }
    });
    let key_method = (!fields.iter().any(|field| *field == "key")).then(|| {
        quote! {
            /// Tells instances rendered from the same tag apart, e.g. in a loop.
            /// Without it, they are told apart by render order.
            #vis fn key(mut self, key: impl ::std::convert::Into<::zopra::gpui_kit::ElementId>) -> Self {
                self.__zopra_key = ::std::option::Option::Some(key.into());
                self
            }
        }
    });

    // `name` opens a render scope around `__zopra_name`, which holds the component body
    let inner_name = format_ident!("__zopra_{}", name);
    let mut inner = input.clone();
    inner.sig.ident = inner_name.clone();
    inner.vis = syn::Visibility::Inherited;
    inner.attrs.retain(|attr| !attr.path().is_ident("doc"));
    let mut outer = input.sig.clone();
    outer.inputs = props
        .iter()
        .map(|prop| -> FnArg {
            let (name, ty) = (&prop.name, &prop.ty);
            parse_quote!(#name: #ty)
        })
        .chain([parse_quote!(window: &mut #window_ty), parse_quote!(cx: &mut #app_ty)])
        .collect();
    let docs = input.attrs.iter().filter(|attr| attr.path().is_ident("doc"));
    let props_doc = fields.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");

    Ok(quote! {
        #(#docs)*
        #[allow(non_snake_case, clippy::too_many_arguments)]
        #vis #outer {
            ::zopra::__private::enter(cx);
            let element = #inner_name(#(#all_names,)* window, cx);
            ::zopra::__private::exit(cx);
            element
        }

        #[allow(non_snake_case, unused_variables, clippy::too_many_arguments)]
        #[inline]
        #inner

        #required_traits

        #[doc = concat!("view! tag alias: `<", stringify!(#tag_alias), " prop={..} />` resolves to the props builder.")]
        #[allow(missing_docs, non_snake_case, non_camel_case_types)]
        #vis type #tag_alias<#(#plain),*> = #props_ident<#(#args,)* #(#unset,)*>;

        #[doc = concat!("Props builder for the ", stringify!(#name), " component. Props: ", #props_doc)]
        #[allow(missing_docs, non_snake_case, non_camel_case_types)]
        #vis struct #props_ident<#(#params,)* #(#states,)*> #where_clause {
            __zopra_key: ::std::option::Option<::zopra::gpui_kit::ElementId>,
            __zopra_marker: ::std::marker::PhantomData<fn() -> (#(#marker,)*)>,
            #(#fields: #field_types,)*
        }

        #[allow(non_camel_case_types)]
        impl<#(#params),*> #props_ident<#(#args,)* #(#unset,)*> #where_clause {
            #vis fn new() -> Self {
                Self {
                    __zopra_key: ::std::option::Option::None,
                    __zopra_marker: ::std::marker::PhantomData,
                    #(#fields: #initial,)*
                }
            }
        }

        #[allow(non_camel_case_types)]
        impl<#(#params,)* #(#states,)*> #props_ident<#(#args,)* #(#states,)*> #where_clause {
            #key_method
            #setters

            #[track_caller]
            #vis fn render(self, window: &mut #window_ty, cx: &mut #app_ty) -> #output
            where
                #(#bounds,)*
            {
                let site = core::panic::Location::caller();
                self.render_at(::zopra::gpui_kit::ElementId::CodeLocation(*site), window, cx)
            }

            #vis fn render_at(self, site: ::zopra::gpui_kit::ElementId, window: &mut #window_ty, cx: &mut #app_ty) -> #output
            where
                #(#bounds,)*
            {
                let #props_ident { __zopra_key, __zopra_marker: _, #(#fields,)* } = self;
                #take_values
                let __zopra_given = __zopra_key.clone();
                // instances from one call site keep apart state by key, else by render order
                let key = __zopra_key.unwrap_or_else(|| {
                    ::zopra::gpui_kit::ElementId::from(::zopra::__private::occurrence(&site, cx))
                });
                let element = window.with_id(site, |window| {
                    window.with_id(key, |window| #name(#(#call_args,)* window, cx))
                });
                #finish
            }
        }

        #[allow(non_camel_case_types)]
        impl<#(#params,)* #(#states,)*> ::std::clone::Clone for #props_ident<#(#args,)* #(#states,)*>
        where
            #(#impl_where,)*
            #(#clone_bounds,)*
        {
            fn clone(&self) -> Self {
                Self {
                    __zopra_key: ::std::clone::Clone::clone(&self.__zopra_key),
                    __zopra_marker: ::std::marker::PhantomData,
                    #(#fields: ::std::clone::Clone::clone(&self.#fields),)*
                }
            }
        }

        #[allow(non_camel_case_types)]
        impl<#(#params,)* #(#states,)*> ::std::cmp::PartialEq for #props_ident<#(#args,)* #(#states,)*>
        where
            #(#impl_where,)*
            #(#eq_bounds,)*
        {
            fn eq(&self, other: &Self) -> bool {
                self.__zopra_key == other.__zopra_key #(&& self.#fields == other.#fields)*
            }
        }

        #component_impl
    })
}
