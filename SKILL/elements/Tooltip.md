# Tooltip

A versatile tooltip component that displays helpful information when hovering over or focusing on elements. Supports text content, custom elements, keyboard shortcuts, different trigger methods, and positioning options.

### 1. Basic Text Tooltips (The Easy Way)

For standard components like `<Button>`, `<Input>`, or `<Checkbox>`, the compiler will just pass the string directly to their built-in `.tooltip()` methods:

```rust
// On a Button
<Button label="Save" tooltip="Save the current document" />

// On an Input
<Input placeholder="Password" tooltip="Must be at least 8 characters" />
```

For raw HTML tags like `<div>`, we will patch the compiler to automatically wrap the string in GPUI's `Tooltip::new(...).build(window, cx)` boilerplate for you behind the scenes:

```rust
<div id="basic-tooltip" tooltip="This is a helpful tooltip">
    "Hover me"
</div>
```

### 2. Tooltips with Keybinding Actions

For components that support actions, you can pass the tuple of arguments straight into the attribute:

```rust
<Button
    label="Save"
    icon={IconName::Save}
    // Passes: (text, action, context)
    tooltip_with_action={"Save the current document", &SaveDocument, Some("Editor")}
/>
```

### 3. Rich / Custom Element Tooltips

When you want a complex tooltip with its own layout, colors, and icons, you can pass a closure into the `tooltip` attribute. Inside it, you can safely nest another `view! {}` macro to design the tooltip using RSX!

```rust
<div
    id="rich-tooltip"
    tooltip={|window, cx| {
        Tooltip::element(|_, cx| {
            // Build the inside of the tooltip using RSX!
            view! {
                <div class="flex gap-2 items-center">
                    <icon name={IconName::Info} />
                    <div text_color={cx.theme().muted_foreground}>"Last login: 2 hours ago"</div>
                    <div text_color={cx.theme().success}>"Status: Active"</div>
                </div>
            }.into_any_element()
        }).build(window, cx)
    }}
>
    "Hover for rich content"
</div>
```

### 4. Custom Styling on a Text Tooltip

If you just want a text tooltip but need to override the theme colors, you can chain the style methods before calling `.build()`:

```rust
<div
    tooltip={|window, cx| {
        Tooltip::new("Custom styled tooltip")
            .bg(cx.theme().accent)
            .text_color(cx.theme().accent_foreground)
            .build(window, cx)
    }}
>
    "Hover me"
</div>
```
