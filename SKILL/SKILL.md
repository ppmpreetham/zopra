---
name: zopra-dev
description: Expert guide for building reactive desktop applications in Rust using Zopra and GPUI. Make sure to use this skill whenever the user mentions Zopra, GPUI, rsx, the `view!` macro, reactive state, tailwind like classes, `use_state`, or when they are building or modifying UI components in a Zopra project.
---

# Zopra & GPUI-RSX Master Development Guide

Zopra is a reactive wrapper around GPUI that enables writing high-performance desktop apps using a React-like paradigm in Rust. It utilizes the `view!` macro to generate GPUI layout trees statically at compile time, heavily leaning on Tailwind-like CSS syntax.

This document contains deep architectural knowledge of how `gpui-rsx` maps to GPUI, the exact limits of the parsing engine, and how to write production-ready Zopra components.

---

## 1. Components & `view!` Syntax

Components use `#[component]` and return an element; `view!` is the usual way to build it. The macro adds `window` and `cx` to the component function and injects them into supported hooks. Call hooks at the top level of the component body. Inside another `view!`, use the generated PascalCase tag and pass props as attributes. Component props can be owned or borrowed; `&str` and `&T` props are borrowed for the immediate render.

```rust
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use zopra::{component, hooks::use_state, view, cn};

#[component]
pub fn counter_button(initial_count: i32, label: String) {
    let (count, set_count) = use_state(initial_count);

    view! {
        <div
            class="flex items-center justify-center bg-[#ededed] p-4 rounded-md cursor-pointer hover:bg-[#d4d4d8]"
            onClick={|| set_count(*count + 1)}
        >
            {label} ": " {count}
        </div>
    }
}
```

### Direct GPUI Attributes

You can bypass `class="..."` and pass variables directly to GPUI methods using attributes!

```rust
let my_color = rgb(0x3b82f6);
<div bg={my_color} flex px_4 />
```

### Control Flow in `view!`

`gpui-rsx` natively supports JSX-like control flow directly inside the macro:

**Conditional Rendering (`when` attribute):**

```rust
// Standard conditional:
<div when={is_visible} class="text-white">"I am visible!"</div>

// Closure conditional (gives you full access to the GPUI builder):
<div when={(is_active, |this| this.bg(rgb(0x3b82f6)))} />
```

**Loops & `key` (CRITICAL):**
If you generate interactive elements dynamically inside a `for` loop, you **MUST** provide a unique `key={expr}` attribute. This composites with the auto-injected ID so GPUI tracks state correctly.

```rust
<div class="flex flex-col">
    { for item in items {
        <div key={item.id} onClick={...} class="p-2 text-white">{item.name}</div>
    } }
</div>
```

**Fragments (`<>`):**
Fragments erase their children into `AnyElement`, so mixed element types are allowed:

```rust
view! {
    <>
        {div()}
        <MyComponent />
    </>
}
```

_Tip: Prefer wrapping mixed children in a parent `<div/>` instead of using a fragment._

---

## 2. Event Handling & State

Interactive GPUI events are mapped to camelCase or snake_case attributes. In a `#[component]` `view!`, the macro normalizes supported event closures and supplies the event parameters, `window`, and `cx`. You can write a short handler such as `onClick={|| set_count(*count + 1)}`. Use explicit parameters when a callback needs the event or GPUI context, or when using a GPUI callback outside a Zopra component.

**Common Event Attributes:**

- `onClick` / `on_click`, `onChange` / `on_change`, `onAuxClick`
- `onHover` (Triggers on mouse enter/leave, boolean `is_hovered`)
- `onMouseDown`, `onMouseUp`, `onMouseUpOut`
- `onMouseExit`, `onMousePressure`, `captureMousePressure`
- `onDrag`, `onDragMove`
- `onKeyDown`, `captureKeyDown`
- `onPinch`, `capturePinch`
- `onBoxedAction`, `onA11yAction`, `onResize` / `on_resize`

The normalizer covers a defined set of event attributes; it does not change arbitrary GPUI closures or every capture callback. Check the GPUI and `gpui-rsx` APIs when using less common event attributes.

---

## 3. Supported Elements & Tags

`gpui-rsx` maps standard HTML tags to specific GPUI components natively:

- `<div>`: Standard layout container (`gpui::div()`).
- `<kbd>`: Styled keyboard shortcuts.
- `<img>`: Requires a `src` attribute. `<img src="path/to/asset.png" class="size-20" />`
- `<svg>`: Requires a `src` attribute. `<svg src="path/to/icon.svg" />`
- `<canvas>`: Requires `prepaint` and `paint` closures. `<canvas prepaint={...} paint={...} />`
- `<input>` / `<textarea>`: Requires a `state` attribute passing the GPUI input model. `<input state={text_state} />`

---

