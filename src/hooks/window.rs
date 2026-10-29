use std::marker::PhantomData;

use gpui_kit::{
    AnyWindowHandle, App, AppContext, IntoElement, Render, Window, WindowHandle, WindowOptions,
};

use super::{Signal, use_signal};

/// A `Copy` handle that opens windows from a component, it can open more than one.
pub struct WindowLauncher<F>(Signal<F>);

impl<F> Clone for WindowLauncher<F> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<F> Copy for WindowLauncher<F> {}

impl<F> WindowLauncher<F>
where
    F: Fn() -> WindowOptions + 'static,
{
    /// Opens a window and renders its root view with that window's own GPUI context.
    pub fn open<R, E>(self, render: R, cx: &mut App) -> anyhow::Result<AnyWindowHandle>
    where
        R: FnMut(&mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        let options = (self.0.read(cx))();
        let handle: WindowHandle<WindowRoot<R, E>> = cx.open_window(options, |_, cx| {
            cx.new(|_| WindowRoot {
                render,
                output: PhantomData,
            })
        })?;
        Ok(handle.into())
    }
}

/// Reusable window launcher for a component. `options` runs on every [`WindowLauncher::open`].
#[track_caller]
pub fn use_window<F>(options: F, window: &mut Window, cx: &mut App) -> WindowLauncher<F>
where
    F: Fn() -> WindowOptions + 'static,
{
    WindowLauncher(use_signal(|| options, window, cx))
}

struct WindowRoot<F, E> {
    render: F,
    output: PhantomData<fn() -> E>,
}

impl<F, E> Render for WindowRoot<F, E>
where
    F: FnMut(&mut Window, &mut App) -> E + 'static,
    E: IntoElement + 'static,
{
    fn render(&mut self, window: &mut Window, cx: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        (self.render)(window, cx)
    }
}
