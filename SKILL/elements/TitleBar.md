Here is how the `TitleBar` API will natively map into your Zopra `gpui-rsx` React-style syntax!

Since `TitleBar` implements GPUI's `Styled` trait and supports arbitrary children, the mapping is beautifully clean.

### 1. Basic Title Bar

**Rust:** `TitleBar::new().child(div().child("My Application"))`
**Zopra RSX:**

```rust
<TitleBar>
    <div>"My Application"</div>
</TitleBar>
```

### 2. Title Bar with Custom Content (Flex, Badges, Buttons)

Any layout wrapping works natively inside the tags:

```rust
<TitleBar>
    <div class="flex items-center gap-3">
        "App Name"
        // (Assuming you have a <Badge> component)
        <Badge count={5} />
    </div>
    <div class="flex items-center gap-2">
        <Button label="settings" icon={IconName::Settings} />
        <Button label="profile" icon={IconName::User} />
    </div>
</TitleBar>
```

### 3. Styled Title Bar (Custom height, colors, borders)

Because it implements `Styled`, we can pass arbitrary tailwind classes or direct GPUI style attributes:

```rust
<TitleBar
    h={gpui_kit::px(40.)}
    bg={cx.theme().accent}
    border_color={cx.theme().accent_border}
    class="border-b-2"
>
    <div text_color={cx.theme().accent_foreground} class="font-semibold">
        "Custom Theme App"
    </div>
</TitleBar>
```

### 4. Linux Custom Window Controls (`on_close_window`)

```rust
<TitleBar
    on_close_window={|_, window, cx| {
        window.push_notification("Saving before close...", cx);
        window.remove_window();
    }}
>
    <div>"Custom Close Behavior"</div>
</TitleBar>
```

---

### ⚠️ Important setup required in `main.rs`

The `<TitleBar>` UI component handles the rendering, but to make sure your OS actually lets it own dragging and double-clicking (and turns off the default OS title bar), you have to modify your `main.rs` window spawn options.

When you open your window in `syzygy/src/main.rs`, we will need to update it to use `TitleBar::window_options()` like this:

```rust
let window_size = WindowOptions {
    window_bounds: Some(config.window_size),
    ..TitleBar::window_options() // <--- This is required for drag/double-click to work!
};
```
