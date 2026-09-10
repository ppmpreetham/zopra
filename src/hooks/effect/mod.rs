use gpui::{App, Context, Entity, Subscription};

pub fn create_effect<T: 'static, V: 'static>(
    entity: &Entity<T>,
    cx: &mut Context<V>,
    mut effect: impl FnMut(&T, &App) + 'static,
) -> Subscription {
    cx.observe(entity, move |_this, changed, cx| {
        effect(changed.read(cx), cx);
    })
}
