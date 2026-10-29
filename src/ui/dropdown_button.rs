use gpui_kit::component::button::{self as kit, ButtonVariant, ButtonVariants};
use gpui_kit::component::menu::PopupMenu;
use gpui_kit::component::{Disableable, Sizable, Size};
use gpui_kit::{Context, ElementId, StyleRefinement, Window};

use super::styled;
use crate::component;

/// A button with a menu beside it. `button` is the main action, `dropdown_menu` builds the
/// menu, and `variant` / `size` / `disabled` style both halves.
#[component]
pub fn dropdown_button<M>(
    button: kit::Button,
    dropdown_menu: M,
    id: Option<ElementId>,
    variant: Option<ButtonVariant>,
    size: Option<Size>,
    outline: Option<bool>,
    disabled: Option<bool>,
    class: Option<StyleRefinement>,
) -> kit::DropdownButton
where
    M: Fn(PopupMenu, &mut Window, &mut Context<PopupMenu>) -> PopupMenu + 'static,
{
    let mut dropdown = kit::DropdownButton::new(id.unwrap_or_else(|| "dropdown-button".into()))
        .button(button)
        .dropdown_menu(dropdown_menu)
        .disabled(disabled.unwrap_or_default());
    if let Some(variant) = variant {
        dropdown = dropdown.with_variant(variant);
    }
    if let Some(size) = size {
        dropdown = dropdown.with_size(size);
    }
    if outline.unwrap_or_default() {
        dropdown = dropdown.outline();
    }
    styled(dropdown, class)
}
