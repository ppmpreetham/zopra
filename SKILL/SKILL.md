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
            onClick={|| set_count(count() + 1)}
        >
            {label} ": " {count().to_string()}
        </div>
    }
}
```

### Props

A component's parameters are its props, set as attributes on its tag (`<CounterButton initial_count={1} label="Clicks" />`):

- **Required:** a plain parameter. Leaving one off is a compile error: "`<CounterButton>` is missing the required prop `label`".
- **Optional:** `Option<T>`. The attribute takes a `T` (`size={px(250.)}`), and a bare flag sets `true` (`vertical`).
- **Callbacks:** `on_pick: Option<F>` with `F: Fn(..)` as a generic parameter. Optional and never boxed.
- **Conversions:** `String` / `SharedString` props take anything stringy, `AnyElement` props take any element (`footer={view! { <div/> }}`), and `ElementId` props take `"text"`.
- **`key`:** every tag accepts `key={..}` to keep a looped instance's state attached to its item. Without it, instances are told apart by render order.
- **Reserved names:** `new`, `key`, `render` and `render_at` can't be prop names.

A component normally renders an element. Declare a return type (`-> Tab`) when it builds something else for its parent, like the `zopra::ui` wrappers do.

### Children

Declare `children: Children` and place it with `{children}`. The content between the tags renders there, inside this component's render, so it sees the contexts this component provides:

```rust
#[component]
fn card(title: SharedString, children: Children) {
    view! {
        <div class="rounded-md p-4">
            <div class="font-bold">{title}</div>
            {children}
        </div>
    }
}

// <Card title="Stats"><Chart /> "updated hourly"</Card>
```

`children: Children<Tab>` collects a specific child type instead of elements (a tab bar's tabs, a settings page's groups). Children are a generic closure, never boxed, and optional: `<Card title="x" />` is fine.

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

Interactive GPUI events are mapped to camelCase or snake_case attributes. In a `#[component]` `view!`, the macro supplies the missing `(event, window, cx)` parameters of GPUI events, so `onClick={|| set_count(count() + 1)}` works. Any other `on_*` closure keeps the parameters you write.

Signal sugar (`count()`, `set_count(v)`) needs a GPUI context. Zopra finds it by:
- position, for the closures it manages (event attributes, `use_event`, effects, `window.open`); there it can be called anything, or `_`;
- otherwise, a closure parameter named `cx` or typed as a GPUI context (`&mut App`, `&mut Context<_>`, `&mut AsyncApp`).

A `move` closure without a context of its own that reads a signal is a compile error explaining exactly that.

**Common Event Attributes:**

- `onClick` / `on_click`, `onAuxClick`
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

One rule: **lowercase tags are built-in elements, capitalized tags are components.** A capitalized tag is a `#[component]` (yours, or a `zopra::ui` one), and the content between its tags is its `children`.

`zopra::ui` provides gpui-kit's widgets as components, shadcn-style:
- `TitleBar`
- `Sidebar`, `SidebarHeader`, `SidebarFooter`, `SidebarGroup`, `SidebarMenu`, `SidebarMenuItem`, `SidebarToggleButton`
- `Resizable`, `ResizablePanel`
- `TabBar`, `Tab`
- `DropdownButton`
- `Settings`, `SettingPage`, `SettingGroup`, `SettingItem` and the setting fields

See `SKILL/elements/` for their props. Each takes `class`, compiled at build time like a `<div>`'s.

The exceptions:
- `<DataTable>` with `<Col>`s is compiled into a typed table.
- `base={expr}` on any tag uses `expr` as the element and passes every other attribute to its builder methods, for gpui-kit options no wrapper exposes:

```rust
<div base={gpui_kit::component::tab::TabBar::new("tabs")} max_width={px(120.)} />
```

Built-in tags:

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

