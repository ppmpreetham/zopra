use gpui_kit::{App, Window};

use crate::scope::{provide, provided};

/// Makes `value` available to this component and everything it renders, keyed by type.
/// The innermost provider wins, and the value goes away when this component finishes rendering.
///
/// Signals are `Copy`, so providing one shares the state itself:
///
/// ```ignore
/// let tabs = use_signal(Tabs::default());
/// provide_context(tabs);
///
/// // in any child
/// let tabs: Signal<Tabs> = use_context();
/// tabs(|tabs| tabs.active = 0);
/// ```
pub fn provide_context<T: 'static>(value: T, _window: &mut Window, cx: &mut App) {
    provide(value, cx);
}

/// Reads the innermost value provided by a parent, or `None` if there is none.
///
/// Elements rendered later, outside the parent's render (e.g. popup menu items), see no context.
pub fn try_use_context<T: Clone + 'static>(_window: &mut Window, cx: &mut App) -> Option<T> {
    provided::<T>(cx).cloned()
}

/// Reads the innermost value provided by a parent.
///
/// # Panics
///
/// Panics if no parent component provided a `T`.
#[track_caller]
pub fn use_context<T: Clone + 'static>(window: &mut Window, cx: &mut App) -> T {
    try_use_context(window, cx).unwrap_or_else(|| {
        panic!(
            "use_context::<{}>() found no value; call provide_context in a parent component first",
            std::any::type_name::<T>()
        )
    })
}
