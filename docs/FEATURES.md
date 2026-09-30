# Zopra Features

Zopra provides several optional features that can be enabled in your `Cargo.toml`.

## `wry`

Enables the `WebView` component, backed by `gpui-wry` and `tauri-apps/wry`. This allows you to easily embed full web views into your native GPUI applications using Zopra's declarative `rsx` syntax.

### Usage

Add the feature to your `Cargo.toml`:

```toml
[dependencies]
zopra = { version = "*", features = ["wry"] }
```

### Component Example

The `wry` feature introduces the `<WebView>` component and a `WebViewController` that you can access via `use_webview()`. The controller allows you to imperatively control the webview (e.g. going back, forwarding, or loading URLs), while keeping the component strictly declarative.

```rust
use zopra::{component, view, hooks::use_state};
use zopra::components::webview::{WebView, use_webview};

#[component]
pub fn browser_view() {
    let (url, set_url) = use_state("https://gpui-kit.com".to_string());

    // use_webview hook
    let webview_ctrl = use_webview();

    view! {
        <div class="flex flex-col size-full p-2 bg-gray-50">
            <div class="flex flex-row gap-2">
                <button
                    label="Back"
                    on_click={
                        let ctrl = webview_ctrl.clone();
                        move |_, _, cx| ctrl.back(cx)
                    }
                />
                <button
                    label="Forward"
                    on_click={
                        let ctrl = webview_ctrl.clone();
                        move |_, _, cx| ctrl.forward(cx)
                    }
                />
                <button
                    label="Load Example"
                    on_click={
                        let ctrl = webview_ctrl.clone();
                        move |_, _, cx| {
                            set_url("https://example.com".to_string(), cx);
                            ctrl.load_url("https://example.com", cx);
                        }
                    }
                />
            </div>

            <WebView
                controller={webview_ctrl.clone()}
                url={url()}
                class="flex-1 border border-gray-300 rounded-md"
            />
        </div>
    }
}
```

### Config Options

The `<WebView>` component accepts the following props:

- `url` (`String`): The URL to load when the view is initialized.
- `controller` (`Option<WebViewController>`): An optional handle created by `use_webview()` to dispatch commands to the WebView.
- `devtools` (`Option<bool>`): Whether to open the web inspector. Defaults to `true` when compiling in debug mode.
- `transparent` (`Option<bool>`): Configures the webview to have a transparent background. Defaults to `false`.
