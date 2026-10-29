//! Regression tests for signal, context, key and macro edge cases.

use std::{cell::RefCell, rc::Rc};

use gpui_kit::{
    Context, IntoElement, ParentElement, Render, TestAppContext, Window, div,
};
use zopra::components::declarative_table::{DeclarativeTableDelegate, Rows};
use zopra::hooks::{Signal, provide_context, try_use_context, use_signal, use_state};
use zopra::{component, view};

type Log<T> = Rc<RefCell<Vec<T>>>;

// #region set skips equal values
struct RenderCount {
    renders: usize,
    signal: Option<Signal<u32>>,
}

impl Render for RenderCount {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.renders += 1;
        let signal = use_signal(|| 0, window, cx);
        // unconditional, but equal: must not schedule another render
        signal.set(0u32, cx);
        self.signal = Some(signal);
        div()
    }
}

#[gpui_kit::test]
async fn setting_an_equal_value_does_not_rerender(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| RenderCount {
        renders: 0,
        signal: None,
    });
    cx.run_until_parked();
    assert_eq!(cx.cx.read(|cx| view.read(cx).renders), 1);

    let signal = cx.cx.read(|cx| view.read(cx).signal).unwrap();
    cx.cx.update(|cx| signal.set(1u32, cx));
    cx.run_until_parked();
    assert_eq!(cx.cx.read(|cx| view.read(cx).renders), 2);
}
// #endregion

// #region components rendered in a loop keep separate state
#[component]
fn item(log: Log<String>) {
    let (count, _) = use_state(0);
    log.borrow_mut().push(format!("{count:?}"));
    div()
}

#[component]
fn list(log: Log<String>, keyed: bool) {
    view! {
        <div>
            {...(0..3usize).map(|i| if keyed {
                view! { <Item key={i} log={log.clone()} /> }
            } else {
                view! { <Item log={log.clone()} /> }
            })}
        </div>
    }
}

struct ListView {
    log: Log<String>,
    keyed: bool,
}

impl Render for ListView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        list(self.log.clone(), self.keyed, window, cx)
    }
}

#[gpui_kit::test]
async fn looped_components_get_their_own_signals(cx: &mut TestAppContext) {
    for keyed in [false, true] {
        let log = Log::default();
        let (_view, cx) = cx.add_window_view(|_, _| ListView {
            log: log.clone(),
            keyed,
        });
        cx.run_until_parked();
        let mut signals = log.borrow().clone();
        signals.dedup();
        assert_eq!(signals.len(), 3, "keyed: {keyed}, signals: {signals:?}");
    }
}
// #endregion

// #region context is scoped to the provider's subtree
#[component]
fn reader(log: Log<Option<u32>>) {
    log.borrow_mut().push(try_use_context::<u32>());
    div()
}

#[component]
fn provider(value: u32, log: Log<Option<u32>>, nested: bool) {
    provide_context(value);
    view! {
        <div>
            <Reader log={log.clone()} />
            {if nested {
                view! { <Provider value={value + 1} log={log.clone()} nested={false} /> }.into_any_element()
            } else {
                div().into_any_element()
            }}
            <Reader log={log.clone()} />
        </div>
    }
}

#[component]
fn context_tree(log: Log<Option<u32>>) {
    view! {
        <div>
            <Provider value={1u32} log={log.clone()} nested={true} />
            <Reader log={log.clone()} />
        </div>
    }
}

struct ContextView(Log<Option<u32>>);

impl Render for ContextView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        context_tree(self.0.clone(), window, cx)
    }
}

#[gpui_kit::test]
async fn context_is_scoped_to_the_subtree(cx: &mut TestAppContext) {
    let log = Log::default();
    let (_view, cx) = cx.add_window_view(|_, _| ContextView(log.clone()));
    cx.run_until_parked();
    // outer reader, nested provider's two readers, outer reader after nesting, sibling outside
    assert_eq!(
        *log.borrow(),
        [Some(1), Some(2), Some(2), Some(1), None]
    );
}
// #endregion

// #region the macro respects shadowing inside view!
#[component]
fn shadowed_in_view() {
    let (count, _) = use_state(5);
    let double = |x: i32| x * 2;
    view! {
        <div>
            // `count` here is the closure parameter, not the signal
            {[double].iter().map(|count| count(1)).sum::<i32>().to_string()}
            {count().to_string()}
        </div>
    }
}

struct ShadowView;

impl Render for ShadowView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        shadowed_in_view(window, cx)
    }
}

#[gpui_kit::test]
async fn view_expressions_respect_shadowing(cx: &mut TestAppContext) {
    let (_view, cx) = cx.add_window_view(|_, _| ShadowView);
    cx.run_until_parked();
}
// #endregion

// #region tables re-sort only when their signal changed
#[gpui_kit::test]
async fn signal_rows_update_only_after_a_write(cx: &mut TestAppContext) {
    cx.update(|cx| {
        let rows = Signal::new(vec![3, 1, 2], None, cx);
        let mut table = DeclarativeTableDelegate::new(Rows::from(rows), Vec::new(), |_: &i32, _| {
            div().into_any_element()
        });
        assert!(!table.update_data(Rows::from(rows), cx));
        rows.update(|rows| rows.push(4), cx);
        assert!(table.update_data(Rows::from(rows), cx));
        assert!(!table.update_data(Rows::from(rows), cx));
        assert!(table.update_data(Rows::from(vec![1]), cx));
    });
}
// #endregion
