use gpui_kit::{AnyElement, App, IntoElement, Window};

/// The content between a component's tags.
///
/// Declare it as the `children` prop and place it with `{children}`. It renders inside this
/// component's render, so it sees the contexts this component provides. Parents whose children
/// are a specific type (like a tab bar's tabs) declare `children: Children<Tab>`.
///
/// ```ignore
/// #[component]
/// fn card(title: SharedString, children: Children) {
///     view! { <div>{title} {children}</div> }
/// }
/// ```
pub trait Children<T = AnyElement>: FnOnce(&mut Window, &mut App) -> Vec<T> {}

impl<T, F: FnOnce(&mut Window, &mut App) -> Vec<T>> Children<T> for F {}

/// Converts one piece of content into the child type a parent collects.
///
/// Elements become `AnyElement`. A typed child (e.g. a `Tab`) converts only into itself;
/// implement `FromChild<MyChild> for MyChild` for your own typed children.
pub trait FromChild<X> {
    fn from_child(child: X) -> Self;
}

impl<X: IntoElement> FromChild<X> for AnyElement {
    fn from_child(child: X) -> Self {
        child.into_any_element()
    }
}

/// typed child that converts only into itself
#[macro_export]
macro_rules! typed_child {
    ($($ty:ty),* $(,)?) => {
        $(impl $crate::FromChild<$ty> for $ty {
            fn from_child(child: $ty) -> Self {
                child
            }
        })*
    };
}

#[doc(hidden)]
pub fn no_children<T>(_window: &mut Window, _cx: &mut App) -> Vec<T> {
    Vec::new()
}
