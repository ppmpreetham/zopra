# TitleBar

`zopra::ui::TitleBar` draws the window's title bar; its children sit beside the window controls.

```rust
use zopra::ui::TitleBar;

view! {
    <TitleBar class="border-b border-[#272a2f]">
        <div class="flex items-center gap-2 px-2">
            <img src="/logo.png" class="size-4" />
            <div class="text-xs font-semibold">"My App"</div>
        </div>
    </TitleBar>
}
```

Props: `class`, `on_close_window={|_, window, cx| ..}` (Linux custom close behaviour).

For the OS to let the title bar drag the window and hide its own, open the window with gpui-kit's title bar options:

```rust
let options = WindowOptions {
    window_bounds: Some(bounds),
    ..gpui_kit::component::TitleBar::window_options()
};
```
