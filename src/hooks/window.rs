use std::{marker::PhantomData, rc::Rc};

use gpui_kit::{
    AnyWindowHandle, App, AppContext, IntoElement, Render, Window, WindowHandle, WindowOptions,
};

/// Describes a window that can be opened from a component.
///
/// Can open more than one window.
pub struct WindowLauncher<F> {
    options: Rc<F>,
}

impl<F> Clone for WindowLauncher<F> {
    fn clone(&self) -> Self {
        Self {
            options: Rc::clone(&self.options),
        }
    }
}

impl<F> WindowLauncher<F>
where
    F: Fn() -> WindowOptions + 'static,
{
    /// Opens a window and renders its root view with that window's own GPUI context.
    pub fn open<R, E>(&self, render: R, cx: &mut App) -> anyhow::Result<AnyWindowHandle>
    where
        R: FnMut(&mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        let handle: WindowHandle<WindowRoot<R, E>> =
            cx.open_window((self.options)(), |_, cx| {
                cx.new(|_| WindowRoot {
                    render,
                    output: PhantomData,
                })
            })?;

        Ok(handle.into())
    }
}

/// Reusable window launcher for a component.
///
/// Evaluated for each call to `WindowLauncher::open`
pub fn use_window<F>(options: F, _window: &mut Window, _cx: &mut App) -> WindowLauncher<F>
where
    F: Fn() -> WindowOptions + 'static,
{
    WindowLauncher {
        options: Rc::new(options),
    }
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
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut gpui_kit::Context<Self>,
    ) -> impl IntoElement {
        (self.render)(window, cx)
    }
}
