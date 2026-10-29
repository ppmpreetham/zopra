# Resizable

`zopra::ui::Resizable` lays its `<ResizablePanel>`s side by side, or stacked with `vertical`, with draggable dividers between them.

```rust
use gpui_kit::px;
use zopra::ui::{Resizable, ResizablePanel};

#[component]
pub fn interactive_layout() {
    view! {
        <Resizable id="main-layout">
            <ResizablePanel size={px(250.)} size_range={px(200.)..px(400.)}>
                <div class="bg-[#141517] h-full p-4">"Left Sidebar"</div>
            </ResizablePanel>

            <ResizablePanel>
                <Resizable id="editor-split" vertical>
                    <ResizablePanel>
                        <div class="bg-[#1e1e1e] h-full p-6 text-white font-mono">"Editor"</div>
                    </ResizablePanel>
                    <ResizablePanel size={px(200.)}>
                        <div class="bg-[#0d0d0d] h-full p-4 text-green-400 font-mono">"Terminal"</div>
                    </ResizablePanel>
                </Resizable>
            </ResizablePanel>
        </Resizable>
    }
}
```

| Tag | Props |
|---|---|
| `<Resizable>` | `id`, `vertical` (default horizontal), `state` (an `Entity<ResizableState>` to share or read sizes), `on_resize={\|state, window, cx\| ..}` |
| `<ResizablePanel>` | `size`, `size_range`, `visible`, `class`; children are its content |

A `<Resizable>`'s children must be `<ResizablePanel>`s; anything else is a compile error.
