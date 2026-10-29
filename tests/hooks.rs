//! Integration tests for window-dependent hooks (`use_state`, `use_event`, signals)

use gpui_kit::{AppContext, Context, Entity, IntoElement, Render, TestAppContext, Window, div};
use zopra::hooks::{Signal, use_signal, use_state};

/// Renders `use_state` pairs and exposes the signals they return.
struct StateView {
    a: Option<Signal<u32>>,
    b: Option<Signal<u32>>,
}

impl Render for StateView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (a, _) = use_state(|| 10u32, window, cx);
        let (_, b) = use_state(|| 20u32, window, cx);
        self.a = Some(a);
        self.b = Some(b);
        div()
    }
}

fn signals(view: &Entity<StateView>, cx: &gpui_kit::App) -> (Signal<u32>, Signal<u32>) {
    let view = view.read(cx);
    (view.a.unwrap(), view.b.unwrap())
}

#[gpui_kit::test]
async fn use_state_starts_at_the_initial_value(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| StateView { a: None, b: None });
    let (a, b) = cx.cx.read(|cx| signals(&view, cx));
    assert_eq!(cx.cx.read(|cx| (a.get(cx), b.get(cx))), (10, 20));
}

#[gpui_kit::test]
async fn use_state_persists_across_re_renders(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| StateView { a: None, b: None });
    let (a, _) = cx.cx.read(|cx| signals(&view, cx));
    cx.cx.update(|cx| a.set(5u32, cx));
    cx.run_until_parked();

    cx.cx.update(|cx| view.update(cx, |_, cx| cx.notify()));
    cx.run_until_parked();
    let (again, _) = cx.cx.read(|cx| signals(&view, cx));
    assert_eq!(cx.cx.read(|cx| again.get(cx)), 5);
}

#[gpui_kit::test]
async fn use_state_signals_are_independent(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| StateView { a: None, b: None });
    let (a, b) = cx.cx.read(|cx| signals(&view, cx));
    cx.cx.update(|cx| a.set(1u32, cx));
    assert_eq!(cx.cx.read(|cx| (a.get(cx), b.get(cx))), (1, 20));
    cx.cx.update(|cx| b.update(|b| *b += 1, cx));
    assert_eq!(cx.cx.read(|cx| (a.get(cx), b.get(cx))), (1, 21));
}

// #region use_event
struct Emitter;

impl gpui_kit::EventEmitter<u32> for Emitter {}

struct EventView {
    publisher: Entity<Emitter>,
    generation: u32,
    seen: std::rc::Rc<std::cell::RefCell<Vec<(u32, u32)>>>,
}

impl Render for EventView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let seen = std::rc::Rc::clone(&self.seen);
        let generation = self.generation;
        zopra::hooks::use_event(
            &self.publisher,
            move |event: &u32, _cx| seen.borrow_mut().push((generation, *event)),
            window,
            cx,
        );
        div()
    }
}

#[gpui_kit::test]
async fn use_event_subscribes_once_and_uses_the_latest_handler(cx: &mut TestAppContext) {
    let publisher = cx.new(|_| Emitter);
    let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let (view, cx) = cx.add_window_view({
        let publisher = publisher.clone();
        let seen = std::rc::Rc::clone(&seen);
        move |_, _| EventView {
            publisher,
            generation: 0,
            seen,
        }
    });
    cx.run_until_parked();

    for generation in 1..=3 {
        cx.cx.update(|cx| {
            view.update(cx, |view, cx| {
                view.generation = generation;
                cx.notify();
            })
        });
        cx.run_until_parked();
    }

    cx.cx.update(|cx| publisher.update(cx, |_, cx| cx.emit(7u32)));
    assert_eq!(*seen.borrow(), vec![(3, 7)]);
}
// #endregion

// #region signals
#[gpui_kit::test]
async fn disposed_signals_ignore_writes(cx: &mut TestAppContext) {
    cx.update(|cx| {
        let signal = Signal::new(1u32, None, cx);
        signal.set(2u32, cx);
        assert_eq!(signal.get(cx), 2);

        signal.dispose(cx);
        assert!(signal.try_read(cx).is_none());
        assert!(signal.try_update(|value| *value = 3, cx).is_none());

        // the slot is reused, but the old handle stays dead
        let reused = Signal::new(5u32, None, cx);
        assert!(signal.try_read(cx).is_none());
        assert_eq!(reused.get(cx), 5);
    });
}

struct MountView {
    mounted: bool,
    signal: Option<Signal<u32>>,
}

impl Render for MountView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.mounted {
            self.signal = Some(use_signal(|| 7, window, cx));
        }
        div()
    }
}

#[gpui_kit::test]
async fn signals_are_disposed_when_their_component_unmounts(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| MountView {
        mounted: true,
        signal: None,
    });
    cx.run_until_parked();
    let signal = cx.cx.read(|cx| view.read(cx).signal).unwrap();
    assert_eq!(cx.cx.read(|cx| signal.get(cx)), 7);

    cx.cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.mounted = false;
            cx.notify();
        })
    });
    cx.run_until_parked();
    // GPUI releases entities dropped by a frame on its next effect flush
    cx.cx.update(|_| {});
    assert!(cx.cx.read(|cx| signal.try_read(cx).is_none()));
}
// #endregion
