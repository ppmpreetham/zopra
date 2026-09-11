/// Runs an effect when the dependencies change
///
/// # Example
///
/// ```
/// use_effect!(|| {
///     println!("count: {count}");
/// }, [count]);
/// ```
#[macro_export]
macro_rules! use_effect {
    ($effect:expr, [$($dep:expr),* $(,)?], $cx:expr) => {{
        let effect = std::rc::Rc::new(std::cell::RefCell::new($effect));
        $(
            let effect_clone = effect.clone();
            $cx.observe(&$dep, move |_view, _entity, cx| {
                (effect_clone.borrow_mut())(cx);
            }).detach();
        )*
    }};
}
