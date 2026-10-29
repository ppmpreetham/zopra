use gpui_kit::component::tab as kit;
use gpui_kit::component::{Icon, Sizable, Size};
use gpui_kit::{AnyElement, App, ParentElement, Pixels, SharedString, StyleRefinement, Window};

use super::styled;
use crate::{Children, component};

/// A row of tabs; `on_click` gets the clicked tab's index.
#[component]
pub fn tab_bar<F: Fn(&usize, &mut Window, &mut App) + 'static>(
    selected_index: Option<usize>,
    on_click: Option<F>,
    underline: Option<bool>,
    pill: Option<bool>,
    outline: Option<bool>,
    segmented: Option<bool>,
    size: Option<Size>,
    max_width: Option<Pixels>,
    menu: Option<bool>,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
    class: Option<StyleRefinement>,
    children: Children<kit::Tab>,
) -> kit::TabBar {
    let mut bar = kit::TabBar::new("tab-bar").children(children());
    if let Some(index) = selected_index {
        bar = bar.selected_index(index);
    }
    if let Some(on_click) = on_click {
        bar = bar.on_click(on_click);
    }
    if underline.unwrap_or_default() {
        bar = bar.underline();
    }
    if pill.unwrap_or_default() {
        bar = bar.pill();
    }
    if outline.unwrap_or_default() {
        bar = bar.outline();
    }
    if segmented.unwrap_or_default() {
        bar = bar.segmented();
    }
    if let Some(size) = size {
        bar = bar.with_size(size);
    }
    if let Some(width) = max_width {
        bar = bar.max_width(width);
    }
    if let Some(menu) = menu {
        bar = bar.menu(menu);
    }
    if let Some(prefix) = prefix {
        bar = bar.prefix(prefix);
    }
    if let Some(suffix) = suffix {
        bar = bar.suffix(suffix);
    }
    styled(bar, class)
}

#[component]
pub fn tab(
    label: Option<SharedString>,
    icon: Option<Icon>,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
    disabled: Option<bool>,
    children: Children,
) -> kit::Tab {
    let mut tab = kit::Tab::new().disabled(disabled.unwrap_or_default()).children(children());
    if let Some(label) = label {
        tab = tab.label(label);
    }
    if let Some(icon) = icon {
        tab = tab.icon(icon);
    }
    if let Some(prefix) = prefix {
        tab = tab.prefix(prefix);
    }
    if let Some(suffix) = suffix {
        tab = tab.suffix(suffix);
    }
    tab
}

crate::typed_child!(kit::Tab);
