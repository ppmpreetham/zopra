use gpui_kit::{AsyncApp, Context, Task, WeakEntity};

/// Spawns an async task bound to the component entity and returns its task handle.
///
/// # Example
///
/// ```ignore
/// use_async(async move |_weak_entity, async_cx| {
///     let value = load().await;
///     async_cx.update(|cx| set_value(value, cx)).ok();
/// });
/// ```
pub fn use_async<T, AsyncFn, R>(f: AsyncFn, cx: &mut Context<T>) -> Task<R>
where
    T: 'static,
    AsyncFn: AsyncFnOnce(WeakEntity<T>, &mut AsyncApp) -> R + 'static,
    R: 'static,
{
    cx.spawn(f)
}
