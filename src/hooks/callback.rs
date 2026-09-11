use gpui::{App, Context, Entity, Window};

pub fn use_callback<E: 'static, T: 'static>(
    entity: &Entity<T>,
    f: impl Fn(&mut T, &E, &mut Window, &mut Context<T>) + 'static,
) -> impl Fn(&E, &mut Window, &mut App) + 'static {
    let entity = entity.downgrade();
    move |e: &E, window: &mut Window, cx: &mut App| {
        entity.update(cx, |t, cx| f(t, e, window, cx)).ok();
    }
}
