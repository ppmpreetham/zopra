# `use_window`

`use_window(|| WindowOptions { ... })` creates a reusable launcher for another GPUI window. Call `open` from an event handler; it creates a window with those options and renders the supplied view using the new window's own context. `open` returns `anyhow::Result<AnyWindowHandle>`.

```rust
let browser = use_window(|| WindowOptions {
    window_bounds: Some(bounds),
    ..Default::default()
});

view! {
    <button onClick={|| {
        browser.open(|| view! { <Browser /> });
    }}>
        "Open browser"
    </button>
}
```

The options factory is retained and called for each `open`. Keep the launcher in the component scope so event handlers can capture it.
