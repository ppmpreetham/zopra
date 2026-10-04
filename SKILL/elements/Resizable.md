# Resizable

```rust
use gpui_kit::component::resizable::*;
use gpui_kit::px;
use zopra::{component, view};

#[component]
pub fn interactive_layout() {
    view! {
        // A stable id lets the component preserve its own layout state.
        <Resizable id="main-layout" horizontal>

            <ResizablePanel
                size={px(250.0)}
                size_range={px(200.0)..px(400.0)}
            >
                <div class="bg-[#141517] h-full p-4">"Left Sidebar"</div>
            </ResizablePanel>

            <ResizablePanel>
                // Nested split panes use their own stable id.
                <Resizable id="editor-split" vertical>

                    <ResizablePanel>
                        <div class="bg-[#1e1e1e] h-full p-6 text-white font-mono">
                            "Editor Top"
                        </div>
                    </ResizablePanel>

                    <ResizablePanel size={px(200.0)}>
                        <div class="bg-[#0d0d0d] h-full p-4 text-green-400 font-mono">
                            "Terminal Bottom"
                        </div>
                    </ResizablePanel>

                </Resizable>
            </ResizablePanel>

        </Resizable>
    }
}
```

- `<Resizable horizontal>` ➔ `gpui_kit::component::resizable::h_resizable`
- `<Resizable vertical>` ➔ `gpui_kit::component::resizable::v_resizable`
- `<ResizablePanel>` ➔ `gpui_kit::component::resizable::resizable_panel()`
