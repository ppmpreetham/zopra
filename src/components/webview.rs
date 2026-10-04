#[cfg(feature = "wry")]
use std::cell::RefCell;
#[cfg(feature = "wry")]
use std::rc::Rc;

#[cfg(feature = "wry")]
use gpui_kit::prelude::*;
#[cfg(feature = "wry")]
use gpui_kit::*;
#[cfg(feature = "wry")]
use zopra_macros::component;

#[cfg(feature = "wry")]
#[derive(Clone, Default)]
pub struct WebViewController {
    webview: Rc<RefCell<Option<gpui_kit::WeakEntity<gpui_wry::WebView>>>>,
}

#[cfg(feature = "wry")]
impl WebViewController {
    pub fn set(&self, webview: &Entity<gpui_wry::WebView>) {
        *self.webview.borrow_mut() = Some(webview.downgrade());
    }

    fn with_webview<R>(
        &self,
        f: impl FnOnce(&Entity<gpui_wry::WebView>, &mut App) -> R,
        cx: &mut App,
    ) -> Option<R> {
        let weak = self.webview.borrow().as_ref()?.clone();
        let entity = weak.upgrade()?;
        Some(f(&entity, cx))
    }

    pub fn load_url(&self, url: &str, cx: &mut App) {
        self.with_webview(|wv, cx| wv.update(cx, |view, _| view.load_url(url)), cx);
    }

    pub fn back(&self, cx: &mut App) {
        self.with_webview(|wv, cx| wv.update(cx, |view, _| _ = view.back()), cx);
    }

    pub fn forward(&self, cx: &mut App) {
        self.with_webview(
            |wv, cx| {
                wv.update(cx, |view, _| {
                    _ = view.raw().evaluate_script("history.forward();")
                })
            },
            cx,
        );
    }

    pub fn reload(&self, cx: &mut App) {
        self.with_webview(
            |wv, cx| {
                wv.update(cx, |view, _| {
                    _ = view.raw().evaluate_script("location.reload();")
                })
            },
            cx,
        );
    }
}

#[cfg(feature = "wry")]
pub fn use_webview(window: &mut Window, cx: &mut App) -> WebViewController {
    crate::hooks::use_model(WebViewController::default, window, cx)
        .read(cx)
        .clone()
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

    let (wv, set_wv) = use_state(|| None::<Entity<gpui_wry::WebView>>, window, cx);

    if wv.is_none() {
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

        if let Some(ctrl) = &controller {
            ctrl.set(&wv);
        }
        set_wv.set(Some(wv), cx);
    }

    crate::view! {
        <div class="size-full">
            {
                if let Some(wv) = wv.as_ref() {
                    wv.clone().into_any_element()
                } else {
                    div().into_any_element()
                }
            }
        </div>
    }
}
