use std::ops::Range;

use gpui_kit::component::resizable as kit;
use gpui_kit::{App, ElementId, Entity, ParentElement, Pixels, StyleRefinement, Window};

use super::styled;
use crate::{Children, component};

/// Panels side by side (or stacked with `vertical`) with draggable dividers.
#[component]
pub fn resizable<F: Fn(&Entity<kit::ResizableState>, &mut Window, &mut App) + 'static>(
    id: Option<ElementId>,
    vertical: Option<bool>,
    horizontal: Option<bool>,
    state: Option<Entity<kit::ResizableState>>,
    on_resize: Option<F>,
    children: Children<kit::ResizablePanel>,
) -> kit::ResizablePanelGroup {
    let id = id.unwrap_or_else(|| "resizable".into());
    let vertical = vertical.unwrap_or_default() && !horizontal.unwrap_or_default();
    let mut group = if vertical { kit::v_resizable(id) } else { kit::h_resizable(id) };
    group = group.children(children());
    if let Some(state) = &state {
        group = group.with_state(state);
    }
    if let Some(on_resize) = on_resize {
        group = group.on_resize(on_resize);
    }
    group
}

#[component]
pub fn resizable_panel(
    size: Option<Pixels>,
    size_range: Option<Range<Pixels>>,
    visible: Option<bool>,
    class: Option<StyleRefinement>,
    children: Children,
) -> kit::ResizablePanel {
    let mut panel = kit::resizable_panel().visible(visible.unwrap_or(true)).children(children());
    if let Some(size) = size {
        panel = panel.size(size);
    }
    if let Some(range) = size_range {
        panel = panel.size_range(range);
    }
    styled(panel, class)
}

crate::typed_child!(kit::ResizablePanel);
