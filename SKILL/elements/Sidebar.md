# Sidebar

`zopra::ui` follows the shadcn sidebar structure: `<Sidebar>` > `<SidebarGroup>` > `<SidebarMenu>` > `<SidebarMenuItem>`.

```rust
use gpui_kit::component::{Icon, IconName, sidebar::SidebarCollapsible};
use zopra::ui::*;

#[component]
pub fn app_sidebar(page: Signal<Page>, collapsed: bool) {
    view! {
        <Sidebar
            collapsed={collapsed}
            collapsible={SidebarCollapsible::Icon}
            class="h-full border-r border-[#272a2f]"
            header={view! {
                <SidebarHeader class="p-4">"Admin Panel"</SidebarHeader>
            }}
            footer={view! {
                <SidebarFooter>
                    <div class="flex items-center gap-2">"Administrator"</div>
                </SidebarFooter>
            }}
        >
            <SidebarGroup label="Overview">
                <SidebarMenu>
                    <SidebarMenuItem
                        label="Dashboard"
                        icon={Icon::new(IconName::LayoutDashboard)}
                        active={page() == Page::Dashboard}
                        on_click={|| page(Page::Dashboard)}
                    />
                    // nested items are children
                    <SidebarMenuItem label="Developer" icon={Icon::new(IconName::Code)}>
                        <SidebarMenuItem label="Console" on_click={|| page(Page::Console)} />
                    </SidebarMenuItem>
                </SidebarMenu>
            </SidebarGroup>
        </Sidebar>
    }
}
```

| Tag | Props |
|---|---|
| `<Sidebar>` | `collapsed`, `collapsible`, `header`, `footer` (any element), `class`; children are `<SidebarGroup>`s |
| `<SidebarHeader>` / `<SidebarFooter>` | `class`; children are any elements |
| `<SidebarGroup>` | `label`; children are `<SidebarMenu>`s |
| `<SidebarMenu>` | `class`; children are `<SidebarMenuItem>`s |
| `<SidebarMenuItem>` | `label`, `icon`, `active`, `disabled`, `on_click`, `class`; children are nested items |
| `<SidebarToggleButton>` | `collapsed`, `on_click` |