## 4. The Tailwind Engine (`gpui-rsx`)

`gpui-rsx` translates Tailwind strings into native GPUI method chains (e.g., `flex items-center` -> `el.flex().items_center()`).

### Supported Syntax & Arbitrary Values

- **Arbitrary Colors:** Full support for hex strings. `bg-[#0a0a0a]`, `text-[#ededed]`, `border-[#ffffff]`.
- **Arbitrary Lengths:** You can bypass the spacing scale entirely using bracket syntax: `w-[150px]`, `w-[5rem]`, `p-[10px]`.
- **Spacing Scale:** Standard tailwind spacing (e.g. `p-4`, `w-48`) natively maps to a `0.25rem` multiplier. So `w-48` evaluates exactly to 12rem (which GPUI translates into pixels).
- **Core Layout:** `flex`, `flex-col`, `flex-row`, `items-center`, `justify-center`, `justify-between`, `size-full`, `w-full`, `h-full`, `absolute`, `relative`.
- **Borders:** `border` (sets 1px width), `border-t`, `border-transparent`, `border-white`, `rounded-md`, `rounded-full`.
- **Interaction Pseudo-classes (Static Only!):** `hover:`, `active:`, `focus:`, `group-hover:`, `group-active:` are completely supported.

### 🛑 CRITICAL TAILWIND TRAPS & ERRORS

If you get **`E0061: this method takes 1 argument but 0 arguments were supplied`**, you fell into one of these traps:

- **`border` is a 1px border:** The class parser maps `border` to `.border_1()`. Directional classes such as `border-b` set the corresponding one-pixel border.

- **Font family classes:** Browser Tailwind font-family utilities are not documented mappings in the current parser. Use an explicit GPUI font-family builder attribute when selecting a family.
- **Arbitrary values:** The parser supports documented arbitrary lengths and colors, but not every CSS/Tailwind value. Use a direct GPUI attribute when a value does not map cleanly, and keep this guide aligned with the actual parser rather than assuming browser CSS behavior.

---

## 4. Usage of Button (or) button groups

```jsx
<button_group id="actions">
  <button id="btn1" label="One" selected={true} />
  <button id="btn2" label="Two" />
  <button id="btn3">
    <div class="flex items-center gap-2">
      <icon name={IconName::Eye} />
      "Three"
    </div>
  </button>
</button_group>
```

### 4.1 Usage of kbd

```jsx
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

### 4.2 Tables

Use the lowercase table tags for a small, manually authored table. For data-backed, sortable tables, use `<DataTable rows={...}>` with `<Col>` accessors:

```jsx
use gpui_kit::{component::table::*, px, rgb, SharedString};
use zopra::{component, view};

#[component]
pub fn invoice_table() {
view! {
// 1. Root Table: Combining Tailwind `class` with GPUI `Styled` trait methods (border_0, rounded_none)
<table
            class="w-full bg-[#141517] shadow-lg text-[#d9dbe0]"
            border_0
            rounded_none
        >
// 2. TableHeader (Mapped to <thead>)
<thead class="bg-[#1d2024] border-b border-[#272a2f]">
<tr>
// 3. TableHead (Mapped to <th>): Fixed width using w={px(...)}
<th w={px(80.0)} class="p-2 text-[12px] font-semibold text-[#858990]">"ID"</th>

                    // Flex-1 column (default behavior when no width is set)
                    <th class="p-2 text-[12px] font-semibold text-[#858990]">"Customer"</th>

                    // Center-aligned text using GPUI's explicit `text_center`
                    <th text_center class="p-2 text-[12px] font-semibold text-[#858990]">"Status"</th>

                    // Right-aligned text using GPUI's explicit `text_right`
                    <th text_right class="p-2 text-[12px] font-semibold text-[#858990]">"Amount"</th>
                </tr>
            </thead>

            // 4. TableBody (Mapped to <tbody>)
            <tbody class="divide-y divide-[#272a2f]">

                // 5. TableRow (Mapped to <tr>) with Tailwind hover states
                <tr class="hover:bg-[#1d2024]">
                    // 6. TableCell (Mapped to <td>)
                    <td class="p-2 font-mono text-[12px] text-[#858990]">"INV001"</td>

                    // Custom cell padding using GPUI's explicit `px_4` instead of a tailwind class
                    <td px_4>"John Doe"</td>

                    <td text_center class="p-2 text-[#10b981]">"Paid"</td>
                    <td text_right class="p-2 font-mono">"$250.00"</td>
                </tr>

                // Row with dynamic custom background via explicit builder `bg={...}`
                <tr bg={rgb(0x1a1c1f)} class="hover:bg-[#1d2024]">
                    <td class="p-2 font-mono text-[12px] text-[#858990]">"INV002"</td>
                    <td class="p-2">"Jane Smith"</td>
                    <td text_center class="p-2 text-[#f59e0b]">"Pending"</td>
                    <td text_right class="p-2 font-mono">"$1,200.00"</td>
                </tr>
            </tbody>

            // 7. TableFooter (Mapped to <tfoot>)
            <tfoot class="bg-[#1d2024] border-t border-[#272a2f]">
                <tr>
                    <td class="p-2 font-semibold">"Total"</td>
                    <td></td> // Empty spacer cell
                    <td></td>
                    <td text_right class="p-2 font-mono font-semibold text-[#e0e1e4]">
                        "$1,450.00"
                    </td>
                </tr>
            </tfoot>

            // 8. TableCaption (Mapped to <caption>)
            <caption text_center class="mt-4 text-[11px] text-[#656970]">
                "A list of recent invoices."
            </caption>
        </table>
    }

}
```

For a data-backed table, `rows` accepts a `Vec<T>`, `Rc<Vec<T>>`, or `Snap<Vec<T>>`. The column accessor renders each value and can provide the sort key when that value implements `PartialOrd`:

```rust
#[derive(Clone)]
struct Invoice {
    id: u32,
    customer: SharedString,
    amount: u32,
}

