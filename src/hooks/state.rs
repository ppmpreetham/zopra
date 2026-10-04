use std::{borrow::Borrow, fmt, ops::Deref, rc::Rc};
use gpui_kit::{App, Entity, IntoElement, Window};

#[derive(Debug)]
pub struct Snap<T>(pub(crate) Rc<T>);

impl<T> Clone for Snap<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T> Deref for Snap<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

use fmt::{Display, Formatter, Result};
impl<T: Display> Display for Snap<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Display::fmt(&**self, f)
    }
}

impl<T> Borrow<T> for Snap<T> {
    fn borrow(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T: PartialEq, Rhs: Borrow<T>> PartialEq<Rhs> for Snap<T> {
    fn eq(&self, other: &Rhs) -> bool {
        **self == *other.borrow()
    }
}

impl PartialEq<Snap<bool>> for bool {
    fn eq(&self, other: &Snap<bool>) -> bool {
        *self == **other
    }
}

impl<T: PartialOrd + PartialEq, Rhs: Borrow<T>> PartialOrd<Rhs> for Snap<T> {
    fn partial_cmp(&self, other: &Rhs) -> Option<std::cmp::Ordering> {
        (**self).partial_cmp(other.borrow())
    }
}

impl<T: fmt::Display + 'static> IntoElement for Snap<T> {
    type Element = <String as IntoElement>::Element;

    fn into_element(self) -> Self::Element {
        self.to_string().into_element()
    }
}

impl<T: Clone> Snap<T> {
    pub fn into_inner(self) -> T {
        Rc::unwrap_or_clone(self.0)
    }
}

impl<T> Snap<T> {
    pub fn into_rc(self) -> Rc<T> {
        self.0
    }
}

/// A cheap-to-clone state update handle.
#[derive(Debug)]
pub struct Setter<T>(Entity<Rc<T>>);

impl<T> Clone for Setter<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// this is some extra api i'm working on (experimental)
impl<T: Clone + PartialEq + 'static> Setter<T> {
    /// Replaces the state and notifies the component only when the value differs.
    pub fn set(&self, value: impl Into<T>, cx: &mut App) {
        let value = value.into();

        self.0.update(cx, |state, cx| {
            if **state != value {
                *state = Rc::new(value);
                cx.notify();
            }
        });
    }

    /// Mutates the current value & notifies the component.
    pub fn update(&self, f: impl FnOnce(&mut T), cx: &mut App) {
        self.0.update(cx, |state, cx| {
            f(Rc::make_mut(state));
            cx.notify();
        });
    }

    /// Reads the latest value
    pub fn current(&self, cx: &App) -> Snap<T> {
        Snap(self.0.read(cx).clone())
    }
}

/// Creates component state and returns its snapshot, updater.
///
/// # Example
///
/// ```ignore
/// let (count, set_count) = use_state(0);
/// set_count(*count + 1);
/// ```
#[track_caller]
pub fn use_state<T: Clone + PartialEq + 'static>(
    init: impl FnOnce() -> T,
    window: &mut Window,
    cx: &mut App,
) -> (Snap<T>, Setter<T>) {
    let entity: Entity<Rc<T>> = window.use_state(cx, |_window, _cx| Rc::new(init()));

    (Snap(entity.read(cx).clone()), Setter(entity))
}
