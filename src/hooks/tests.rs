//! Unit tests for the hooks

//! NOTE: window dependent hooks (`use_state`, `use_event`, signals)
//! are covered by the integration tests in `tests/`.

use gpui_kit::{Context, IntoElement, Render, TestAppContext, Window, div};
use std::{cell::RefCell, rc::Rc};

use super::run_effect;

// #region use_effect
// `run_effect` is a render-pass hook: it exercises the same element-state
// keyed `use_state` a component body runs under, so probe it from a `Render`.
struct EffectProbe {
    dep: u32,
    runs: Rc<RefCell<Vec<u32>>>,
}

impl Render for EffectProbe {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let runs = Rc::clone(&self.runs);
        let dep = self.dep;
        run_effect(
            dep,
            move |_window, _cx| runs.borrow_mut().push(dep),
            window,
            cx,
        );
        div()
    }
}

#[gpui_kit::test]
async fn effect_runs_on_mount_and_when_value_dependencies_change(cx: &mut TestAppContext) {
    let runs = Rc::new(RefCell::new(Vec::<u32>::new()));
    let (view, cx) = cx.add_window_view(|_, _| EffectProbe {
        dep: 1,
        runs: Rc::clone(&runs),
    });
    cx.run_until_parked();
    assert_eq!(*runs.borrow(), vec![1]);

    for dependency in [1, 2] {
        cx.cx.update(|cx| {
            view.update(cx, |probe, cx| {
                probe.dep = dependency;
                cx.notify();
            })
        });
        cx.run_until_parked();
    }

    assert_eq!(*runs.borrow(), vec![1, 2]);
}
// #endregion
