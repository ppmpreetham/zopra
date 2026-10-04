# Sidebar

_(Notice how we use the `label` attribute to pass the strings required by `SidebarGroup::new("...")` and `SidebarMenuItem::new("...")`)_:

```rust
use gpui_kit::component::sidebar::*;
use gpui_kit::component::{Badge, Switch, Side};
use gpui_kit::component::IconName;
use zopra::{component, hooks::use_state, view};

#[component]
pub fn admin_dashboard() {
    let (collapsed, set_collapsed) = use_state(false);

    view! {
        // 1. Sidebar Root
        <Sidebar
            collapsible={true}
            collapsed={*collapsed}
            width={280}
            // `header` and `footer` map directly to the `.header()` and `.footer()` builder methods.
            // We use `view! { ... }` to pass complex trees into those attributes!
            header={view! {
                <SidebarHeader p_4>
                    <div class="flex items-center gap-2">
                        <div class="size-8 rounded-full bg-blue-500 flex items-center justify-center">
                            <icon name={IconName::Crown} />
                        </div>
                        "Admin Panel"
                    </div>
                </SidebarHeader>
            }}
            footer={view! {
                <SidebarFooter justify_between>
                    <div class="flex items-center gap-2">
                        <icon name={IconName::User} />
                        "Administrator"
                    </div>
                    <icon name={IconName::LogOut} />
                </SidebarFooter>
            }}
        >
            // 2. Sidebar Groups (maps to SidebarGroup::new(label))
            <SidebarGroup label="Overview">
                <SidebarMenu>
                    <SidebarMenuItem
                        label="Dashboard"
                        icon={IconName::LayoutDashboard}
                        active={true}
                    />
                    <SidebarMenuItem
                        label="Analytics"
                        icon={IconName::TrendingUp}
                        // Suffixes can also take inline Zopra views!
                        suffix={view! { <Badge count={2} /> }}
                    />
                </SidebarMenu>
            </SidebarGroup>

            <SidebarGroup label="Settings">
                <SidebarMenu>
                    // 3. Nested Menu Items
                    // Instead of `.children([...])`, just nest them naturally!
                    <SidebarMenuItem label="Developer" icon={IconName::Code}>
                        <SidebarMenuItem
                            label="Debug Mode"
                            suffix={view! { <Switch id="debug" checked={false} xsmall /> }}
                        />
                        <SidebarMenuItem
                            label="Console"
                            on_click={|_, _, _| println!("Console opened")}
                        />
                    </SidebarMenuItem>
                </SidebarMenu>
            </SidebarGroup>
        </Sidebar>

        // 4. Toggle Button (placed wherever you want in your layout)
        <SidebarToggleButton
            collapsed={*collapsed}
            on_click={|| set_collapsed(!*collapsed)}
        />
    }
}
```

### Constructor Mappings

- `<Sidebar>` ➔ `gpui_kit::component::sidebar::Sidebar::new()`
- `<SidebarHeader>` ➔ `gpui_kit::component::sidebar::SidebarHeader::new()`
- `<SidebarFooter>` ➔ `gpui_kit::component::sidebar::SidebarFooter::new()`
- `<SidebarMenu>` ➔ `gpui_kit::component::sidebar::SidebarMenu::new()`
- `<SidebarGroup label="...">` ➔ `gpui_kit::component::sidebar::SidebarGroup::new(label)`
- `<SidebarMenuItem label="...">` ➔ `gpui_kit::component::sidebar::SidebarMenuItem::new(label)`
- `<SidebarToggleButton>` ➔ `gpui_kit::component::sidebar::SidebarToggleButton::new()`
