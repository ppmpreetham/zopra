### 1. Basic Tabs & Variants

The `<nav>` tag will automatically generate an ID under the hood and map directly to `TabBar::new()`. Any boolean variants act as flags:

```rust
// Default Tabs
<nav selected_index={active_tab} on_click={move |index, _, cx| set_active_tab(*index, cx)}>
    <Tab label="Account" />
    <Tab label="Profile" disabled={true} />
    <Tab label="Settings" />
</nav>

// Pill Tabs
<nav pill selected_index={0}>
    <Tab label="Account" />
</nav>

// Underline Tabs (Small size)
<nav underline small selected_index={0}>
    <Tab label="Account" />
</nav>
```

### 2. Segmented Tabs

For segmented tabs, the GPUI API allows you to pass standard icons or strings directly as children without even needing a `<Tab>` wrapper!

```rust
<nav segmented selected_index={0}>
    <icon name={IconName::Bot} />
    <icon name={IconName::Calendar} />
    "Settings"
    "About"
</nav>
```

### 3. Rich / Custom Tab Content

If you want badges, colors, or custom flex layouts inside a tab, you just nest them directly inside the `<Tab>` tag as children:

```rust
<Tab>
    <div class="flex items-center gap-2">
        <icon name={IconName::Folder} />
        "Documents"

        // A little notification badge
        <div class="px-1 py-0.5 text-xs bg-blue-500 rounded-sm">
            "12"
        </div>
    </div>
</Tab>
```

### 4. Advanced: Max Width, Menus, Prefix, and Suffix

You can easily cap the width (which auto-truncates text), add a dropdown menu for overflowing tabs, and attach components to the far left/right of the nav bar:

```rust
<nav
    max_width={px(120.)}
    menu={true}
    prefix={view! { <Button icon={IconName::ArrowLeft} ghost xsmall /> }.into_any_element()}
    suffix={view! { <Button icon={IconName::Plus} ghost xsmall /> }.into_any_element()}
>
    <Tab label="Extremely Long Tab Name That Will Truncate" />
    <Tab label="Short" />
</nav>
```

### 5. Tabs with Close Buttons (Per-Tab Suffix)

You can also put suffixes on _individual_ tabs to create close buttons!

```rust
<Tab
    label="document.txt"
    suffix={view! { <Button icon={IconName::X} ghost xsmall /> }.into_any_element()}
/>
```
