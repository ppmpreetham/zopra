## Button

Here is the full API mapping for `<button>` in Zopra (`gpui-rsx`), based directly on the `gpui_kit::component::button::Button` API.

In Zopra, any builder method on the `Button` struct translates directly to an attribute in the `<button>` RSX tag.

### Full API Reference for `<button>`

| Attribute                       | Type                                    | Description                                                                                                                                        |
| :------------------------------ | :-------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------- |
| `id`                            | `&str`                                  | Unique identifier for the button. Generates automatically if omitted, but explicitly providing it is best practice.                                |
| `label`                         | `impl Into<SharedString>`               | The text displayed inside the button. **Note:** Prefer `label="Text"` over placing text as a child to ensure perfect GPUI flex height constraints. |
| `icon`                          | `IconName`, `Spinner`, `ProgressCircle` | Adds an icon. Regular icons are automatically swapped with a spinner if `loading={true}` is set.                                                   |
| `on_click`                      | `move \|event, window, cx\| { ... }`    | Closure executed when the button is clicked.                                                                                                       |
| **Variants (Boolean Flags)**    |                                         |                                                                                                                                                    |
| `primary`                       | `bool`                                  | Applies the primary theme variant.                                                                                                                 |
| `secondary`                     | `bool`                                  | Default variant; applies secondary theme colors.                                                                                                   |
| `danger`                        | `bool`                                  | Applies destructive/danger theme (red).                                                                                                            |
| `warning`                       | `bool`                                  | Applies warning theme (yellow/orange).                                                                                                             |
| `success`                       | `bool`                                  | Applies success theme (green).                                                                                                                     |
| `info`                          | `bool`                                  | Applies info theme (blue).                                                                                                                         |
| `ghost`                         | `bool`                                  | Removes background unless hovered.                                                                                                                 |
| `link`                          | `bool`                                  | Styles the button to look like a hyperlink.                                                                                                        |
| `text`                          | `bool`                                  | Pure text button with no background or borders.                                                                                                    |
| **Modifiers (Boolean Flags)**   |                                         |                                                                                                                                                    |
| `outline`                       | `bool`                                  | Makes the background transparent and applies a border colored by the chosen variant.                                                               |
| `compact`                       | `bool`                                  | Reduces internal padding for a condensed look.                                                                                                     |
| `dropdown_caret`                | `bool`                                  | Adds a chevron-down icon at the far right of the button.                                                                                           |
| **Sizes (Boolean Flags)**       |                                         |                                                                                                                                                    |
| `xsmall`                        | `bool`                                  | Extra small size.                                                                                                                                  |
| `small`                         | `bool`                                  | Small size.                                                                                                                                        |
| `large`                         | `bool`                                  | Large size.                                                                                                                                        |
| _(default)_                     | -                                       | Medium size if no size attribute is provided.                                                                                                      |
| **States**                      |                                         |                                                                                                                                                    |
| `disabled`                      | `bool`                                  | Greys out the button and blocks clicks.                                                                                                            |
| `loading`                       | `bool`                                  | Replaces the icon with a spinner (if present) and blocks clicks.                                                                                   |
| `selected`                      | `bool`                                  | Forces the button into an active/pressed visual state (useful for toggles).                                                                        |
| **Tooltips**                    |                                         |                                                                                                                                                    |
| `tooltip`                       | `&str`                                  | String to show when hovering over the button.                                                                                                      |
| `tooltip_placement`             | `gpui_kit::component::Placement`        | Position of the tooltip (e.g., `Placement::Bottom`).                                                                                               |
| `hoverClass`                    | `&str`                                  | Classes to apply when the button is hovered (must be a static string literal, e.g., `hoverClass="bg-gray-100"`).                                   |
| `activeClass`                   | `&str`                                  | Classes to apply when the button is actively being pressed down.                                                                                   |
| `focusClass`                    | `&str`                                  | Classes to apply when the button has keyboard focus.                                                                                               |
| `on_hover` / `onHover`          | `move \|hovered, window, cx\|`          | Closure triggered when the mouse enters or leaves the button. Provides a `bool` indicating if it's currently hovered.                              |
| `on_mouse_down` / `onMouseDown` | `move \|event, window, cx\|`            | Closure triggered when a mouse button is pressed down on the element.                                                                              |
| `on_mouse_up` / `onMouseUp`     | `move \|event, window, cx\|`            | Closure triggered when a mouse button is released.                                                                                                 |
| `on_mouse_up_out`               | `move \|event, window, cx\|`            | Closure triggered when a mouse button is released _outside_ the button after being pressed inside.                                                 |
| `on_drag` / `onDrag`            | `move \|drag_info, cx\|`                | Triggered when dragging initiates from this button.                                                                                                |
| `focusable`                     | `bool`                                  | Allows the button to receive keyboard focus (usually true by default for buttons, but can be overridden).                                          |
| `hoverable_tooltip`             | `bool`                                  | Allows the tooltip to remain open when the mouse moves from the button onto the tooltip itself.                                                    |
| `tooltip_show_delay`            | `Duration`                              | Delay before the tooltip appears (e.g., `std::time::Duration::from_millis(500)`).                                                                  |
| `group`                         | `&str`                                  | Assigns this element to a hover group (e.g., `group="my-group"`).                                                                                  |
| `group_hover` / `groupHover`    | `("group-name", \|style\| ...)`         | Applies a style refinement closure when a specific group is hovered.                                                                               |

