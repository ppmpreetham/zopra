use gpui_kit::prelude::*;
use gpui_kit::div;

use crate::{Children, component};

/// Shows its children while `when` is true. With `mode="hidden"`, hidden children stay
/// rendered but invisible, so their state survives (like React's `<Activity>`).
#[component]
pub fn activity(when: bool, mode: Option<&'static str>, children: Children) {
    if when {
        div().children(children())
    } else if mode == Some("hidden") {
        div().invisible().children(children())
    } else {
        div()
    }
}
