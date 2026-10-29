# DropdownButton

A split button: a primary action on the left and a menu trigger on the right.

- `<DropdownButton>` (`zopra::ui`) owns the menu and the styles both halves share.
- The inner `<button>` owns the action: label, tooltip, click handler.

### Import

```rust
use gpui_kit::component::menu::PopupMenuItem;
use zopra::ui::DropdownButton;
```

### Basic Usage

Pass the primary action as `button` and build the menu in `dropdown_menu`.

```rust
view! {
    <DropdownButton
        id="deploy-dropdown"
        button={view! {
            <button id="deploy-btn" label="Deploy" on_click={|| deploy()} />
        }}
        dropdown_menu={|menu, _window, _cx| {
            menu.item(PopupMenuItem::new("Staging").on_click(|_, _, cx| { /* ... */ }))
                .separator()
                .item(PopupMenuItem::new("Production").on_click(|_, _, cx| { /* ... */ }))
        }}
    />
}
```

---

## API Reference

### `<DropdownButton>` Props

| Prop | Type | Description |
| :--- | :--- | :--- |
| `button` | `Button` | **Required.** The primary action, usually `view! { <button ... /> }`. |
| `dropdown_menu` | closure | **Required.** `\|menu, window, cx\|` builds and returns the menu. |
| `id` | `&str` | Identifies the button; defaults to one per tag. |
| `variant` | `ButtonVariant` | Colours both halves, e.g. `ButtonVariant::Primary`. |
| `size` | `Size` | Sizes both halves, e.g. `Size::Small`. |
| `outline` | flag | Outlined style. |
| `disabled` | `bool` | Disables both halves. |
| `class` | classes | Extra styles. |

### Inner `<button>` Properties

These properties belong to the inner button and dictate the primary click target's behavior.

| Attribute  | Type       | Description                                                                      |
| :--------- | :--------- | :------------------------------------------------------------------------------- |
| `id`       | `&str`     | **Required.** Unique identifier for the primary action half.                     |
| `label`    | `&str`     | Text displayed inside the primary button.                                        |
| `icon`     | `IconName` | An icon rendered next to the label.                                              |
| `tooltip`  | `&str`     | Text displayed when hovering the primary action.                                 |
| `loading`  | `bool`     | If true, replaces the icon with a spinner and disables the button.               |
| `on_click` | `Closure`  | `\|event, window, cx\|` executed when the primary action (left half) is clicked. |

---

## Patterns & Examples

### 1. Variants & Overrides

If you leave a variant off the inner button, it inherits the shared style from the `<DropdownButton>`. However, you can explicitly override styles on the inner button to make it look different from the dropdown trigger.

```rust
view! {
    // Both halves will be compact, but the dropdown trigger inherits the ghost variant
    <DropdownButton
        id="save-dropdown"
        compact
        ghost
        button={view! {
            // Overrides the ghost variant on the left half to be a primary solid button
            <button
                id="save-btn"
                primary
                label="Save"
            />
        }}
        dropdown_menu={|menu, _, _| { /* ... */ }}
    />
}
```

### 2. State-Driven Dropdown (Local Enum)

Use `PopupMenuItem` closures to drive a reactive local state via a standard Zopra `use_state` hook, instantly updating the primary button based on dropdown selections.

```rust
use strum_macros::FromRepr;
use zopra::{view, component, hooks::use_state};

#[derive(Clone, Copy, Debug, PartialEq, Eq, FromRepr)]
#[repr(usize)]
pub enum ActionTab {
    SaveDraft = 0,
    ExportJson = 1,
}

#[component]
pub fn dropdown_with_enum() {
    let (active_action, set_active_action) = use_state(ActionTab::SaveDraft);
    let action_val = active_action();

    let button_label = match action_val {
        ActionTab::SaveDraft => "Save Draft",
        ActionTab::ExportJson => "Export JSON",
    };

    view! {
        <DropdownButton
            id="action-dropdown"
            primary
            button={view! {
                <button
                    id="action-btn"
                    label={button_label}
                    on_click={move |_, _, _| println!("Executing: {:?}", action_val)}
                />
            }}
            dropdown_menu={move |menu, _window, _cx| {
                menu.item(
                        PopupMenuItem::new("Save Draft")
                            .on_click(move |_, _, cx| set_active_action(ActionTab::SaveDraft, cx))
                    )
                    .item(
                        PopupMenuItem::new("Export JSON")
                            .on_click(move |_, _, cx| set_active_action(ActionTab::ExportJson, cx))
                    )
            }}
        />
    }
}
```

### 3. Custom Popover Anchoring

If the dropdown is located near the right edge of the screen, you may want the menu to anchor to the `BottomRight` rather than the default `BottomLeft`.

```rust
view! {
    <DropdownButton
        id="anchor-dropdown"
        button={view! { <button id="btn" label="Actions" /> }}
        dropdown_menu_with_anchor={(
            gpui_kit::Anchor::BottomRight,
            |menu, _, _| {
                menu.item(PopupMenuItem::new("Option 1"))
            }
        )}
    />
}
```

### Full Example

```rust
use gpui_kit::*;
use gpui_kit::component::button::{Button, DropdownButton};
use gpui_kit::component::menu::PopupMenuItem;
use zopra::{view, component, hooks::use_state};
use strum_macros::FromRepr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, FromRepr)]
#[repr(usize)]
pub enum ActionTab {
    SaveDraft = 0,
    ExportJson = 1,
    Discard = 2,
}

#[component]
pub fn dropdown_with_enum() {
    // 1. Initialize state just like the tabs
    let (active_action, set_active_action) = use_state(ActionTab::SaveDraft);
    let action_val = active_action();

    // 2. Reactively derive the button label based on the current state
    let button_label = match action_val {
        ActionTab::SaveDraft => "Save Draft",
        ActionTab::ExportJson => "Export JSON",
        ActionTab::Discard => "Discard Changes",
    };

    view! {
        <DropdownButton
            id="action-dropdown"
            primary
            button={view! {
                <button
                    id="action-btn"
                    label={button_label}
                    on_click={move |_, _, _| println!("Executing primary action: {:?}", action_val)}
                />
            }}
            // 3. Render the menu options and map their clicks to state updates
            dropdown_menu={move |menu, _window, _cx| {
                menu.item(
                        PopupMenuItem::new("Save Draft")
                            .on_click(move |_, _, cx| set_active_action(ActionTab::SaveDraft, cx))
                    )
                    .item(
                        PopupMenuItem::new("Export JSON")
                            .on_click(move |_, _, cx| set_active_action(ActionTab::ExportJson, cx))
                    )
                    .separator()
                    .item(
                        PopupMenuItem::new("Discard Changes")
                            .on_click(move |_, _, cx| set_active_action(ActionTab::Discard, cx))
                    )
            }}
        />
    }
}
```
