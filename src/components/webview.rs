#[cfg(feature = "wry")]
use gpui_kit::prelude::*;
#[cfg(feature = "wry")]
use gpui_kit::*;
#[cfg(feature = "wry")]
use zopra_macros::component;

#[cfg(feature = "wry")]
use crate::hooks::Signal;

/// A `Copy` handle that drives a [`WebView`] from outside it (back, forward, load...).
#[cfg(feature = "wry")]
#[derive(Clone, Copy, Debug)]
pub struct WebViewController(Signal<Option<WeakEntity<gpui_wry::WebView>>>);

#[cfg(feature = "wry")]
impl WebViewController {
    pub fn set(self, webview: &Entity<gpui_wry::WebView>, cx: &mut App) {
        let webview = webview.downgrade();
        self.0.update_untracked(|current| *current = Some(webview), cx);
    }

    fn with_webview(self, f: impl FnOnce(&mut gpui_wry::WebView), cx: &mut App) {
        if let Some(webview) = self.0.read(cx).as_ref().and_then(WeakEntity::upgrade) {
            webview.update(cx, |webview, _| f(webview));
        }
    }

    pub fn load_url(self, url: &str, cx: &mut App) {
        self.with_webview(|view| view.load_url(url), cx);
    }

    pub fn back(self, cx: &mut App) {
        self.with_webview(|view| _ = view.back(), cx);
    }

    pub fn forward(self, cx: &mut App) {
        self.with_webview(|view| _ = view.raw().evaluate_script("history.forward();"), cx);
    }

    pub fn reload(self, cx: &mut App) {
        self.with_webview(|view| _ = view.raw().evaluate_script("location.reload();"), cx);
    }
}

#[cfg(feature = "wry")]
#[track_caller]
pub fn use_webview(window: &mut Window, cx: &mut App) -> WebViewController {
    WebViewController(crate::hooks::use_signal(|| None, window, cx))
}

#[cfg(feature = "wry")]
fn build_webview(
    builder: lb_wry::WebViewBuilder,
    window: &mut Window,
) -> Result<lb_wry::WebView, String> {
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    )))]
    {
        use gpui_wry::lb_wry::WebViewBuilderExtUnix;
        let fixed = gtk::Fixed::builder().build();
        fixed.show_all();
        return builder.build_gtk(&fixed).map_err(|err| err.to_string());
    }

    #[cfg(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    ))]
    {
        use raw_window_handle::HasWindowHandle;
        let handle = window
            .window_handle()
            .map_err(|_| "no window handle".to_string())?;
        builder
            .build_as_child(&handle)
            .map_err(|err| err.to_string())
    }
}

#[cfg(feature = "wry")]
#[component]
pub fn WebView(
    url: String,
    controller: Option<WebViewController>,
    transparent: Option<bool>,
    devtools: Option<bool>,
    proxy: Option<lb_wry::ProxyConfig>,
) {
    use crate::hooks::use_state;

    let (wv, set_wv) = use_state(None::<Entity<gpui_wry::WebView>>);

    if wv.read().is_none() {
        let mut builder = lb_wry::WebViewBuilder::new().with_url(&url);
        if let Some(p) = proxy {
            builder = builder.with_proxy_config(p);
        }
        if devtools.unwrap_or(cfg!(debug_assertions)) && cfg!(debug_assertions) {
            builder = builder.with_devtools(true);
        }
        if transparent.unwrap_or(false) {
            builder = builder.with_transparent(true);
        }

        let Some(webview) = build_webview(builder, window).ok() else {
            return crate::view! { <div class="size-full" /> };
        };

        let wv = cx.new(|inner_cx| gpui_wry::WebView::new(webview, window, inner_cx));

        if let Some(ctrl) = controller {
            ctrl.set(&wv, cx);
        }
        set_wv(Some(wv));
    }

    crate::view! {
        <div class="size-full">
            {
                if let Some(wv) = wv.read() {
                    wv.clone().into_any_element()
                } else {
                    div().into_any_element()
                }
            }
        </div>
    }
}
