use gpui_kit::{App, AsyncApp, Window};

use super::{Signal, use_signal};

/// The current state of an async resource
pub enum ResourceState<T, E> {
    /// The resource is loading, including while a changed dependency is loading
    Loading,
    /// The resource completed successfully
    Ready(T),
    /// The resource failed
    Failed(E),
}

struct Request<D> {
    deps: Option<D>,
    generation: u64,
}

/// Loads an async resource on mount and again whenever `deps` changes
/// Also Exposes its status to the UI!!! :3
///
/// # Example
///
/// ```ignore
/// let user = use_resource(user_id, || async move {
///     fetch_user(user_id).await
/// });
///
/// match user.read() {
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
) -> Signal<ResourceState<T, E>>
where
    D: PartialEq + 'static,
    T: 'static,
    E: 'static,
    F: AsyncFnOnce() -> Result<T, E> + 'static,
{
    let state = use_signal(|| ResourceState::Loading, window, cx);
    let request = window.use_state(cx, |_window, _cx| Request {
        deps: None,
        generation: 0,
    });

    if request.read(cx).deps.as_ref() != Some(&deps) {
        let generation = request.update(cx, |request, _cx| {
            request.deps = Some(deps);
            request.generation += 1;
            request.generation
        });
        // this render already shows the change, so don't schedule another
        state.update_untracked(|state| *state = ResourceState::Loading, cx);

        let request = request.downgrade();
        cx.spawn(async move |cx: &mut AsyncApp| {
            let result = f().await;
            cx.update(|cx| {
                let current = request
                    .upgrade()
                    .is_some_and(|request| request.read(cx).generation == generation);
                if current {
                    let result = match result {
                        Ok(value) => ResourceState::Ready(value),
                        Err(err) => ResourceState::Failed(err),
                    };
                    state.update(|state| *state = result, cx);
                }
            });
        })
        .detach();
    }

    state
}
