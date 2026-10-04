# Kbd

```rust
// Basic shortcut
<kbd keys="cmd-shift-p" />

// Zoom out with text styling added
<kbd keys="cmd--" text_color={cx.theme().accent} />

// Without visual styling
<kbd keys="cmd-s" appearance={false} />

// Nested in your command palette text
<div class="flex gap-2 items-center">
    "Find in files:"
    <kbd keys="cmd-shift-f" />
</div>
```

### Under the hood:

If you write `<kbd keys="cmd-shift-p" />`, the compiler now natively hooks into it and automatically generates:

```rust
gpui_kit::component::kbd::Kbd::new(gpui_kit::Keystroke::parse("cmd-shift-p").unwrap())
```

If you don't provide a `keys` attribute, it simply generates a fallback:

```rust
<kbd />
// becomes: Kbd::new(Keystroke::parse("unknown").unwrap())
```
