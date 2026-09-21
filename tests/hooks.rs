//! Integration tests for window-dependent hooks (`use_state`,
//! `use_callback`)

use gpui::{App, AppContext, Context, Entity, IntoElement, Render, TestAppContext, Window, div};
use zopra::hooks::{use_callback, use_state};

struct SignalView {
    get: Option<Box<dyn Fn(&App) -> u32>>,
    set: Option<Box<dyn Fn(u32, &mut App)>>,
}

impl SignalView {
    fn new() -> Self {
        Self { get: None, set: None }
    }
}

impl Render for SignalView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (get, set) = use_state(0u32, window, cx);
        self.get = Some(Box::new(get));
        self.set = Some(Box::new(set));
        div()
    }
}

struct TwoSignalsView {
    a_get: Option<Box<dyn Fn(&App) -> u32>>,
    a_set: Option<Box<dyn Fn(u32, &mut App)>>,
    b_get: Option<Box<dyn Fn(&App) -> u32>>,
    b_set: Option<Box<dyn Fn(u32, &mut App)>>,
}

impl TwoSignalsView {
    fn new() -> Self {
        Self { a_get: None, a_set: None, b_get: None, b_set: None }
    }
}

impl Render for TwoSignalsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (get_a, set_a) = use_state(10u32, window, cx);
        let (get_b, set_b) = use_state(20u32, window, cx);
        self.a_get = Some(Box::new(get_a));
        self.a_set = Some(Box::new(set_a));
        self.b_get = Some(Box::new(get_b));
        self.b_set = Some(Box::new(set_b));
        div()
    }
}

struct Holder(u32);

fn get_signal(view: &Entity<SignalView>, cx: &App) -> u32 {
    (view.read(cx).get.as_ref().unwrap())(cx)
}

fn set_signal(view: &Entity<SignalView>, value: u32, cx: &mut App) {
    view.update(cx, |view, cx| (view.set.as_ref().unwrap())(value, cx));
}

#[gpui::test]
async fn use_state_starts_at_the_initial_value(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| SignalView::new());
    assert_eq!(cx.cx.read(|cx| get_signal(&view, cx)), 0);
}

#[gpui::test]
async fn use_state_setter_updates_the_value(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| SignalView::new());
    cx.cx.update(|cx| set_signal(&view, 5, cx));
    assert_eq!(cx.cx.read(|cx| get_signal(&view, cx)), 5);
    cx.cx.update(|cx| set_signal(&view, 9, cx));
    assert_eq!(cx.cx.read(|cx| get_signal(&view, cx)), 9);
}

#[gpui::test]
async fn use_state_persists_across_re_renders(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| SignalView::new());
    cx.cx.update(|cx| set_signal(&view, 5, cx));
    assert_eq!(cx.cx.read(|cx| get_signal(&view, cx)), 5);

    cx.cx.update(|cx| view.update(cx, |_, cx| cx.notify()));
    cx.run_until_parked();
    assert_eq!(cx.cx.read(|cx| get_signal(&view, cx)), 5);
}

#[gpui::test]
async fn two_use_state_signals_are_independent(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| TwoSignalsView::new());
    let read_b = |cx: &App| (view.read(cx).b_get.as_ref().unwrap())(cx);

    cx.cx.update(|cx| view.update(cx, |view, cx| (view.a_set.as_ref().unwrap())(1, cx)));
    assert_eq!(cx.cx.read(read_b), 20);

    cx.cx.update(|cx| view.update(cx, |view, cx| (view.b_set.as_ref().unwrap())(2, cx)));
    assert_eq!(cx.cx.read(read_b), 2);
    let read_a = |cx: &App| (view.read(cx).a_get.as_ref().unwrap())(cx);
    assert_eq!(cx.cx.read(read_a), 1);
}

#[gpui::test]
async fn use_callback_updates_the_bound_entity(cx: &mut TestAppContext) {
    let holder: Entity<Holder> = cx.new(|_| Holder(0));
    let bump = use_callback(&holder, |holder: &mut Holder, amount: &u32, _window, _cx| {
        holder.0 += *amount;
    });

    let visual = cx.add_empty_window();
    visual.update(|window, cx| bump(&3, window, cx));
    assert_eq!(visual.cx.read(|cx| holder.read(cx).0), 3);
    visual.update(|window, cx| bump(&4, window, cx));
    assert_eq!(visual.cx.read(|cx| holder.read(cx).0), 7);
}

#[gpui::test]
async fn use_callback_is_a_noop_after_the_entity_is_released(cx: &mut TestAppContext) {
    let holder: Entity<Holder> = cx.new(|_| Holder(0));
    let bump = use_callback(&holder, |holder: &mut Holder, _amount: &u32, _window, _cx| {
        holder.0 += 1;
    });
    drop(holder);
    cx.run_until_parked();

    let visual = cx.add_empty_window();
    visual.update(|window, cx| bump(&1, window, cx));
}
