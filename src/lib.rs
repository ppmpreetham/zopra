pub mod config;
pub mod hooks;
pub mod utils;
pub mod css;
mod children;
mod props;
mod scope;

pub use children::{Children, FromChild};

pub trait Component {
    fn render_component(self, window: &mut gpui_kit::Window, cx: &mut gpui_kit::App) -> gpui_kit::AnyElement;
}

/// A component's required props are checked when it compiles.
///
/// ```compile_fail,E0277
/// use gpui_kit::ParentElement;
/// use zopra::component;
///
/// #[component]
/// fn greet(name: String) {
///     gpui_kit::div().child(name)
/// }
///
/// fn render(window: &mut gpui_kit::Window, cx: &mut gpui_kit::App) {
///     // error[E0277]: `<Greet>` is missing the required prop `name`
///     let _ = Greet::new().render(window, cx);
/// }
/// ```
#[doc(hidden)]
pub mod required_props_are_checked {}

/// A `move` closure needs its own GPUI context to read a signal; zopra says so.
///
/// ```compile_fail
/// use gpui_kit::{InteractiveElement, StatefulInteractiveElement, div};
/// use zopra::{component, hooks::use_state};
///
/// #[component]
/// fn counter() {
///     let (count, set_count) = use_state(0);
///     // error: `set_count` needs a GPUI context, but this `move` closure has none
///     div().id("counter").on_click(move |_, _, _| set_count(count() + 1))
/// }
/// ```
#[doc(hidden)]
pub mod closures_need_a_context {}

/// An unknown class is a compile error that names it.
///
/// ```compile_fail
/// use gpui_kit::Styled;
///
/// fn render() -> gpui_kit::Div {
///     // error: Unknown class `flex-cols`. If it's a GPUI style method, use it as an attribute instead: `<div flex_cols />`.
///     zopra::view! { <div class="flex flex-cols" /> }
/// }
/// ```
#[doc(hidden)]
pub mod unknown_classes_are_named {}

/// Used by code that `#[component]` and `view!` generate.
#[doc(hidden)]
pub mod __private {
    pub use crate::children::no_children;
    pub use crate::hooks::table::{use_table, use_table_with};
    pub use crate::props::{Filled, Missing, Optional, Set};
    pub use crate::scope::{enter, exit, occurrence};
}

extern crate self as zopra;
pub use gpui_kit;

pub use zopra_gpui_view::rsx as view;
pub use zopra_macros::component;
pub use zopra_macros::signals;
pub mod components;
pub mod ui;
#[cfg(feature = "wry")]
pub use lb_wry;

#[cfg(feature = "wry")]
pub use components::webview::{WebView, WebViewController, WebViewProps};

pub mod prelude {
    pub use crate::cn;
    pub use crate::hooks::{
        ResourceState, Signal, WindowLauncher, provide_context, try_use_context, use_context,
        use_event, use_resource, use_signal, use_state, use_window,
    };
    #[cfg(feature = "wry")]
    pub use crate::hooks::use_webview;
    pub use crate::use_effect;
    pub use crate::view;
    pub use crate::{Children, Component, FromChild};
    pub use zopra_macros::component;
    pub use zopra_macros::signals;
}
