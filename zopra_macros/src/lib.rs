use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, Punct, Spacing, Span, TokenStream as TokenStream2, TokenTree};
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::{
    Expr, ExprAsync, ExprCall, ExprClosure, ExprForLoop, ExprIf, ExprLoop, ExprMatch, ExprWhile,
    ItemFn, Local, Macro, Pat, Stmt, parse_macro_input, parse_quote,
    visit::{self, Visit},
    visit_mut::{self, VisitMut},
};

const SKIP_FNS: &[&str] = &[
    "use_state",
    "use_effect",
    "use_table",
    "use_table_with",
    "use_model",
    "use_resource",
    "use_window",
    "use_webview",
];
const CLOSURE_CX_FNS: &[&str] = &["use_callback", "use_event", "use_effect"];
const ASYNC_CX_FNS: &[&str] = &["use_async"];

fn is_bare_ident(arg: &Expr, name: &str) -> bool {
    matches!(arg, Expr::Path(p) if p.path.segments.last().is_some_and(|s| s.ident == name))
}

fn path_last_ident(expr: &Expr) -> String {
    match expr {
        Expr::Path(p) => p
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn is_hook_path(path: &syn::Path, fns: &[&str]) -> bool {
    let Some(seg) = path.segments.last() else {
        return false;
    };
    if !fns.contains(&seg.ident.to_string().as_str()) {
        return false;
    }
    if path.segments.len() == 1 {
        return true;
    }
    let first = path.segments.first().map(|s| s.ident.to_string());
    match first.as_deref() {
        Some("zopra") => true,
        Some("crate") => path.segments.len() == 3 && path.segments[1].ident == "hooks",
        _ => false,
    }
}

fn is_setter_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "Setter"))
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
    let Some(syn::GenericArgument::Type(inner)) = args.args.first() else {
        return None;
    };
    Some(inner)
}

fn has_elided_prop_lifetime(ty: &syn::Type) -> bool {
    if let syn::Type::Reference(reference) = ty {
        return reference.lifetime.is_none();
    }
    option_inner(ty)
        .and_then(|inner| match inner {
            syn::Type::Reference(reference) => Some(reference.lifetime.is_none()),
            _ => None,
        })
        .unwrap_or(false)
}

fn name_prop_lifetime(ty: &mut syn::Type, lifetime: &syn::Lifetime) {
    let target = if let syn::Type::Reference(reference) = ty {
        Some(reference)
    } else if let syn::Type::Path(path) = ty {
        path.path
            .segments
            .last_mut()
            .and_then(|segment| match &mut segment.arguments {
                syn::PathArguments::AngleBracketed(args) => {
                    args.args.iter_mut().find_map(|arg| match arg {
                        syn::GenericArgument::Type(syn::Type::Reference(reference)) => {
                            Some(reference)
                        }
                        _ => None,
                    })
                }
                _ => None,
            })
    } else {
        None
    };
    if let Some(reference) = target
        && reference.lifetime.is_none()
    {
        reference.lifetime = Some(lifetime.clone());
    }
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
struct ClosureNames {
    used: Vec<syn::Ident>,
    shadowed: HashSet<String>,
}

impl<'ast> Visit<'ast> for ClosureNames {
    fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
        if path.path.segments.len() == 1 {
            let ident = path.path.segments[0].ident.clone();
            if !self.used.iter().any(|used| used == &ident) {
                self.used.push(ident);
            }
        }
        visit::visit_expr_path(self, path);
    }

    fn visit_pat_ident(&mut self, pat: &'ast syn::PatIdent) {
        self.shadowed.insert(pat.ident.to_string());
        visit::visit_pat_ident(self, pat);
    }
}

#[derive(Default)]
struct Scope {
    signals: HashSet<String>,
    setters: HashSet<String>,
    models: HashSet<String>,
    window_launchers: HashSet<String>,
    plain: HashSet<String>,
}

#[derive(Default)]
struct InjectCx {
    scopes: Vec<Scope>,
    cond_depth: usize,
    closure_depth: usize,
    block_depth: usize,
    error: Option<syn::Error>,
}

impl InjectCx {
    fn event_arity(name: &str) -> Option<usize> {
        match name {
            "onClick"
            | "on_click"
            | "onChange"
            | "on_change"
            | "onMouseDown"
            | "on_mouse_down"
            | "onMouseUp"
            | "on_mouse_up"
            | "onMouseMove"
            | "on_mouse_move"
            | "onMouseExit"
            | "on_mouse_exit"
            | "onMousePressure"
            | "on_mouse_pressure"
            | "onPinch"
            | "on_pinch"
            | "onMouseDownOut"
            | "on_mouse_down_out"
            | "onMouseUpOut"
            | "on_mouse_up_out"
            | "onAnyMouseDown"
            | "on_any_mouse_down"
            | "onAnyMouseUp"
            | "on_any_mouse_up"
            | "onKeyDown"
            | "on_key_down"
            | "onKeyUp"
            | "on_key_up"
            | "onModifiersChanged"
            | "on_modifiers_changed"
            | "onHover"
            | "on_hover"
            | "onAuxClick"
            | "on_aux_click"
            | "onScrollWheel"
            | "on_scroll_wheel"
            | "onDrag"
            | "on_drag"
            | "onDragMove"
            | "on_drag_move"
            | "onDrop"
            | "on_drop"
            | "onAction"
            | "on_action"
            | "onBoxedAction"
            | "on_boxed_action"
            | "onDoubleClick"
            | "on_double_click"
            | "onA11yAction"
            | "on_a11y_action"
            | "onResize"
            | "on_resize" => Some(3),
            _ => None,
        }
    }

