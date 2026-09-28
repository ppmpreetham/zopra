
use gpui_kit::{App, Entity, Window, AppContext};
use gpui_kit::component::table::{TableState, TableDelegate};

/// Lazily initializes a TableState entity exactly once, managing it in GPUI.
///
/// # Example
///
/// ```ignore
/// let state = use_table(Deletegate::new());
/// ```
#[track_caller]
pub fn use_table<D: TableDelegate + 'static, F: FnOnce() -> D>(
    init_delegate: F,
    window: &mut Window,
    cx: &mut App,
) -> Entity<TableState<D>> {
    let entity: Entity<Option<Entity<TableState<D>>>> = window.use_state(cx, |_window, _cx| None);

    let state_entity_opt = entity.read(cx).clone();

    if let Some(state_entity) = state_entity_opt {
        state_entity
    } else {
        let delegate = init_delegate();
        let new_state_entity = cx.new(|cx| TableState::new(delegate, window, cx));

        entity.update(cx, |state, _cx| {
            *state = Some(new_state_entity.clone());
        });

        new_state_entity
    }
}
