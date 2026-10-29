use std::{cell::RefCell, rc::Rc};

use gpui_kit::{
    AnyElement, App, Context, ElementId, IntoElement, ParentElement, Render, SharedString,
    TestAppContext, Window, div,
};
use zopra::hooks::{Signal, use_signal};
use zopra::{Component, component};

type Log = Rc<RefCell<Vec<String>>>;

#[component]
fn labelled(#[into] tags: Option<Vec<String>>, key: Option<ElementId>, log: Log) {
    log.borrow_mut().push(format!(
        "{:?} {}",
        tags.unwrap_or_default(),
        key.map(|key| key.to_string()).unwrap_or_default()
    ));
    div()
}

#[component]
fn shown(text: SharedString) {
    div().child(text)
}

#[derive(Clone, Copy)]
struct Counter(Signal<usize>);

impl Counter {
    fn bump(self, cx: &mut App) -> usize {
        self.0.update_untracked(|n| *n += 1, cx);
        *self.0.read(cx)
    }
}

#[track_caller]
fn use_counter(window: &mut Window, cx: &mut App) -> Counter {
    Counter(use_signal(|| 0, window, cx))
}

#[component]
fn counted(log: Log) {
    let counter = use_counter();
    let count = counter.bump();
    log.borrow_mut().push(count.to_string());
    div()
}

fn erased(props: impl Component, window: &mut Window, cx: &mut App) -> AnyElement {
    props.render_component(window, cx)
}

struct Host(Log);

impl Render for Host {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let log = self.0.clone();
        let props = Shown::new().text("hi");
        assert!(props.clone() == props);
        assert!(Shown::new().text("a") != Shown::new().text("b"));
        div()
            .child(Labelled::new().tags(["a".to_string()]).log(log.clone()).key("k").render(window, cx))
            .child(Counted::new().log(log).render(window, cx))
            .child(erased(props, window, cx))
    }
}

#[gpui_kit::test]
async fn macro_extensions_work(cx: &mut TestAppContext) {
    let log = Log::default();
    let (view, cx) = cx.add_window_view(|_, _| Host(log.clone()));
    cx.run_until_parked();
    view.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    assert_eq!(*log.borrow(), [r#"["a"] k"#, "1", r#"["a"] k"#, "2"]);
}