- **Colors:** the Tailwind v4.2 palette (incl. `mauve`, `olive`, `mist`, `taupe`), `black`, `white`, `transparent`, hex/`rgb()`/`rgba()` in brackets (`bg-[#0a0a0a]`), and theme colors (`bg-theme-muted`). Every color takes an opacity: `bg-red-500/50`, `text-theme-primary/25`, `border-black/[0.3]`.
- **Spacing Scale:** `p-4`, `w-48` use the `0.25rem` step (`w-48` = 12rem). Also `px`, `full`, `auto` (`w-px`, `basis-full`, `ms-auto`), negatives (`-mt-2`), fractions on sizes (`w-1/2`), and brackets (`w-[150px]`, `p-[1rem]`, `h-[50%]`).
- **Logical / extra lengths:** `ps pe ms me pbs pbe mbs mbe start end` (left-to-right), `basis-*`, `space-x-* / space-y-*` (become `gap-x` / `gap-y`).
- **Layout:** flex classes, `grow-N`, `shrink-N`, `grid grid-cols-N grid-rows-N col-span-N col-start-N col-end-N row-*`, `place-items/content/self-*` (`place-items` aligns the cross axis only; GPUI has no `justify-items`), `overflow-hidden/visible/clip/auto/scroll` (+ `-x-`/`-y-`), `scrollbar-none`, `scrollbar-thin`, `scrollbar-[6px]`.
- **Text:** `text-xs` .. `text-9xl`, `text-sm/6` (size/line height), `leading-tight` / `leading-6` / `leading-[1.4]`, `font-sans` / `font-mono` (theme fonts), `font-serif`, font weights.
- **Borders & radius:** `border` (1px), `border-t`, `border-2`, colors; v4 radius scale `rounded-xs` .. `rounded-4xl`, `rounded` (.25rem), `-none`, `-full`, `-[6px]`, per side/corner (`rounded-t-lg`, `rounded-tl-xs`, `rounded-s-md`, `rounded-ee-xl`).
- **Shadows & rings:** `shadow`, `shadow-2xs` .. `shadow-2xl`, `shadow-none`, `shadow-inner`, `inset-shadow-2xs/xs/sm`, `ring`, `ring-2`, `ring-[3px]`, `inset-ring-*`, and colors for each (`shadow-red-500/30`, `ring-theme-primary`). A ring with no color uses the theme's `ring`. Presets are GPUI's own `shadow_*` methods (same values as v4); colors, inset shadows and rings go onto that list. A `hover:` shadow replaces the whole list while hovered.
- **Gradients:** `bg-linear-to-r` (or `bg-linear-45`, `bg-linear-[30deg]`) with `from-*` / `to-*` and optional `from-10%` / `to-90%`. GPUI gradients have two stops, so `via-*` is an error.
- **Variants:** `hover:`, `active:`, `focus:`, `group-hover:`; `dark:` (theme mode); breakpoints `sm: md: lg: xl: 2xl:` and `max-sm:` .. `max-2xl:` (window width, in rems). Variants nest condition-first: `md:dark:hover:bg-white` works, `hover:md:...` is an error.

### 🛑 CRITICAL TAILWIND TRAPS & ERRORS

- **Real Tailwind that GPUI can't do yet is a warning, and the class is ignored**, with what to use instead: `transition-*` / `animate-*` (use `with_animation`), filters, `rotate-*` / `scale-*`, `z-*`, `divide-*`, `outline-*`, `uppercase`, `tracking-*`, `fixed` / `sticky`, `inline-*`, `w-screen`, and variants like `disabled:`, `first:`, `data-*:`, `peer-*:`. Example: ``warning: ... `z-10` is ignored: GPUI doesn't support it yet. GPUI paints in tree order and has no z-index; ...``
- **Unknown classes are compile errors** naming the class: ``Unknown class `flex-cols`. If it's a GPUI style method, use it as an attribute instead: `<div flex_cols />`.`` Reach for an attribute (`<div flex_grow={2.0} />`) for anything the classes don't cover.
- **`dark:` needs `cx`, breakpoints need `window`** in scope with those names (components and `render` already have them). Dynamic classes (`class={expr}`) only cover the common subset, not variants, gradients or shadows; keep those in literal strings.
- **`w-fit` is `w-auto`:** GPUI has no fit/min/max-content sizing.
- **`border` is a 1px border:** `border` maps to `.border_1()`; directional classes such as `border-b` set that side's one-pixel border.

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
            <tbody>

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

