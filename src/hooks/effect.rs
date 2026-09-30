/// Runs an effect when the dependencies change
/// Must be called inside a `#[component]` body.
///
/// # Example
///
/// ```ignore
/// use_effect!(|| {
///     println!("count: {count}");
/// }, [count]);
/// ```
///
/// NOTE: effects do NOT run on mount, but only when a dependency is notified.
/// Run one-shot startup logic directly in the component body instead.
#[macro_export]
macro_rules! use_effect {
    ($effect:expr, [$($dep:expr),* $(,)?] $(,)?) => {{
        compile_error!(
            "use_effect! is missing the `cx` argument: write `use_effect!(effect, [deps], cx)`. \
             This error usually means the call is not inside a #[component] body (zopra hooks \
             only work there). If you are inside a #[component] and already passed `cx` \
             somewhere by hand, remove it as the macro rewriter injects it for you."
        );
    }};
    ($effect:expr, [$($dep:expr),* $(,)?], $cx:expr) => {{
        let effect: std::rc::Rc<std::cell::RefCell<Box<dyn FnMut(&mut App)>>> =
            std::rc::Rc::new(std::cell::RefCell::new(Box::new($effect)));
        $(
            let effect_clone = effect.clone();
            $cx.observe(&$dep, move |_view, _entity, cx| {
                (effect_clone.borrow_mut())(cx);
            }).detach();
        )*
    }};
}
