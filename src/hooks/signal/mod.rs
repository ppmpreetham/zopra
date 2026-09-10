use gpui::{App, AppContext, Entity};

/// Creates a signal
///
/// # Example
///
/// ```rust
/// let (count, set_count) = use_state(0);
///
/// set_count(1);
///
/// let value = count();
/// assert_eq!(value, 1);
/// ```
pub fn use_state<T: 'static + Clone>(
    initial: T,
    cx: &mut App,
) -> (impl Fn(&App) -> T + Clone, impl Fn(T, &mut App) + Clone) {
    let entity: Entity<T> = cx.new(|_cx| initial);

    let getter = {
        let entity = entity.clone();
        move |cx: &App| entity.read(cx).clone()
    };

    let setter = move |value: T, cx: &mut App| {
        entity.update(cx, |state, cx| {
            *state = value;
            cx.notify();
        });
    };

    (getter, setter)
}
