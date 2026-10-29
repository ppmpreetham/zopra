# TabBar

`zopra::ui::TabBar` holds `<Tab>`s. `on_click` gets the clicked tab's index.

```rust
use zopra::ui::{Tab, TabBar};

let tab = use_signal(0usize);

view! {
    <TabBar selected_index={tab()} on_click={|index, _, cx| tab(*index)} underline>
        <Tab label="Account" />
        <Tab label="Profile" disabled={true} />
        <Tab label="Settings" />
    </TabBar>
}
```

Variants are flags: `underline`, `pill`, `outline`, `segmented`. Other props: `size`, `max_width` (truncates long labels), `menu` (a menu for tabs that don't fit), `prefix` / `suffix` (elements at either end), `class`.

## Rich tabs

A `<Tab>`'s children render inside it, and `prefix` / `suffix` take any element:

```rust
<Tab
    label="document.txt"
    suffix={view! { <div on_click={|| close(id)}>"×"</div> }}
>
    <div class="px-1 text-xs bg-blue-500 rounded-sm">"12"</div>
</Tab>
```

## Tabs from data

Read what you need from a signal first, then build the tabs. Reading a signal borrows `cx`, which each `<Tab>` needs while it renders:

```rust
let names = docs.read().iter().map(|doc| doc.name.clone()).collect::<Vec<_>>();

view! {
    <TabBar selected_index={open()} on_click={|index, _, cx| open(*index)}>
        {...names.into_iter().map(|name| view! { <Tab label={name} /> })}
    </TabBar>
}
```
