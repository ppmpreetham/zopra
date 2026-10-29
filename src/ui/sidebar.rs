use gpui_kit::component::sidebar as kit;
use gpui_kit::component::Icon;
use gpui_kit::{AnyElement, App, ClickEvent, ParentElement, SharedString, StyleRefinement, Window};

use super::styled;
use crate::{Children, component};

/// A collapsible sidebar of `<SidebarGroup>`s.
#[component]
pub fn sidebar(
    collapsed: Option<bool>,
    collapsible: Option<kit::SidebarCollapsible>,
    header: Option<AnyElement>,
    footer: Option<AnyElement>,
    class: Option<StyleRefinement>,
    children: Children<kit::SidebarGroup<kit::SidebarMenu>>,
) -> kit::Sidebar<kit::SidebarGroup<kit::SidebarMenu>> {
    let mut sidebar = kit::Sidebar::new("sidebar").children(children());
    if let Some(collapsed) = collapsed {
        sidebar = sidebar.collapsed(collapsed);
    }
    if let Some(collapsible) = collapsible {
        sidebar = sidebar.collapsible(collapsible);
    }
    if let Some(header) = header {
        sidebar = sidebar.header(header);
    }
    if let Some(footer) = footer {
        sidebar = sidebar.footer(footer);
    }
    styled(sidebar, class)
}

#[component]
pub fn sidebar_header(class: Option<StyleRefinement>, children: Children) -> kit::SidebarHeader {
    styled(kit::SidebarHeader::new().children(children()), class)
}

#[component]
pub fn sidebar_footer(class: Option<StyleRefinement>, children: Children) -> kit::SidebarFooter {
    styled(kit::SidebarFooter::new().children(children()), class)
}

/// A labelled `<SidebarMenu>`.
#[component]
pub fn sidebar_group(
    label: SharedString,
    children: Children<kit::SidebarMenu>,
) -> kit::SidebarGroup<kit::SidebarMenu> {
    kit::SidebarGroup::new(label).children(children())
}

#[component]
pub fn sidebar_menu(
    class: Option<StyleRefinement>,
    children: Children<kit::SidebarMenuItem>,
) -> kit::SidebarMenu {
    styled(kit::SidebarMenu::new().children(children()), class)
}

/// One sidebar entry; its children are nested entries.
#[component]
pub fn sidebar_menu_item<F: Fn(&ClickEvent, &mut Window, &mut App) + 'static>(
    label: SharedString,
    icon: Option<Icon>,
    active: Option<bool>,
    disabled: Option<bool>,
    on_click: Option<F>,
    class: Option<StyleRefinement>,
    children: Children<kit::SidebarMenuItem>,
) -> kit::SidebarMenuItem {
    let mut item = kit::SidebarMenuItem::new(label)
        .active(active.unwrap_or_default())
        .disable(disabled.unwrap_or_default())
        .children(children());
    if let Some(icon) = icon {
        item = item.icon(icon);
    }
    if let Some(on_click) = on_click {
        item = item.on_click(on_click);
    }
    styled(item, class)
}

#[component]
pub fn sidebar_toggle_button<F: Fn(&ClickEvent, &mut Window, &mut App) + 'static>(
    collapsed: Option<bool>,
    on_click: Option<F>,
) -> kit::SidebarToggleButton {
    let mut button = kit::SidebarToggleButton::new().collapsed(collapsed.unwrap_or_default());
    if let Some(on_click) = on_click {
        button = button.on_click(on_click);
    }
    button
}


crate::typed_child!(kit::SidebarGroup<kit::SidebarMenu>, kit::SidebarMenu, kit::SidebarMenuItem);
