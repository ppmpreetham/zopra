use gpui_kit::prelude::*;
use gpui_kit::{AnyElement, App, Window, div};

/// Conditionally render a component without losing the state
///
/// Synonymous with `<Activity />` from React.
#[derive(Default)]
pub struct ActivityProps {
    when: bool,
    mode: &'static str,
    children: Vec<AnyElement>,
}

impl ActivityProps {
    #[must_use]
    pub fn when(mut self, when: bool) -> Self {
        self.when = when;
        self
    }

    #[must_use]
    pub fn mode(mut self, mode: &'static str) -> Self {
        self.mode = mode;
        self
    }

    #[must_use]
    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    #[track_caller]
    #[must_use]
    pub fn render(self, _window: &mut Window, _cx: &mut App) -> AnyElement {
        if self.when {
            div().children(self.children).into_any_element()
        } else if self.mode == "hidden" {
            div().invisible().children(self.children).into_any_element()
        } else {
            div().into_any_element()
        }
    }
}
