use gpui_kit::{App, Entity, Window};

/// Waits until after a render to run a closure, and runs it again when its dependencies change
pub fn run_effect<D, F>(deps: D, effect: F, window: &mut Window, cx: &mut App)
where
    D: Clone + PartialEq + 'static,
    F: FnOnce() + 'static,
{
    let previous: Entity<Option<D>> = window.use_state(cx, |_window, _cx| None);
    let changed = previous.read(cx).as_ref() != Some(&deps);
    if changed {
        previous.update(cx, |old, _cx| *old = Some(deps));
        window.defer(cx, move |_window, _cx| effect());
    }
}

/// Runs an effect after the initial render and again when its dependencies
/// differ from their values on the previous render.
///
/// Must be called inside a `#[component]` body.
///
/// # Example
///
/// ```ignore
/// use_effect!(move || {
///     println!("count: {count}");
/// }, [count]);
/// ```
///
/// Use an empty dependency list (`[]`) for an effect that runs only after
/// the initial render.

#[macro_export]
macro_rules! use_effect {
    ($effect:expr, [$($dep:expr),* $(,)?], $cx:expr, __zopra_component $(,)?) => {{
        $crate::hooks::run_effect(($($dep,)*), $effect, window, $cx);
    }};
    ($effect:expr, [$($dep:expr),* $(,)?], $cx:expr $(,)?) => {
        compile_error!(
            "use_effect! can only be used inside a #[component]. In a component, write \
             `use_effect!(effect, [deps])` and let Zopra inject `window` and `cx`. Outside a \
             component, call `zopra::hooks::run_effect(deps, effect, window, cx)` directly."
        );
    };
    ($effect:expr, [$($dep:expr),* $(,)?] $(,)?) => {
        compile_error!(
            "use_effect! is missing its component context. Write `use_effect!(effect, [deps])` \
             inside a #[component]; Zopra injects `window` and `cx`. Outside a component, call \
             `zopra::hooks::run_effect(deps, effect, window, cx)` directly."
        );
    };
    ($($tokens:tt)*) => {
        compile_error!(
            "invalid use_effect! syntax: write `use_effect!(effect, [deps])` inside a #[component]."
        );
    };
}

/// Evaluates an expression inline each time the component renders.
///
/// Despite its name, this macro does not run only when the component mounts.
/// It also does not schedule the expression to run after rendering.
///
/// # Example
///
/// ```ignore
/// use\_mount!({
///     println!("rendering");
/// });
/// ```
///
/// For logic that should run after the initial render, use `use_effect!` with
/// an empty dependency list (`[]`).

#[macro_export]
macro_rules! use_mount {
    ($effect:expr $(,)?) => {{
        $effect();
    }};
}

/// Evaluates an expression inline each time the component renders, passing
/// the component's `cx`.
///
/// Despite its name, this macro does not run only when the component mounts.
/// It also does not schedule the expression to run after rendering.
///
/// # Example
///
/// ```ignore
/// use\_mount\_cx!(|cx| {
///     // Use cx here
/// });
/// ```
///
/// For logic that should run after the initial render, use `use_effect!` with
/// an empty dependency list (`[]`).
#[macro_export]
macro_rules! use_mount_cx {
    ($effect:expr $(,)?) => {{
        $effect(cx);
    }};
}