---

### Documentation Examples

Here are two complete, no-placeholder examples you can drop directly into your Zopra documentation.

#### Example 1: State-Driven Loading Button

This example demonstrates how to use the `loading` state. GPUI automatically handles replacing the `Save` icon with an animated spinner when `loading={true}` is passed.

```rust
use zopra::{component, hooks::use_state, view};
use gpui_kit::assets::IconName;

#[component]
pub fn submit_form_button() {
    // 1. Define a state for the loading sequence
    let (is_loading, set_is_loading) = use_state(false);
    let loading_state = *is_loading;

    view! {
        <div class="flex p-4">
            <button
                id="save-changes-btn"
                primary
                large
                // 2. Dynamically change the label text
                label={if loading_state { "Saving Changes..." } else { "Save Changes" }}
                // 3. Assign a static icon; GPUI turns this into a spinner automatically when loading is true
                icon={IconName::Save}
                // 4. Pass the boolean loading state
                loading={loading_state}
                on_click={|| {
                    set_is_loading(true);
                    // In a real app, you would trigger a background task here
                    // and set_is_loading(false) when it finishes.
                }}
            />
        </div>
    }
}
```

#### Example 2: Secondary Action Toolbar

This example shows how to combine variants, modifiers (`outline`, `ghost`), tooltips, and dropdown carets to build a clean secondary toolbar.

```rust
use zopra::{component, view};
use gpui_kit::assets::IconName;
use gpui_kit::component::Placement;

#[component]
pub fn item_toolbar() {
    view! {
        <div class="flex flex-row gap-2 p-2 items-center bg-gray-100 rounded-md">

            // A danger outline button with a tooltip
            <button
                id="delete-item-btn"
                label="Delete"
                danger
                outline
                small
                icon={IconName::Trash}
                tooltip="Permanently delete this item"
                tooltip_placement={Placement::Top}
                on_click={move |_, _, _| println!("Delete clicked")}
            />

            // A ghost button that acts as a dropdown trigger
            <button
                id="options-menu-btn"
                label="Options"
                ghost
                small
                dropdown_caret
                on_click={move |_, _, _| println!("Options clicked")}
            />

        </div>
    }
}
```

### Note on `class` vs `hoverClass`

If you are passing a static string literal or using the `if / else` compile-time optimization inside `class={...}`, you can just use Tailwind-style prefixes instead of `hoverClass`:

```rust
class="bg-black hover:bg-gray-800 focus:outline"
```

But if you are forced to use a dynamic variable (like `class={my_dynamic_string}`), then `hoverClass="..."` is the correct way to add static hover styles.
