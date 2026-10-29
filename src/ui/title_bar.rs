use gpui_kit::component as kit;
use gpui_kit::{App, ClickEvent, ParentElement, StyleRefinement, Window};

use super::styled;
use crate::{Children, component};

/// The window's title bar; its children sit beside the window controls.
/// Open the window with `gpui_kit::component::TitleBar::window_options()` so it can drag the window.
#[component]
pub fn title_bar<F: Fn(&ClickEvent, &mut Window, &mut App) + 'static>(
    on_close_window: Option<F>,
    class: Option<StyleRefinement>,
    children: Children,
) -> kit::TitleBar {
    let mut bar = kit::TitleBar::new().children(children());
    if let Some(on_close_window) = on_close_window {
        bar = bar.on_close_window(on_close_window);
    }
    styled(bar, class)
}
