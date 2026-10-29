use gpui_kit::{App, Entity, Window};
use gpui_kit::base::input::InputState;

/// Creates a persistent `InputState` entity for use with text inputs.
///
/// Text inputs in GPUI are complex state machines that manage their own cursor position,
/// text selection, and focus. This hook ensures the input state is created exactly once
/// and persists across render passes.
///
/// # Example
///
/// ```ignore
/// let input = use_input(window, cx);
///
/// view! {
///     <input state={&input} class="py-2 px-4" />
/// }
/// ```
#[track_caller]
pub fn use_input(window: &mut Window, cx: &mut App) -> Entity<InputState> {
    window.use_state(cx, |window, cx| InputState::new(window, cx))
}
