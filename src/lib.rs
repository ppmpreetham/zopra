pub mod config;
pub mod hooks;
pub mod utils;

extern crate self as zopra;
pub use gpui_kit;

pub use zopra_gpui_view::rsx as view;
pub use zopra_macros::component;
pub use zopra_macros::signals;
pub mod components;
#[cfg(feature = "wry")]
pub use lb_wry;

#[cfg(feature = "wry")]
pub use components::webview::{WebView, WebViewController, WebViewProps};

pub mod prelude {
    pub use crate::cn;
    pub use crate::components::declarative_table::DeclarativeTableDelegate;
    #[cfg(feature = "wry")]
    pub use crate::hooks::use_webview;
    pub use crate::hooks::{
        Model, ResourceState, Setter, Snap, use_async, use_callback, use_event, use_model,
        use_resource, use_state, use_table, use_window,
    };
    pub use crate::use_effect;
    pub use crate::use_mount;
    pub use crate::view;
    pub use zopra_macros::component;
    pub use zopra_macros::signals;
}
