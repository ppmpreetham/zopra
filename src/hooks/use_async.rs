use gpui::{AsyncApp, Context, Task, WeakEntity};

pub fn use_async<T, AsyncFn, R>(f: AsyncFn, cx: &mut Context<T>) -> Task<R>
where
    T: 'static,
    AsyncFn: AsyncFnOnce(WeakEntity<T>, &mut AsyncApp) -> R + 'static,
    R: 'static,
{
    cx.spawn(f)
}
