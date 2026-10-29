# Settings

`zopra::ui` builds a settings screen from `<Settings>` > `<SettingPage>` > `<SettingGroup>` > `<SettingItem>`, with one field inside each item.

```rust
use gpui_kit::component::Size;
use zopra::ui::*;

#[component]
pub fn app_settings(font: Signal<SharedString>, auto_theme: Signal<bool>, font_size: Signal<f64>) {
    view! {
        <Settings id="app-settings" size={Size::Medium}>
            <SettingPage title="General" description="Global preferences" default_open={true}>
                <SettingGroup title="Appearance">
                    <SettingItem title="Auto Switch Theme" description="Follow the system theme.">
                        <SettingSwitch
                            value={move |cx| auto_theme.get(cx)}
                            on_change={move |on, cx| auto_theme.set(on, cx)}
                            default_value={false}
                        />
                    </SettingItem>
                    <SettingItem title="Font Family">
                        <SettingDropdown
                            options={vec![("Inter".into(), "Inter".into()), ("Mono".into(), "Mono".into())]}
                            value={move |cx| font.get(cx)}
                            on_change={move |name, cx| font.set(name, cx)}
                        />
                    </SettingItem>
                    <SettingItem title="Font Size">
                        <SettingNumberInput
                            options={NumberFieldOptions { min: 8.0, max: 72.0, ..Default::default() }}
                            value={move |cx| font_size.get(cx)}
                            on_change={move |size, cx| font_size.set(size, cx)}
                        />
                    </SettingItem>
                </SettingGroup>
            </SettingPage>
        </Settings>
    }
}
```

A field reads its value with `value={|cx| ..}` and writes it with `on_change={|value, cx| ..}`. These closures run whenever the settings screen asks, so capture signals (they're `Copy`) and read and write them through `cx`.

| Tag | Props |
|---|---|
| `<Settings>` | `id`, `size`, `group_variant`, `sidebar_width`, `sidebar_size_range`; children are `<SettingPage>`s |
| `<SettingPage>` | `title`, `description`, `icon`, `default_open`, `resettable`; children are `<SettingGroup>`s |
| `<SettingGroup>` | `title`, `description`, `variant`; children are `<SettingItem>`s |
| `<SettingItem>` | `title`, `description`, `layout`, `keywords`, `disabled`; its one child is a field |
| `<SettingCustomItem>` | `content={\|options, window, cx\| ..}` draws the whole row |

Fields: `<SettingSwitch>`, `<SettingCheckbox>` (bool), `<SettingTextInput>`, `<SettingDropdown>` (`options`, `scrollable`) (text), `<SettingNumberInput>` (`options`) (number), each with `value`, `on_change` and an optional `default_value`. `<SettingCustomField content={|options, window, cx| ..} />` draws any element as the field.
