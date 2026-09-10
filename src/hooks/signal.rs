use gpui::{App, Entity, Window};

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
#[track_caller]
pub fn use_state<T: 'static + Clone>(
    initial: T,
    window: &mut Window,
    cx: &mut App,
) -> (impl Fn(&App) -> T + Clone, impl Fn(T, &mut App) + Clone) {
    let entity: Entity<T> = window.use_state(cx, |_window, _cx| initial);

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
