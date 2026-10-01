#[cfg(feature = "wry")]
use gpui_kit::prelude::*;
#[cfg(feature = "wry")]
use gpui_kit::*;
#[cfg(feature = "wry")]
use gpui_wry;
#[cfg(feature = "wry")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "wry")]
use zopra_macros::component;

#[cfg(feature = "wry")]
#[derive(Clone)]
pub struct WebViewController {
    pub webview: Arc<Mutex<Option<Entity<gpui_wry::WebView>>>>,
}

#[cfg(feature = "wry")]
impl Default for WebViewController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "wry")]
impl WebViewController {
    pub fn new() -> Self {
        Self {
            webview: Arc::new(Mutex::new(None)),
        }
    }

    pub fn load_url(&self, url: &str, cx: &mut App) {

        if let Ok(guard) = self.webview.lock() {

            if let Some(wv) = &*guard {

                wv.update(cx, |view, _| {

                    view.load_url(url);
                });
            }
        }
    }

    pub fn back(&self, cx: &mut App) {
        if let Ok(guard) = self.webview.lock()
            && let Some(wv) = &*guard
        {
            wv.update(cx, |view, _| {
                _ = view.back();
            });
        }
    }

    pub fn forward(&self, cx: &mut App) {
        if let Ok(guard) = self.webview.lock()
            && let Some(wv) = &*guard
        {
            wv.update(cx, |view, _| {
                _ = view.raw().evaluate_script("history.forward();");
            });
        }
    }

    pub fn reload(&self, cx: &mut App) {
        if let Ok(guard) = self.webview.lock()
            && let Some(wv) = &*guard
        {
            wv.update(cx, |view, _| {
                _ = view.raw().evaluate_script("location.reload();");
            });
        }
    }
}

#[cfg(feature = "wry")]
pub fn use_webview(window: &mut gpui_kit::Window, cx: &mut gpui_kit::App) -> WebViewController {
    let (ctrl, _) = crate::hooks::use_state(WebViewController::new(), window, cx);
    ctrl(cx)
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

    let (get_wv, set_wv) = use_state(None::<Entity<gpui_wry::WebView>>, window, cx);
    

    if get_wv(cx).is_none() {
        let wv = cx.new(|cx| {
            let mut builder = lb_wry::WebViewBuilder::new();
            builder = builder.with_url(&url);
            if let Some(p) = proxy {
                builder = builder.with_proxy_config(p);
            }

            if devtools.unwrap_or(cfg!(debug_assertions)) {
                #[cfg(debug_assertions)]
                {
                    builder = builder.with_devtools(true);
                }
            }

            if transparent.unwrap_or(false) {
                builder = builder.with_transparent(true);
            }

            #[cfg(not(any(
                target_os = "windows",
                target_os = "macos",
                target_os = "ios",
                target_os = "android"
            )))]
            let webview = {
                use gpui_wry::lb_wry::WebViewBuilderExtUnix;
                use gtk::prelude::*;
                let fixed = gtk::Fixed::builder().build();
                fixed.show_all();
                builder.build_gtk(&fixed).unwrap()
            };
            #[cfg(any(
                target_os = "windows",
                target_os = "macos",
                target_os = "ios",
                target_os = "android"
            ))]
            let webview = {
                use raw_window_handle::HasWindowHandle;
                let window_handle = window.window_handle().expect("No window handle");
                builder.build_as_child(&window_handle).unwrap()
            };

            let view = gpui_wry::WebView::new(webview, window, cx);
            view
        });

        wv.update(cx, |v, _| v.load_url(&url));

        if let Some(ctrl) = &controller
            && let Ok(mut guard) = ctrl.webview.lock()
        {
            *guard = Some(wv.clone());
        }

        set_wv(Some(wv.clone()), cx);
    }

    crate::view! {
        <div class="size-full">
            {
                if let Some(wv) = get_wv(cx) {
                    wv.into_any_element()
                } else {
                    div().into_any_element()
                }
            }
        </div>
    }
}

