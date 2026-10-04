Here is the comprehensive documentation for the `<DropdownButton>` component, completely tailored to your `gpui-rsx` setup. You can copy and paste this entire block directly into your project's docs or README.

---

# DropdownButton

A split-button component that combines a primary action button on the left with a dropdown menu trigger on the right.

In `gpui-rsx`, the component is structurally split:

1. The outer `<DropdownButton>` controls the menu behavior and **shared** styles (size, colors).
2. The inner `<button>` controls the **action-specific** content (labels, tooltips, click handlers).

### Import

```rust
use gpui_kit::component::button::{Button, DropdownButton};
use gpui_kit::component::menu::PopupMenuItem;
```

### Basic Usage

Pass your primary action via the `button` attribute, and build your menu via the `dropdown_menu` closure.

```rust
view! {
    <DropdownButton
        id="deploy-dropdown"
        button={view! {
            <button
                id="deploy-btn"
                label="Deploy"
                on_click={|_, _, _| println!("Deploying...")}
            />
        }}
        dropdown_menu={|menu, _window, _cx| {
            menu.item(PopupMenuItem::new("Staging").on_click(|_,_,cx| { /* ... */ }))
                .separator()
                .item(PopupMenuItem::new("Production").on_click(|_,_,cx| { /* ... */ }))
        }}
    />
}
```

---

## API Reference

### Outer `<DropdownButton>` Properties

These properties define the shell, the menu, and styles applied to _both_ halves of the button simultaneously.

| Attribute                   | Type      | Description                                                                                                              |
| :-------------------------- | :-------- | :----------------------------------------------------------------------------------------------------------------------- |
| `id`                        | `&str`    | **Required.** Unique identifier for the component.                                                                       |
| `button`                    | `Element` | **Required.** The primary action button. Usually passed as `view! { <button ... /> }`.                                   |
| `dropdown_menu`             | `Closure` | A closure `\|menu, window, cx\|` that builds and returns the dropdown options.                                           |
| `dropdown_menu_with_anchor` | `Tuple`   | `(Anchor, Closure)`. Positions the popup menu relative to a custom anchor point (e.g., `gpui_kit::Anchor::BottomRight`). |

**Shared Styling Flags:**
Apply these directly to the `<DropdownButton>` to style both halves:

- **Colors**: `primary`, `ghost`, `danger`, `transparent`
- **Sizes**: `small`, `large`, `compact`
- **States**: `disabled`

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
