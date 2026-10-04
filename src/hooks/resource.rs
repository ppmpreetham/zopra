use std::rc::Rc;
use gpui_kit::{App, Entity, Window};
use super::Snap;

/// The current state of an async resource
pub enum ResourceState<T, E> {
    /// The resource is loading, including while a changed dependency is loading
    Loading,
    /// The resource completed successfully
    Ready(T),
    /// The resource failed
    Failed(E),
}

struct ResourceSlot<T, E> {
    generation: u64,
    state: Rc<ResourceState<T, E>>,
}

/// Loads an async resource on mount and again whenever `deps` changes
/// Also Exposes its status to the UI!!! :3
///
/// # Example
///
/// ```ignore
/// let user = use_resource([user_id], || async move {
///     fetch_user(user_id).await
/// });
///
/// match &*user {
///     ResourceState::Loading => { /* render a loading state */ }
///     ResourceState::Ready(user) => { /* render the user */ }
///     ResourceState::Failed(error) => { /* render the error */ }
/// }
/// ```
#[track_caller]
pub fn use_resource<D, T, E, F>(
    deps: D,
    f: F,
    window: &mut Window,
    cx: &mut App,
) -> Snap<ResourceState<T, E>>
where
    D: Clone + PartialEq + 'static,
    T: 'static,
    E: 'static,
    F: AsyncFnOnce() -> Result<T, E> + 'static,
{
    let previous: Entity<Option<D>> = window.use_state(cx, |_window, _cx| None);
    let state: Entity<ResourceSlot<T, E>> = window.use_state(cx, |_window, _cx| ResourceSlot {
        generation: 0,
        state: Rc::new(ResourceState::Loading),
    });

    let changed = previous.read(cx).as_ref() != Some(&deps);
    if changed {
        previous.update(cx, |old, _cx| *old = Some(deps));
        let generation = state.update(cx, |slot, _cx| {
            slot.generation = slot.generation.wrapping_add(1);
            slot.state = Rc::new(ResourceState::Loading);
            slot.generation
        });
        let writer = state.downgrade();
        cx.spawn(async move |cx: &mut gpui_kit::AsyncApp| {
            let result = f().await;
            _ = writer.update(cx, |slot, cx| {
                if slot.generation == generation {
                    slot.state = Rc::new(match result {
                        Ok(value) => ResourceState::Ready(value),
                        Err(err) => ResourceState::Failed(err),
                    });
                    cx.notify();
                }
            });
        })
        .detach();
    }

    Snap(state.read(cx).state.clone())
}