    fn normalize_event_value(&mut self, attr: &str, group: &Group) -> Option<Group> {
        let arity = Self::event_arity(attr)?;
        let parsed = syn::parse2::<Expr>(group.stream()).ok()?;
        let mut closure = match parsed {
            Expr::Closure(closure) => closure,
            Expr::Path(path) if self.is_setter(&path_last_ident(&Expr::Path(path.clone()))) => {
                let setter = path;
                parse_quote!(|__value| #setter(__value))
            }
            _ => return None,
        };

        if closure.inputs.is_empty() {
            closure.inputs.push(parse_quote!(__event));
        }
        while closure.inputs.len() < arity {
            closure.inputs.push(if closure.inputs.len() == arity - 1 {
                parse_quote!(cx)
            } else {
                parse_quote!(window)
            });
        }
        closure.capture = Some(parse_quote!(move));
        let mut expr = self.wrap_closure(closure, None);
        self.visit_expr_mut(&mut expr);
        let mut rewritten = quote::quote!(#expr);
        self.rewrite_signal_calls(&mut rewritten);
        Some(rebuild_group(group, rewritten))
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn is_signal(&self, name: &str) -> bool {
        self.scopes
            .iter()
            .rev()
            .find(|scope| scope.plain.contains(name) || scope.signals.contains(name))
            .is_some_and(|scope| scope.signals.contains(name))
    }

    fn is_setter(&self, name: &str) -> bool {
        self.scopes
            .iter()
            .rev()
            .find(|scope| scope.plain.contains(name) || scope.setters.contains(name))
            .is_some_and(|scope| scope.setters.contains(name))
    }

    fn is_window_launcher(&self, name: &str) -> bool {
        self.scopes
            .iter()
            .rev()
            .find(|scope| scope.plain.contains(name) || scope.window_launchers.contains(name))
            .is_some_and(|scope| scope.window_launchers.contains(name))
    }

    fn is_clone_handle(&self, name: &str) -> bool {
        self.scopes.iter().any(|scope| {
            scope.signals.contains(name)
                || scope.setters.contains(name)
                || scope.models.contains(name)
                || scope.window_launchers.contains(name)
        })
    }

    fn is_model(&self, name: &str) -> bool {
        self.scopes
            .iter()
            .rev()
            .find(|scope| scope.plain.contains(name) || scope.models.contains(name))
            .is_some_and(|scope| scope.models.contains(name))
    }

    fn clone_names(&self, closure: &ExprClosure) -> Vec<syn::Ident> {
        let mut names = ClosureNames::default();
        names.visit_expr_closure(closure);
        names
            .used
            .into_iter()
            .filter(|ident| {
                let name = ident.to_string();
                !names.shadowed.contains(&name) && self.is_clone_handle(&name)
            })
            .collect()
    }

    fn wrap_closure(&self, mut closure: ExprClosure, parameters: Option<Vec<Pat>>) -> Expr {
        if let Some(inputs) = parameters {
            closure.inputs = inputs.into_iter().collect();
        }
        closure.capture = Some(parse_quote!(move));
        let clones = self.clone_names(&closure);
        if clones.is_empty() {
            return Expr::Closure(closure);
        }

        let clone_stmts = clones
            .iter()
            .map(|ident| parse_quote!(let #ident = #ident.clone();))
            .collect::<Vec<Stmt>>();
        let body = match *closure.body {
            Expr::Block(mut block) => {
                let mut statements = clone_stmts;
                statements.append(&mut block.block.stmts);
                block.block.stmts = statements;
                Expr::Block(block)
            }
            body => {
                let mut statements = clone_stmts;
                statements.push(Stmt::Expr(body, None));
                Expr::Block(parse_quote!({ #(#statements)* }))
            }
        };
        closure.body = Box::new(body);
        closure.capture = Some(parse_quote!(move));
        let outer = clones
            .iter()
            .map(|ident| parse_quote!(let #ident = #ident.clone();))
            .collect::<Vec<Stmt>>();
        let closure = Expr::Closure(closure);
        parse_quote!({ #(#outer)* #closure })
    }

    fn register_pat(&mut self, pat: &Pat, as_signal: bool) {
        let names = collect_idents_from_pat(pat)
            .into_iter()
            .map(|ident| ident.to_string());
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

    fn ends_with_ident(tokens: &TokenStream2, name: &str) -> bool {
        matches!(tokens.clone().into_iter().last(),
            Some(TokenTree::Ident(id)) if id == name)
    }

    fn cx_only(tokens: &TokenStream2) -> bool {
        matches!(tokens.clone().into_iter().collect::<Vec<_>>().as_slice(), [TokenTree::Ident(id)] if id == "cx")
    }

    fn append_ident(tokens: &mut TokenStream2, name: &str, span: Span) {
        if !tokens.is_empty() {
            let mut comma = Punct::new(',', Spacing::Alone);
            comma.set_span(span);
            tokens.extend(std::iter::once(TokenTree::Punct(comma)));
        }
        tokens.extend(std::iter::once(TokenTree::Ident(proc_macro2::Ident::new(
            name,
            Span::call_site(),
        ))));
    }

    fn rewrite_signal_calls(&mut self, tokens: &mut TokenStream2) {
        let toks: Vec<TokenTree> = tokens.clone().into_iter().collect();
        let mut out: Vec<TokenTree> = Vec::with_capacity(toks.len());
        let mut i = 0;

        while i < toks.len() {
            match &toks[i] {
                TokenTree::Ident(attr)
                    if Self::event_arity(&attr.to_string()).is_some()
                        && matches!(toks.get(i + 1), Some(TokenTree::Punct(p)) if p.as_char() == '=')
                        && let Some(TokenTree::Group(group)) = toks.get(i + 2)
                        && group.delimiter() == Delimiter::Brace
                        && let Some(rewritten) =
                            self.normalize_event_value(&attr.to_string(), group) =>
                {
                    out.push(TokenTree::Ident(attr.clone()));
                    out.push(toks[i + 1].clone());
                    out.push(TokenTree::Group(rewritten));
                    i += 3;
                }
                TokenTree::Group(group) => {
                    let mut inner = group.stream();
                    self.rewrite_signal_calls(&mut inner);
                    out.push(TokenTree::Group(rebuild_group(group, inner)));
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
                    if group.stream().is_empty() || Self::cx_only(&group.stream()) {
                        out.push(TokenTree::Ident(ident.clone()));
                        i += 2;
                        continue;
                    }
                    let mut inner = group.stream();
                    let ends_with_cx = Self::ends_with_ident(&inner, "cx");
                    self.rewrite_signal_calls(&mut inner);

                    if !ends_with_cx {
                        Self::append_ident(&mut inner, "cx", group.span());
                    }

                    out.push(TokenTree::Ident(ident.clone()));
                    out.push(TokenTree::Group(rebuild_group(&group, inner)));
                    i += 2;
                }
                TokenTree::Ident(ident)
                    if self.is_setter(&ident.to_string())
                        && matches!(toks.get(i + 1), Some(TokenTree::Punct(p)) if p.as_char() == '.')
                        && matches!(toks.get(i + 2), Some(TokenTree::Ident(method)) if method == "current")
                        && let Some(TokenTree::Group(group)) = toks.get(i + 3)
                        && group.delimiter() == Delimiter::Parenthesis =>
                {
                    let mut inner = group.stream();
                    if !Self::ends_with_ident(&inner, "cx") {
                        Self::append_ident(&mut inner, "cx", group.span());
                    }
                    out.push(TokenTree::Ident(ident.clone()));
                    out.push(toks[i + 1].clone());
                    out.push(toks[i + 2].clone());
                    out.push(TokenTree::Group(rebuild_group(group, inner)));
                    i += 4;
                }
                TokenTree::Ident(ident)
                    if self.is_setter(&ident.to_string())
                        && let Some(TokenTree::Group(group)) = toks.get(i + 1)
                        && group.delimiter() == Delimiter::Parenthesis =>
                {
                    let mut inner = group.stream();
                    let mut arg_tokens = inner.clone().into_iter();
                    let first = arg_tokens.next();
                    let closure_arg = matches!(first, Some(TokenTree::Punct(ref p)) if p.as_char() == '|')
                        || matches!(first, Some(TokenTree::Ident(ref id)) if id == "move" || id == "async");
                    let method = if closure_arg { "update" } else { "set" };
                    if !Self::ends_with_ident(&inner, "cx") {
                        Self::append_ident(&mut inner, "cx", group.span());
                    }
                    let mut dot = Punct::new('.', Spacing::Alone);
                    dot.set_span(ident.span());
                    out.push(TokenTree::Ident(ident.clone()));
                    out.push(TokenTree::Punct(dot));
                    out.push(TokenTree::Ident(proc_macro2::Ident::new(
                        method,
                        ident.span(),
                    )));
                    out.push(TokenTree::Group(rebuild_group(group, inner)));
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
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if let Expr::Call(call) = expr {
            if let Expr::Path(path) = &*call.func {
                let name = path_last_ident(&Expr::Path(path.clone()));
                if self.is_setter(&name)
                    && (call.args.len() == 1
                        || (call.args.len() == 2 && is_bare_ident(call.args.last().unwrap(), "cx")))
                {
                    let method_name = if matches!(call.args.first(), Some(Expr::Closure(_))) {
                        "update"
                    } else {
                        "set"
                    };
                    let method = syn::Ident::new(method_name, path.path.span());
                    let setter = path.clone();
                    let value = call.args.first().cloned().unwrap();
                    *expr = parse_quote!(#setter.#method(#value, cx));
                    self.visit_expr_mut(expr);
                    return;
                }
                if self.is_signal(&name)
                    && (call.args.is_empty()
                        || (call.args.len() == 1
                            && is_bare_ident(call.args.first().unwrap(), "cx")))
                {
                    *expr = Expr::Path(path.clone());
                    return;
                }
            }
        }
        visit_mut::visit_expr_mut(self, expr);
    }

    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        self.push_scope();
        self.block_depth += 1;
        visit_mut::visit_block_mut(self, block);
        self.block_depth -= 1;
        self.pop_scope();
    }

    fn visit_expr_async_mut(&mut self, expr_async: &mut ExprAsync) {
        self.closure_depth += 1;
        visit_mut::visit_expr_async_mut(self, expr_async);
        self.closure_depth -= 1;
    }

    fn visit_expr_closure_mut(&mut self, closure: &mut ExprClosure) {
        self.push_scope();
        self.closure_depth += 1;
        for input in &closure.inputs {
            self.register_pat(input, false);
        }
        visit_mut::visit_expr_closure_mut(self, closure);
        self.closure_depth -= 1;
        self.pop_scope();
    }

    fn visit_local_mut(&mut self, local: &mut Local) {
        let alias_of_signal = matches!(
            local.init.as_ref().map(|i| &*i.expr),
            Some(expr) if self.is_signal(&path_last_ident(expr))
        );
        let alias_of_setter = matches!(
            local.init.as_ref().map(|i| &*i.expr),
            Some(Expr::MethodCall(call))
                if call.method == "clone"
                    && matches!(&*call.receiver, Expr::Path(path)
                        if self.is_setter(&path_last_ident(&Expr::Path(path.clone()))))
        );
        let alias_of_window_launcher = matches!(
            local.init.as_ref().map(|i| &*i.expr),
            Some(Expr::MethodCall(call))
                if call.method == "clone"
                    && matches!(&*call.receiver, Expr::Path(path)
                        if self.is_window_launcher(&path_last_ident(&Expr::Path(path.clone()))))
        );
        let pair_is_state = matches!(local.init.as_ref().map(|i| &*i.expr), Some(Expr::Call(ExprCall { func, .. }))
            if matches!(&**func, Expr::Path(p) if is_hook_path(&p.path, &["use_state"])))
            && matches!(&local.pat, Pat::Tuple(tuple) if tuple.elems.len() == 2);
        let snap_hook = matches!(local.init.as_ref().map(|i| &*i.expr), Some(Expr::Call(ExprCall { func, .. }))
            if matches!(&**func, Expr::Path(p) if is_hook_path(&p.path, &["use_resource"])))
            && !matches!(&local.pat, Pat::Tuple(_));
        let model_hook = matches!(local.init.as_ref().map(|i| &*i.expr), Some(Expr::Call(ExprCall { func, .. }))
            if matches!(&**func, Expr::Path(p) if is_hook_path(&p.path, &["use_model"])));
        let window_hook = matches!(local.init.as_ref().map(|i| &*i.expr), Some(Expr::Call(ExprCall { func, .. }))
            if matches!(&**func, Expr::Path(p) if is_hook_path(&p.path, &["use_window"])));
        let table_with_hook = matches!(local.init.as_ref().map(|i| &*i.expr), Some(Expr::Call(ExprCall { func, .. }))
            if matches!(&**func, Expr::Path(p) if is_hook_path(&p.path, &["use_table_with"])));
        let webview_hook = matches!(local.init.as_ref().map(|i| &*i.expr), Some(Expr::Call(ExprCall { func, .. }))
            if matches!(&**func, Expr::Path(p) if is_hook_path(&p.path, &["use_webview"])));
        visit_mut::visit_local_mut(self, local);
        if alias_of_window_launcher {
            self.register_pat(&local.pat, false);
            if let Some(scope) = self.scopes.last_mut() {
                for ident in collect_idents_from_pat(&local.pat) {
                    scope.window_launchers.insert(ident.to_string());
                }
            }
        } else if alias_of_setter {
            if let Some(scope) = self.scopes.last_mut() {
                for ident in collect_idents_from_pat(&local.pat) {
                    scope.setters.insert(ident.to_string());
                }
            }
        } else if alias_of_signal {
            self.register_pat(&local.pat, true);
        } else if pair_is_state {
            if let Pat::Tuple(tuple) = &local.pat {
                if let Some(first) = tuple.elems.first() {
                    self.register_pat(first, true);
                }
                if let Some(second) = tuple.elems.get(1) {
                    if let Some(scope) = self.scopes.last_mut() {
                        for ident in collect_idents_from_pat(second) {
                            scope.setters.insert(ident.to_string());
                        }
                    }
                }
            }
        } else if model_hook {
            self.register_pat(&local.pat, false);
            if let Some(scope) = self.scopes.last_mut() {
                for ident in collect_idents_from_pat(&local.pat) {
                    scope.models.insert(ident.to_string());
                }
            }
        } else if window_hook {
            self.register_pat(&local.pat, false);
            if let Some(scope) = self.scopes.last_mut() {
                for ident in collect_idents_from_pat(&local.pat) {
                    scope.window_launchers.insert(ident.to_string());
                }
            }
        } else if table_with_hook || webview_hook {
            self.register_pat(&local.pat, false);
        } else {
            self.register_pat(&local.pat, snap_hook || Self::init_is_hook_call(local));
        }
    }

    fn visit_expr_match_mut(&mut self, expr_match: &mut ExprMatch) {
        self.visit_expr_mut(&mut expr_match.expr);
        for arm in &mut expr_match.arms {
            self.push_scope();
            self.cond_depth += 1;
            self.register_pat(&arm.pat, false);
            if let Pat::Guard(guard_pat) = &mut arm.pat {
                self.visit_expr_mut(&mut guard_pat.guard);
            }
            self.visit_expr_mut(&mut arm.body);
            self.cond_depth -= 1;
            self.pop_scope();
        }
    }

    fn visit_expr_if_mut(&mut self, expr_if: &mut ExprIf) {
        self.visit_expr_mut(&mut expr_if.cond);

        self.push_scope();
        self.cond_depth += 1;
        if let Expr::Let(expr_let) = &*expr_if.cond {
            self.register_pat(&expr_let.pat, false);
        }
        self.visit_block_mut(&mut expr_if.then_branch);
        self.cond_depth -= 1;
        self.pop_scope();

        if let Some((_, else_expr)) = &mut expr_if.else_branch {
            self.visit_expr_mut(else_expr);
        }
    }

    fn visit_expr_while_mut(&mut self, expr_while: &mut ExprWhile) {
        self.visit_expr_mut(&mut expr_while.cond);

        self.push_scope();
        self.cond_depth += 1;
        if let Expr::Let(expr_let) = &*expr_while.cond {
            self.register_pat(&expr_let.pat, false);
        }
        self.visit_block_mut(&mut expr_while.body);
        self.cond_depth -= 1;
        self.pop_scope();
    }

    fn visit_expr_for_loop_mut(&mut self, expr_for_loop: &mut ExprForLoop) {
        self.visit_expr_mut(&mut expr_for_loop.expr);

        self.push_scope();
        self.cond_depth += 1;
        self.register_pat(&expr_for_loop.pat, false);
        self.visit_block_mut(&mut expr_for_loop.body);
        self.cond_depth -= 1;
        self.pop_scope();
    }

    fn visit_expr_loop_mut(&mut self, expr_loop: &mut ExprLoop) {
        self.push_scope();
        self.cond_depth += 1;
        self.visit_block_mut(&mut expr_loop.body);
        self.cond_depth -= 1;
        self.pop_scope();
    }

    fn visit_expr_call_mut(&mut self, call: &mut ExprCall) {
        visit_mut::visit_expr_call_mut(self, call);

        if let Expr::Path(path) = &*call.func
            && is_hook_path(&path.path, &["use_async", "use_callback", "use_event"])
            && let Some(Expr::Closure(closure)) = call.args.last_mut()
        {
            inject_cx_param(closure);
            let wrapped = self.wrap_closure(closure.clone(), None);
            *call.args.last_mut().unwrap() = wrapped;
        }

        if let Expr::Path(path) = &*call.func {
            let name = path_last_ident(&Expr::Path(path.clone()));
            if is_hook_path(&path.path, &["use_state", "use_model"])
                && call.args.len() == 1
                && !matches!(call.args.first(), Some(Expr::Closure(_)))
            {
                let value = call.args.first().cloned().unwrap();
                let value = match value {
                    Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit),
                        ..
                    }) if is_hook_path(&path.path, &["use_state"]) => {
                        parse_quote!(::gpui_kit::SharedString::new_static(#lit))
                    }
                    value => value,
                };
                call.args[0] = parse_quote!(|| #value);
            }
            if self.is_setter(&name)
                && (call.args.len() == 1
                    || (call.args.len() == 2 && is_bare_ident(call.args.last().unwrap(), "cx")))
            {
                let arg = call.args.first().unwrap();
                let method = if matches!(arg, Expr::Closure(_)) {
                    syn::Ident::new("update", path.path.span())
                } else {
                    syn::Ident::new("set", path.path.span())
                };
                let setter = path.clone();
                call.func = parse_quote!(#setter.#method);
                call.args.push(parse_quote!(cx));
                return;
            }
            if self.is_signal(&name)
                && (call.args.is_empty()
                    || (call.args.len() == 1 && is_bare_ident(call.args.first().unwrap(), "cx")))
            {
                call.args.clear();
                *call.func = Expr::Path(path.clone());
                return;
            }
        }

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
            if is_hook
                && (self.cond_depth > 0 || self.closure_depth > 0 || self.block_depth > 1)
                && self.error.is_none()
                && let Some(span) = path_expr.path.segments.last().map(|s| s.ident.span())
            {
                let message = if self.closure_depth > 0 {
                    "zopra hooks must be called at the top level of a #[component] body, not inside a closure or event handler; declare the hook first, then use its handle in the closure"
                } else if self.cond_depth > 0 {
                    "zopra hooks must be called unconditionally at the top level of a #[component] body, not inside if/match/loop; declare the hook first, then use its value in the branch"
                } else {
                    "zopra hooks must be called at the top level of a #[component] body, not inside a nested block; move the hook call into the component body"
                };
                self.error = Some(syn::Error::new(span, message));
            }
            let ends_with_window_cx = call.args.len() >= 2
                && is_bare_ident(call.args.last().unwrap(), "cx")
                && is_bare_ident(&call.args[call.args.len() - 2], "window");
            if name == "use_table"
                && let Some(expr) = call.args.first_mut()
            {
                let inner = expr.clone();
                *expr = syn::parse_quote!(|| #inner);
            }
            if (name == "use_state"
                || name == "use_model"
                || name == "use_table"
                || name == "use_table_with"
                || name == "use_resource"
                || name == "use_window"
                || name == "use_webview")
                && !ends_with_window_cx
            {
                call.args.push(parse_quote!(window));
            }
            if !call.args.last().is_some_and(|a| is_bare_ident(a, "cx")) {
                call.args.push(parse_quote!(cx));
            }
            if name == "use_window"
                && let Some(Expr::Closure(options)) = call.args.first_mut()
            {
                options.capture = Some(parse_quote!(move));
            }
            return;
        }

        if let Some(Expr::Closure(closure)) = call.args.last_mut()
            && (is_hook_path(&path_expr.path, CLOSURE_CX_FNS)
                || is_hook_path(&path_expr.path, ASYNC_CX_FNS))
        {
            inject_cx_param(closure);
        }
    }

    fn visit_expr_method_call_mut(&mut self, call: &mut syn::ExprMethodCall) {
        visit_mut::visit_expr_method_call_mut(self, call);
        let binding = match &*call.receiver {
            Expr::Path(path) => path_last_ident(&Expr::Path(path.clone())),
            _ => return,
        };
        if self.is_window_launcher(&binding) && call.method == "open" {
            if let Some(Expr::Closure(render)) = call.args.first_mut() {
                if render.inputs.is_empty() {
                    render.inputs.push(parse_quote!(window));
                    render.inputs.push(parse_quote!(cx));
                }
                render.capture = Some(parse_quote!(move));
                if !call.args.iter().any(|arg| is_bare_ident(arg, "cx")) {
                    call.args.push(parse_quote!(cx));
                }
            }
            return;
        }
        if self.is_setter(&binding)
            && matches!(
                call.method.to_string().as_str(),
                "current" | "set" | "update"
            )
            && !call.args.iter().any(|arg| is_bare_ident(arg, "cx"))
        {
            if call.method == "update" && call.args.len() == 1 {
                call.args.push(parse_quote!(cx));
            } else if matches!(call.method.to_string().as_str(), "current" | "set") {
                call.args.push(parse_quote!(cx));
            }
        } else if self.is_clone_handle(&binding) && self.is_model(&binding) {
            match call.method.to_string().as_str() {
                "read" if call.args.is_empty() => call.args.push(parse_quote!(cx)),
                "update" if call.args.len() == 1 => {
                    let f = call.args.pop().unwrap();
                    call.args.push(parse_quote!(cx));
                    call.args.push(f);
                }
                _ => {}
            }
        }
    }

    fn visit_macro_mut(&mut self, mac: &mut Macro) {
        if mac.path.is_ident("signals") {
            if let Ok(names) = syn::parse2::<SignalNames>(mac.tokens.clone())
                && let Some(scope) = self.scopes.last_mut()
            {
                for ident in names.idents {
                    scope.signals.insert(ident.to_string());
                }
            }
            mac.tokens = TokenStream2::new();
            return;
        }

        let is_effect_mac = is_hook_path(&mac.path, &["use_effect"]);
        let is_hook_mac = is_hook_path(&mac.path, &["use_effect", "use_table"]);

        if is_effect_mac
            && (self.cond_depth > 0 || self.closure_depth > 0 || self.block_depth > 1)
            && self.error.is_none()
        {
            let message = if self.closure_depth > 0 {
                "use_effect! must be called at the top level of a #[component] body, not inside a closure or event handler"
            } else if self.cond_depth > 0 {
                "use_effect! must be called unconditionally at the top level of a #[component] body, not inside if/match/loop"
            } else {
                "use_effect! must be called at the top level of a #[component] body, not inside a nested block"
            };
            self.error = Some(syn::Error::new(mac.path.span(), message));
        }

        let mut tokens = mac.tokens.clone();
        if is_effect_mac {
            let token_vec = tokens.clone().into_iter().collect::<Vec<_>>();
            if let Some(comma_ix) = token_vec
                .iter()
                .position(|token| matches!(token, TokenTree::Punct(p) if p.as_char() == ','))
            {
                let effect_tokens = token_vec[..comma_ix]
                    .iter()
                    .cloned()
                    .collect::<TokenStream2>();
                if let Ok(Expr::Closure(closure)) = syn::parse2::<Expr>(effect_tokens) {
                    let wrapped = self.wrap_closure(closure, None);
                    let mut wrapped = wrapped;
                    self.visit_expr_mut(&mut wrapped);
                    let remaining = token_vec[comma_ix..]
                        .iter()
                        .cloned()
                        .collect::<TokenStream2>();
                    tokens = quote::quote!(#wrapped #remaining);
                }
            }
        }
        self.rewrite_signal_calls(&mut tokens);

        if is_hook_mac && !Self::ends_with_ident(&tokens, "cx") {
            Self::append_ident(&mut tokens, "cx", mac.bang_token.span);
        }

        if is_effect_mac {
            // lets the declarative macro distinguish component-rewritten calls
            // from direct use outside a component, where `window` and `cx` are unavailable.
            Self::append_ident(&mut tokens, "__zopra_component", mac.bang_token.span);
        }

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

fn rebuild_group(group: &Group, stream: TokenStream2) -> Group {
    let mut rebuilt = Group::new(group.delimiter(), stream);
    rebuilt.set_span(group.span());
    rebuilt
}

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    match generate_component(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate_component(mut input: ItemFn) -> syn::Result<TokenStream2> {
    use quote::quote;
    use syn::spanned::Spanned;
    use syn::{FnArg, Pat, PatType};

    let mut errors: Vec<syn::Error> = Vec::new();
    let mut props: Vec<(syn::Ident, syn::Type, Option<Pat>)> = Vec::new();
    for arg in input.sig.inputs.iter() {
        match arg {
            FnArg::Typed(PatType { pat, ty, .. }) => match &**pat {
                Pat::Ident(pat_ident) => {
                    props.push((pat_ident.ident.clone(), (**ty).clone(), None))
                }
                pat_other => {
                    let field =
                        syn::Ident::new(&format!("__zopra_prop{}", props.len()), pat_other.span());
                    props.push((field, (**ty).clone(), Some((**pat).clone())));
                }
            },
            FnArg::Receiver(receiver) => errors.push(syn::Error::new_spanned(
                receiver,
                "methods (self) are not supported in components",
            )),
        }
    }
    if let Some(err) = errors.pop() {
        return Err(err);
    }

    let prop_lifetime = syn::Lifetime::new("'__zopra_p", Span::call_site());
    let needs_prop_lifetime = props.iter().any(|(_, ty, _)| has_elided_prop_lifetime(ty));
    if needs_prop_lifetime {
        input
            .sig
            .generics
            .params
            .insert(0, parse_quote!('__zopra_p));
        for (_, ty, _) in &mut props {
            name_prop_lifetime(ty, &prop_lifetime);
        }
        for arg in &mut input.sig.inputs {
            if let FnArg::Typed(arg) = arg {
                name_prop_lifetime(&mut arg.ty, &prop_lifetime);
            }
        }
    }

    let generic_use_list: TokenStream2 = input
        .sig
        .generics
        .params
        .iter()
        .map(|param| match param {
            syn::GenericParam::Type(tp) => {
                let ident = &tp.ident;
                quote! { #ident, }
            }
            syn::GenericParam::Lifetime(lt) => {
                let lifetime = &lt.lifetime;
                quote! { #lifetime, }
            }
            syn::GenericParam::Const(cp) => {
                let ident = &cp.ident;
                quote! { #ident, }
            }
        })
        .collect();
    if generic_use_list.is_empty() {
        input.sig.output = parse_quote!(-> impl ::zopra::gpui_kit::IntoElement + use<>);
    } else {
        input.sig.output =
            parse_quote!(-> impl ::zopra::gpui_kit::IntoElement + use<#generic_use_list>);
    }
    input
        .sig
        .inputs
        .push(parse_quote!(window: &mut ::zopra::gpui_kit::Window));
    input
        .sig
        .inputs
        .push(parse_quote!(cx: &mut ::zopra::gpui_kit::App));

    let mut rewriter = InjectCx::default();
    rewriter.push_scope();
    for arg in &input.sig.inputs {
        if let FnArg::Typed(arg) = arg
            && is_setter_type(&arg.ty)
        {
            if let Some(scope) = rewriter.scopes.last_mut() {
                for ident in collect_idents_from_pat(&arg.pat) {
                    scope.setters.insert(ident.to_string());
                }
            }
        }
    }
    rewriter.visit_block_mut(&mut input.block);
    rewriter.pop_scope();
    if let Some(err) = rewriter.error.take() {
        return Err(err);
    }

    let view_tail_mac = match input.block.stmts.last_mut() {
        Some(Stmt::Expr(Expr::Macro(expr_mac), None)) => Some(&mut expr_mac.mac),
        Some(Stmt::Macro(stmt_mac)) => Some(&mut stmt_mac.mac),
        _ => None,
    };
    if let Some(mac) = view_tail_mac
        && mac.path.is_ident("view")
    {
        let mut toks = mac.tokens.clone().into_iter();
        let starts_fragment = matches!(
            (toks.next(), toks.next()),
            (Some(TokenTree::Punct(a)), Some(TokenTree::Punct(b)))
                if a.as_char() == '<' && b.as_char() == '>'
        );
        if starts_fragment {
            let inner = mac.tokens.clone();
            mac.tokens = quote::quote! { <div> #inner </div> };
        }
    }

    let mut prop_store_tys: Vec<syn::Type> = Vec::new();
    let mut prop_call_exprs: Vec<TokenStream2> = Vec::new();
    let mut prop_optional: Vec<bool> = Vec::new();
    for (prop, ty, _) in &props {
        if let Some(inner) = option_inner(ty) {
            prop_store_tys.push((*inner).clone());
            prop_call_exprs.push(parse_quote!(#prop));
            prop_optional.push(true);
            continue;
        }
        prop_store_tys.push((*ty).clone());
        prop_call_exprs.push(parse_quote!(#prop));
        prop_optional.push(false);
    }

    let vis = &input.vis;
    let name = &input.sig.ident;
    let props_str = format!("{}Props", to_pascal(&name.to_string()));
    let props_ident = syn::Ident::new(&props_str, name.span());
    let tag_alias = syn::Ident::new(&to_pascal(&name.to_string()), name.span());

    let mut prop_fields = TokenStream2::new();
    let mut setters = TokenStream2::new();
    let mut prop_args = TokenStream2::new();
    let mut prop_defaults = TokenStream2::new();
    for ((prop, _, _), (store_ty, call_expr)) in props
        .iter()
        .zip(prop_store_tys.iter().zip(prop_call_exprs.iter()))
    {
        prop_fields.extend(quote! { #prop: ::std::option::Option<#store_ty>, });
        let string_like = matches!(store_ty, syn::Type::Path(tp)
            if tp.path.segments.last().is_some_and(|seg| seg.ident == "String" || seg.ident == "SharedString"));
        let setter_ty = if string_like {
            quote! { impl ::std::convert::Into<#store_ty> }
        } else {
            quote! { #store_ty }
        };
        let setter_value = if string_like {
            quote! { #prop.into() }
        } else {
            quote! { #prop }
        };
        setters.extend(quote! {
            #vis fn #prop(mut self, #prop: #setter_ty) -> Self {
                self.#prop = ::std::option::Option::Some(#setter_value);
                self
            }
        });
        prop_args.extend(quote! { #prop, });
        prop_defaults.extend(quote! { #prop: ::std::option::Option::None, });
        _ = call_expr;
    }
    let mut prop_unwraps = TokenStream2::new();
    for ((prop, _, pat), (call_expr, optional)) in props
        .iter()
        .zip(prop_call_exprs.iter().zip(prop_optional.iter()))
    {
        let err = format!(
            "missing required prop `{prop}` for component `{name}` (rsx tag `<{} ...>`)",
            name
        );
        match (pat, optional) {
            (_, true) => {}
            (Some(pat), false) => {
                prop_unwraps.extend(quote! {
                    let #prop = #prop.expect(#err);
                    let #prop = #call_expr;
                    let #pat = #prop.clone();
                });
            }
            (None, false) => {
                prop_unwraps.extend(quote! {
                    let #prop = #prop.expect(#err);
                    let #prop = #call_expr;
                });
            }
        }
    }

    let props_doc = props
        .iter()
        .map(|(prop, _, _)| prop.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let (impl_generics, ty_generics, where_clause) = input.sig.generics.split_for_impl();

    Ok(quote! {
        #[allow(non_snake_case, unused_variables)]
        #input

        #[doc = concat!("view! tag alias: `<", stringify!(#tag_alias), " prop={..} />` resolves to the props builder.")]
        #[allow(missing_docs, non_snake_case, non_camel_case_types)]
        #vis type #tag_alias #ty_generics = #props_ident #ty_generics;

        #[doc = concat!("Props builder for the ", stringify!(#name), " component (immediate-call model, no RenderOnce). Props: ", #props_doc)]
        #[allow(missing_docs, non_snake_case, non_camel_case_types)]
        #vis struct #props_ident #ty_generics #where_clause {
            #prop_fields
        }

        impl #impl_generics #props_ident #ty_generics #where_clause {
            #vis fn new() -> Self {
                Self { #prop_defaults }
            }

            #setters

            #[track_caller]
            #vis fn render(
                self,
                window: &mut ::zopra::gpui_kit::Window,
                cx: &mut ::zopra::gpui_kit::App,
            ) -> ::zopra::gpui_kit::AnyElement {
                let site = core::panic::Location::caller();
                ::zopra::gpui_kit::IntoElement::into_any_element(
                    self.render_at(::zopra::gpui_kit::ElementId::CodeLocation(*site), window, cx),
                )
            }

            #vis fn render_at(
                self,
                site: ::zopra::gpui_kit::ElementId,
                window: &mut ::zopra::gpui_kit::Window,
                cx: &mut ::zopra::gpui_kit::App,
            ) -> ::zopra::gpui_kit::AnyElement {
                let #props_ident { #prop_args } = self;
                #prop_unwraps
                let element = window.with_id(site, |window| {
                    #name(#prop_args window, cx)
                });
                ::zopra::gpui_kit::IntoElement::into_any_element(element)
            }
        }
    })
}
