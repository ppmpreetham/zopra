/// Runs an effect when the dependencies change
///
/// # Example
///
/// ```
/// use_effect!([count], || {
///     println!("count: {count}");
/// });
/// ```
#[macro_export]
macro_rules! use_effect {
    ([$($dep:expr),* $(,)?], $effect:expr, $cx:expr) => {{
        let effect = std::rc::Rc::new(std::cell::RefCell::new($effect));
        $(
            let effect_clone = effect.clone();
            $cx.observe(&$dep, move |_view, _entity, cx| {
                (effect_clone.borrow_mut())(cx);
            }).detach();
        )*
    }};
}
