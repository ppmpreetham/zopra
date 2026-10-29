//! Compile time tracking

/// A required prop that hasn't been set yet.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Missing;

/// A prop that has been set.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Set<T>(pub T);

/// Implemented by `Set<T>` only, so a component renders only once every required prop is set.
#[doc(hidden)]
pub trait Filled {
    type Value;
    fn value(self) -> Self::Value;
}

/// An optional callback prop `Option<C>` for whichever closure type `C` was given
#[doc(hidden)]
pub trait Optional {
    type Inner;
    fn into_option(self) -> Option<Self::Inner>;
}

impl<T> Optional for Option<T> {
    type Inner = T;
    fn into_option(self) -> Option<T> {
        self
    }
}

impl<T> Filled for Set<T> {
    type Value = T;
    fn value(self) -> T {
        self.0
    }
}
