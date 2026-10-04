# Icon

```rust
use gpui_kit::component::button::Button;
use gpui_kit::component::Icon;
use gpui_kit::{assets::IconName, prelude::FluentBuilder};
use gpui_kit::{radians, rgb, px, Transformation};
use zopra::{component, view};

#[component]
pub fn icon_showcase() {
    let search_bytes = include_bytes!("assets/search.svg");

    view! {
        <div class="flex flex-col gap-4 p-4 bg-[#141517]">

            // 1. Basic Icons & Navigation
            <div class="flex gap-2">
                <icon name={IconName::ArrowLeft} />
                <icon name={IconName::ChevronDown} />
            </div>

            // 2. Predefined & Custom Sizes
            <div class="flex gap-2 items-center">
                <icon name={IconName::Search} xsmall />
                <icon name={IconName::Search} small />
                <icon name={IconName::Search} medium /> // default
                <icon name={IconName::Search} large />
                <icon name={IconName::Search} size={px(32.0)} />
            </div>

            // 3. Status Icons with Colors
            <div class="flex gap-2">
                // Success
                <icon name={IconName::CircleCheck} text_color={rgb(0x10b981)} />
                // Error
                <icon name={IconName::CircleX} text_color={rgb(0xef4444)} />
                // Warning
                <icon name={IconName::TriangleAlert} text_color={rgb(0xf59e0b)} />

                // (Or you could just use Tailwind classes if supported!)
                <icon name={IconName::Heart} class="text-red-500" />
            </div>

            // 4. Rotated & Transformed Icons
            <div class="flex gap-2">
                <icon
                    name={IconName::ArrowUp}
                    rotate={radians(std::f32::consts::FRAC_PI_2)}
                />
                <icon
                    name={IconName::ChevronRight}
                    transform={Transformation::rotate(radians(std::f32::consts::PI))}
                />
            </div>

            // 5. Custom SVGs from Asset Paths (Maps to Icon::empty().path(...))
            <div class="flex gap-2">
                <icon path="icons/my-brand-logo.svg" large text_color={rgb(0x7c6ff2)} />
            </div>

            // 6. SVG Bytes & Fallbacks (Maps to Icon::default().data(...))
            <div class="flex gap-2">
                // Directly embeds the SVG bytes
                <icon data={search_bytes} text_color={rgb(0xd9dbe0)} />

                // Last source wins: sets path, then overwrites with direct bytes
                <icon path="icons/old.svg" data={search_bytes} />
            </div>

            // 7. Component Integration
            // Note: If you're passing an icon INTO a button builder, you'd still pass the object,
            // but for standalone animated loading icons:
            <div class="flex justify-center p-2">
                <icon name={IconName::LoaderCircle} text_color={rgb(0x656970)} medium />
            </div>

        </div>
    }
}
```
