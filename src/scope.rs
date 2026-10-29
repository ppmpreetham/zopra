use gpui_kit::{App, ElementId, Global};

struct Scope {
    undo: usize,
    children: usize,
}

#[derive(Default)]
struct RenderStack {
    scopes: Vec<Scope>,
    undo: Vec<fn(&mut App)>,
    children: Vec<(ElementId, usize)>,
}

impl Global for RenderStack {}

#[doc(hidden)]
pub fn enter(cx: &mut App) {
    let stack = cx.default_global::<RenderStack>();
    let scope = Scope {
        undo: stack.undo.len(),
        children: stack.children.len(),
    };
    stack.scopes.push(scope);
}

#[doc(hidden)]
pub fn exit(cx: &mut App) {
    let stack = cx.global_mut::<RenderStack>();
    let Some(scope) = stack.scopes.pop() else {
        return;
    };
    stack.children.truncate(scope.children);
    while cx.global::<RenderStack>().undo.len() > scope.undo {
        let undo = cx.global_mut::<RenderStack>().undo.pop();
        undo.into_iter().for_each(|undo| undo(cx));
    }
}

#[doc(hidden)]
pub fn occurrence(site: &ElementId, cx: &mut App) -> usize {
    let stack = cx.default_global::<RenderStack>();
    let Some(scope) = stack.scopes.last() else {
        return 0;
    };
    let siblings = &mut stack.children[scope.children..];
    if let Some((_, count)) = siblings.iter_mut().find(|(id, _)| id == site) {
        *count += 1;
        *count
    } else {
        stack.children.push((site.clone(), 0));
        0
    }
}

struct Provided<T>(Vec<T>);

impl<T> Default for Provided<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T: 'static> Global for Provided<T> {}

fn unprovide<T: 'static>(cx: &mut App) {
    cx.global_mut::<Provided<T>>().0.pop();
}

pub(crate) fn provide<T: 'static>(value: T, cx: &mut App) {
    cx.default_global::<Provided<T>>().0.push(value);
    cx.default_global::<RenderStack>().undo.push(unprovide::<T>);
}

pub(crate) fn provided<T: 'static>(cx: &App) -> Option<&T> {
    cx.try_global::<Provided<T>>()?.0.last()
}
