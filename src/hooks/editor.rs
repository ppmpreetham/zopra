use gpui_kit::{App, Entity, Window};
use gpui_kit::component::input::{EditorState, RangeDecoration, RangeDecorationCollection, TextDecoration, TextDecorationCollection};

#[track_caller]
pub fn use_editor(window: &mut Window, cx: &mut App) -> Entity<EditorState> {
    window.use_state(cx, |window, cx| EditorState::new(window, cx))
}

#[track_caller]
pub fn use_editor_ranges(
    window: &mut Window,
    cx: &mut App,
    editor: &Entity<EditorState>,
    decorations: Vec<RangeDecoration>,
) {
    let collection = window.use_state(cx, |_, _| None::<RangeDecorationCollection>);
    let prev_decorations = window.use_state(cx, |_, _| Vec::<RangeDecoration>::new());

    let mut is_different = false;
    prev_decorations.update(cx, |prev, _| {
        if *prev != decorations {
            *prev = decorations.clone();
            is_different = true;
        }
    });

    if is_different {
        let needs_creation = collection.update(cx, |col, cx| {
            if let Some(c) = col {
                c.set(decorations.clone(), cx);
                false
            } else {
                true
            }
        });

        if needs_creation {
            let col = editor.update(cx, |ed, cx| {
                ed.create_range_decorations_collection(decorations, cx)
            });
            collection.update(cx, |c, _| *c = Some(col));
        }
    }
}

/// Automatically syncs a list of `TextDecoration`s to the editor.
#[track_caller]
pub fn use_editor_text_styles(
    window: &mut Window,
    cx: &mut App,
    editor: &Entity<EditorState>,
    decorations: Vec<TextDecoration>,
) {
    let collection = window.use_state(cx, |_, _| None::<TextDecorationCollection>);
    let prev_decorations = window.use_state(cx, |_, _| Vec::<TextDecoration>::new());

    let mut is_different = false;
    prev_decorations.update(cx, |prev, _| {
        if *prev != decorations {
            *prev = decorations.clone();
            is_different = true;
        }
    });

    if is_different {
        let needs_creation = collection.update(cx, |col, cx| {
            if let Some(c) = col {
                c.set(decorations.clone(), cx);
                false
            } else {
                true
            }
        });

        if needs_creation {
            let col = editor.update(cx, |ed, cx| {
                ed.create_decorations_collection(decorations, cx)
            });
            collection.update(cx, |c, _| *c = Some(col));
        }
    }
}