#[component]
fn invoices() {
    let rows = vec![
        Invoice { id: 1, customer: "Ada".into(), amount: 250 },
        Invoice { id: 2, customer: "Lin".into(), amount: 1200 },
    ];

    view! {
        <DataTable rows={rows}>
            <Col title="ID" r={|row| row.id} sortable />
            <Col title="Customer" r={|row| row.customer.clone()} sortable />
            <Col title="Amount" r={|row| row.amount} sortable />
        </DataTable>
    }
}
```

## Resizable

```jsx
use gpui_kit::component::resizable::*;
use gpui_kit::px;
use zopra::{component, hooks::use_state, view};

#[component]
pub fn interactive_layout() {
    view! {
        // GPUI-RSX creates and retains resizable state from this stable id.
        <Resizable id="main-layout" horizontal>

            <ResizablePanel
                size={px(250.0)}
                size_range={px(200.0)..px(400.0)}
            >
                <div class="bg-[#141517] h-full p-4">"Left Sidebar"</div>
            </ResizablePanel>

            <ResizablePanel>
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

## 5. The Dynamic vs Static Class Conundrum (CRITICAL)

This is the most important architectural constraint in Zopra.

### Static Classes (Fast Path)

When you write `<div class="hover:bg-white text-black">`, the macro parses supported utilities at **compile time** and emits GPUI builder calls. Support depends on the parser and target element; this is not full browser CSS or Tailwind.

### Dynamic Classes & `cn!`

`cn!` combines conditional class inputs into one string and runs the class merge once. It is not a runtime Tailwind parser. Keep interaction utilities such as `hover:` in static `class` values or use the component's dedicated interaction attributes; only use dynamic class values supported by the target builder.
---

## 6. Zopra Hooks

Hooks belong unconditionally at the top level of a `#[component]` body. Zopra injects `window` and `cx`, so component code normally omits those framework arguments. The rewriter reports a focused compile error when a hook is called inside a branch, nested block, or closure. See [`SKILL/hooks/`](hooks/README.md) for hook-specific behavior and diagnostics.

The focused reference pages for each hook are in [`SKILL/hooks/`](hooks/README.md). They cover state, models, resources, effects, events, async work, callbacks, tables, window launchers, mount macros, and the optional Wry webview hook.

### `use_state`

`use_state` returns an immutable `Snap<T>` and a callable `Setter<T>`. State types need `Clone + PartialEq`; derive both for structs. Reading, displaying, comparing, and field access work through deref. Arithmetic consumes values, so dereference explicitly. `set` skips equal values; closure setters mutate with copy-on-write and notify.

```rust
let (count, set_count) = use_state(0);
let (draft, set_draft) = use_state(String::new());

view! {
    <button onClick={|| set_count(*count + 1)}>{format!("Count: {count}")}</button>
    <input value={&draft} onChange={set_draft} />
    <button onClick={|| set_draft(|text| text.push('!'))}>"Edit"</button>
}
```

String literals passed to `use_state` become `SharedString`; use `String::new()` when you need to mutate the string in place. Event handlers capture the snapshot from the render that created them. Use `set_count(|value| *value += 1)` or `set_count.current()` for long-lived async work that needs the latest value.

### `use_model`

Use `use_model` for large or frequently updated data that should not be cloned into snapshots and does not need `PartialEq`:

```rust
let rows = use_model(|| Vec::<Row>::new());
rows.update(|rows| rows.push(Row::default()));
let row_count = rows.read().len();
```

### `use_resource`

`use_resource(deps, load)` runs on mount and when the `Clone + PartialEq` dependency value changes. It returns `Snap<ResourceState<T, E>>` with `Loading`, `Ready(value)`, or `Failed(error)` variants. Older requests are ignored if a newer dependency value has started loading.

```rust
let user = use_resource([user_id], || async move {
    fetch_user(user_id).await
});

match &*user {
    ResourceState::Loading => view! { <div>"Loading"</div> },
    ResourceState::Ready(user) => view! { <div>{&user.name}</div> },
    ResourceState::Failed(error) => view! { <div>{error.to_string()}</div> },
}
```

### `use_effect!`

Runs after render on mount and when its dependency values change. No explicit `cx` is needed in a component:

```rust
use_effect!(|| println!("Count is now {count}"), [count]);
```

### `use_event`

Subscribes to an entity implementing GPUI's `EventEmitter<Evt>` and calls the closure for each event. In a component, the macro supplies `cx`:

```rust
use_event(&publisher, |event| {
    println!("Received event: {event:?}");
});
```

### `use_async`

Spawns an async task bound to the component entity. The async closure receives a weak entity and `AsyncApp`; use setters' `current()` method when the task needs fresh state:

```rust
use_async(async move |_weak_entity, async_cx| {
    let result = fetch_data().await;
    async_cx.update(|cx| set_result(result, cx)).ok();
});
```

### `use_callback`

Creates a callback that updates a target entity if it is still alive. It receives a mutable entity, the event, the current window, and the target entity context:

```rust
let handler = use_callback(&entity, |state, event, window, cx| {
    state.handle(event, window, cx);
});
```

### `use_window`

Creates a reusable launcher for another GPUI window. Call `open` from an event handler; the render closure is called with the new window's own context. `open` returns `anyhow::Result<AnyWindowHandle>`.

```rust
let browser = use_window(|| WindowOptions {
    window_bounds: Some(bounds),
    ..Default::default()
});

view! {
    <button id="open-browser" onClick={|| {
        let _ = browser.open(|| view! { <Browser /> });
    }} />
}
```

---

## 7. Handling Rust Types in `view!`

Because `view!` expands into Rust code, you can use native Rust patterns directly.

### Option Types (`if let Some`)

When dealing with `Option<T>`, you can use standard Rust branching:

```rust
view! {
    <div>
        {if let Some(text) = &optional_text {
            view! { <span>{text.clone()}</span> }
        } else {
            view! { <span>"No text"</span> }
        }}
    </div>
}
```

### Variables and String Interpolation

Variables can be passed directly inside braces. GPUI accepts `&str` and `String` seamlessly.

```rust
let title = "Hello World";
let subtitle = format!("User {}", user_id);

view! {
    <div class="flex flex-col">
        <h1>{title}</h1>
        <h2>{subtitle}</h2>
    </div>
}
```

---

## 8. Best Practices & Optimization

Straight from the `gpui-rsx` performance documentation, follow these rules to keep Zopra apps lightning fast:

### 1. Flatten Structure (Avoid Over-Nesting)

Do not nest `<div>`s unnecessarily just to apply spacing. Combine flex classes on a single container!
❌ **BAD:**

```rust
view! {
    <div>
        <div class="p-4">
            <div>"Content"</div>
        </div>
    </div>
}
```

✅ **GOOD:**

```rust
view! {
    <div class="p-4">
        "Content"
    </div>
}
```

### 2. Component Splitting

Break complex UIs into smaller helper methods returning `impl IntoElement`.

```rust
impl MyState {
    fn render_header(&self) -> impl IntoElement {
        view! { <header>"Header"</header> }
    }
}
```

### 3. Use Constants for Styles

For heavily reused colors, define standard Rust constants rather than relying on string parsing:

```rust
const PRIMARY_BG: gpui::Rgba = gpui::rgb(0x3b82f6);
const PRIMARY_TEXT: gpui::Rgba = gpui::rgb(0xffffff);

view! {
    <button bg={PRIMARY_BG} text_color={PRIMARY_TEXT}>
        "Click Me"
    </button>
}
```

### 4. Performance: Static vs Dynamic

- **Static strings** (`class="flex bg-white"`) are parsed at compile-time into O(1) chained method calls. There is ZERO runtime overhead!
- **Dynamic strings** (`class={cn!(...)}`) incur a slight runtime overhead as they are hashed/matched against a lookup table.
  _Rule of thumb:_ Always use static strings unless the state forces the classes to change dynamically.

---

## 9. Debugging the Macro

If `view!` is failing to compile and the error is cryptic (e.g. `method not found in E`), you can inspect what the macro actually generates.

1. Install cargo-expand: `cargo install cargo-expand`
2. Run: `cargo expand --lib` or `cargo expand --bin project`

You can also use the `rsx_expand!` debugging macro to preview the generated Rust code for a specific block locally.
