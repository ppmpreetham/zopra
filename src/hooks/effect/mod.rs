use gpui::{Context, Entity, Subscription};
use std::{cell::RefCell, rc::Rc};

pub fn create_effect<T: 'static, V: 'static>(
    cx: &mut Context<V>,
    deps: &[&Entity<T>],
    mut effect: impl FnMut(&mut Context<V>) + 'static,
) -> Vec<Subscription> {
    let effect = Rc::new(RefCell::new(effect));

    deps.iter()
        .map(|dep| {
            let effect = effect.clone();
            cx.observe(dep, move |_view, _entity, cx| {
                (effect.borrow_mut())(cx);
            })
        })
        .collect()
}
