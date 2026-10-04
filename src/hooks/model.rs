use std::ops::Deref;

use gpui_kit::{App, Entity, Window};

/// Mutable entity-backed state for large or frequently updated values.
///
/// # Example
///
/// ```ignore
/// let rows = use\_model(|| Vec::<Row>::new());
///
/// rows.update(cx, |rows| {
///     rows.push(Row::default());
/// });
///
/// println!("{} rows", rows.read(cx).len());
/// ```

pub struct Model<T>(Entity<T>);

impl<T> Clone for Model<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: 'static> Model<T> {
    /// Reads the current model.
    pub fn read<'a>(&'a self, cx: &'a App) -> impl Deref<Target = T> + 'a {
        self.0.read(cx)
    }

    /// Mutates the model in place and notifies after the update.
    pub fn update(&self, cx: &mut App, f: impl FnOnce(&mut T)) {
        self.0.update(cx, |value, cx| {
            f(value);
            cx.notify();
        });
    }
}

/// Creates mutable entity-backed state owned by the current component.
///
/// Use this for large or frequently updated data that should not be cloned for
/// render snapshots. Use [`crate::hooks::use_state`] for small values.
///
/// # Example
///
/// ```ignore
/// let rows = use_model(|| Vec::<Row>::new());
/// rows.update(|rows| rows.push(Row::default()));
/// println!("{} rows", rows.read().len());
/// ```
#[track_caller]
pub fn use_model<T: 'static>(
    init: impl FnOnce() -> T,
    window: &mut Window,
    cx: &mut App,
) -> Model<T> {
    Model(window.use_state(cx, |_window, _cx| init()))
}