For a data-backed table, `rows` takes a `Vec<T>` or a `Signal<Vec<T>>`. A signal is read in place with no copy, and the table re-sorts only after the signal is written. Rows must be `Clone` (context menus get a copy of their row). The column accessor renders each value and can provide the sort key when that value implements `PartialOrd`:

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
use zopra::ui::{Resizable, ResizablePanel};
use gpui_kit::px;
use zopra::{component, view};

#[component]
pub fn interactive_layout() {
    view! {
        // a stable id keeps the panel sizes across renders
        <Resizable id="main-layout">

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

The focused reference pages for each hook are in [`SKILL/hooks/`](hooks/README.md). They cover state, resources, effects, events, context, inputs, window launchers, and the optional Wry webview hook.

### `use_state`

`use_state` returns a `(getter, setter)` pair of `Signal<T>`: `Copy` handles to state owned by the app, so closures capture them without cloning. `count()` reads (clones), `count.read()` borrows, `set_count(v)` replaces and `set_count(|v| ..)` mutates in place; the macro passes `cx`. Reads always see the latest value, including in handlers and async tasks. `T` only needs `'static`.

```rust
let (count, set_count) = use_state(0);
let (draft, set_draft) = use_state(String::new());

view! {
    <button onClick={|| set_count(count() + 1)}>{format!("Count: {}", count())}</button>
    <input value={draft()} onChange={set_draft} />
    <button onClick={|| set_draft(|text| text.push('!'))}>"Edit"</button>
}
```

String literals passed to `use_state` become `SharedString`; use `String::new()` when you need to mutate the string in place. `use_signal(init)` returns one signal instead of a pair. A signal is disposed when its component unmounts: writes after that are ignored, and `try_update` returns `None` so long-running tasks can stop.

Signals are `Copy`, so sharing state is just passing the handle: as a `Signal<T>` prop, or with `provide_context(signal)` and `let s: Signal<T> = use_context();` in any component it renders (the innermost provider wins; components rendered outside that render, such as popup menu items, see no context). `set` skips equal values, so setting state during render cannot loop. Components rendered from one tag in a loop keep separate state; pass `key={id}` to keep it attached to an item when the list reorders.

### `use_resource`

`use_resource(deps, load)` runs on mount and when the `PartialEq` dependency value changes. It returns a `Signal<ResourceState<T, E>>` with `Loading`, `Ready(value)`, or `Failed(error)` variants. Older requests are ignored if a newer dependency value has started loading.

```rust
let user = use_resource(user_id, || async move {
    fetch_user(user_id).await
});

match user.read() {
    ResourceState::Loading => view! { <div>"Loading"</div> },
    ResourceState::Ready(user) => view! { <div>{user.name.clone()}</div> },
    ResourceState::Failed(error) => view! { <div>{error.to_string()}</div> },
}
```

### `use_effect!`

Runs after render on mount and when its dependency values change. No explicit `cx` is needed in a component:

```rust
use_effect!(|| println!("Count is now {}", count()), [count()]);
```

### `use_event`

Subscribes to an entity implementing GPUI's `EventEmitter<Evt>` and calls the closure for each event. In a component, the macro supplies `cx`:

```rust
use_event(&publisher, |event| {
    println!("Received event: {event:?}");
});
```

### `provide_context` / `use_context`

Shares a value with every component rendered inside this one, looked up by type. Signals are `Copy`, so providing one shares the state itself; the innermost provider wins:

```rust
// parent
let theme = use_signal(Theme::Dark);
provide_context(theme);

// any component it renders (either form)
let theme: Signal<Theme> = use_context();
let theme = use_context::<Signal<Theme>>();
```

`use_context` panics if no parent provided the type; `try_use_context` returns an `Option`. Elements rendered later, outside the parent's render (like popup menu items), see no context, so pass those their values directly.

### `use_input`

Text inputs in GPUI are state machines that manage their own cursor and focus, so they need a persistent `Entity`. `use_input` creates it once and reuses it across renders. The handle is refcounted; closures that use it get their own copy automatically:

```rust
let input = use_input();

view! {
    <input state={&input} class="py-2 px-4" />
    <button onClick={|_, _, cx| println!("Input: {}", input.read(cx).text())}>"Submit"</button>
}
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
