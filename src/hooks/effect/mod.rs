#[macro_export]
macro_rules! use_effect {
    ($cx:expr, [$($dep:expr),* $(,)?], $effect:expr) => {{
        let effect = std::rc::Rc::new(std::cell::RefCell::new($effect));
        let mut subs = Vec::new();

        $(
            let effect_clone = effect.clone();
            subs.push($cx.observe(&$dep, move |_view, _entity, cx| {
                (effect_clone.borrow_mut())(cx);
            }));
        )*

        subs
    }};
}
