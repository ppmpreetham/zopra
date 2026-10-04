# `use_webview`

`use_webview()` returns a cloneable `WebViewController`. The component macro injects its GPUI context. The hook and component are available only with Zopra's `wry` feature enabled.

```rust
let webview = use_webview();

view! {
    <WebView url="https://example.com" controller={Some(webview.clone())} />
    <button onClick={|| webview.back(cx)}>"Back"</button>
    <button onClick={|| webview.reload(cx)}>"Reload"</button>
}
```

The controller exposes `load_url(url, cx)`, `back(cx)`, `forward(cx)`, and `reload(cx)`. The methods currently take explicit `cx`; they operate on the attached webview and do nothing if its entity is no longer available. The `WebView` component itself is also gated behind the `wry` feature.
