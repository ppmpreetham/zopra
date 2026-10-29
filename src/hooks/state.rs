use gpui_kit::{App, Window};

use super::{Signal, use_signal};

/// Creates component state as a `(getter, setter)` pair.
///
/// Both halves are the same `Copy` [`Signal`], so either one can read or write.
///
/// # Example
///
/// ```ignore
/// let (count, set_count) = use_state(0);
/// set_count(count() + 1);
/// ```
#[track_caller]
pub fn use_state<T: 'static>(
    init: impl FnOnce() -> T,
    window: &mut Window,
    cx: &mut App,
) -> (Signal<T>, Signal<T>) {
    let signal = use_signal(init, window, cx);
    (signal, signal)
}
