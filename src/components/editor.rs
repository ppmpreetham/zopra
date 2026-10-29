use gpui_kit::component::input::{Editor as GpuiEditor, EditorState};
use gpui_kit::prelude::*;
use gpui_kit::{Entity, IntoElement};
use zopra_macros::component;

#[component]
pub fn Editor(
    state: &Entity<EditorState>,
    language: Option<&'static str>,
    line_numbers: Option<bool>,
    folding: Option<bool>,
    auto_close: Option<bool>,
    smart_indent: Option<bool>,
    readonly: Option<bool>,
    searchable: Option<bool>,
    show_whitespaces: Option<bool>,
) {
    state.update(cx, |ed, cx| {
        if let Some(l) = language {
            ed.set_highlighter(l, cx);
        }
        if let Some(ln) = line_numbers {
            ed.set_line_number(ln, window, cx);
        }
        if let Some(f) = folding {
            ed.set_folding(f, window, cx);
        }
        if let Some(ac) = auto_close {
            ed.set_auto_close(ac, window, cx);
        }
        if let Some(si) = smart_indent {
            ed.set_smart_indent(si, window, cx);
        }
        if let Some(r) = readonly {
            ed.set_readonly(r, cx);
        }
        if let Some(s) = searchable {
            ed.set_searchable(s, cx);
        }
        if let Some(sw) = show_whitespaces {
            ed.set_show_whitespaces(sw, window, cx);
        }
    });

    GpuiEditor::new(state).size_full().into_any_element()
}

