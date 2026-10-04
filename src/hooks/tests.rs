//! Unit tests for the hooks

//! NOTE: if you want to look at window dependent hooks (`use_state`, `use_callback`),
//! they are covered by the integration tests in `tests/hooks.rs`.

use gpui_kit::{AppContext, Entity, EventEmitter, TestAppContext};
use std::{cell::RefCell, rc::Rc};

use super::run_effect;
use super::{use_async, use_event};

struct Emitter;

impl EventEmitter<Ping> for Emitter {}

#[derive(Clone, Copy, PartialEq, Debug)]
struct Ping(u32);

// #region use_effect
#[gpui_kit::test]
async fn effect_runs_on_mount_and_when_value_dependencies_change(cx: &mut TestAppContext) {
    let runs = Rc::new(RefCell::new(Vec::<u32>::new()));
    let window = cx.add_empty_window();

    for dependency in [1, 1, 2] {
        let runs = Rc::clone(&runs);
        window.update(|window, app| {
            run_effect(
                dependency,
                move || runs.borrow_mut().push(dependency),
                window,
                app,
            )
        });
    }
    cx.run_until_parked();

    assert_eq!(*runs.borrow(), vec![1, 2]);
}
// #endregion

//#region use_event
#[gpui_kit::test]
async fn use_event_invokes_callback_for_each_emitted_event(cx: &mut TestAppContext) {
    let got = Rc::new(RefCell::new(Vec::<u32>::new()));
    let publisher: Entity<Emitter> = cx.new(|_| Emitter);
    cx.update(|cx| {
        let got = Rc::clone(&got);
        use_event(
            &publisher,
            move |event, _cx| got.borrow_mut().push(event.0),
            cx,
        );
    });

    assert_eq!(*got.borrow(), Vec::<u32>::new());
    cx.update(|cx| publisher.update(cx, |_, cx| cx.emit(Ping(3))));
    assert_eq!(*got.borrow(), vec![3]);
    cx.update(|cx| publisher.update(cx, |_, cx| cx.emit(Ping(4))));
    assert_eq!(*got.borrow(), vec![3, 4]);
}

#[gpui_kit::test]
async fn use_event_stops_after_publisher_is_released(cx: &mut TestAppContext) {
    let got = Rc::new(RefCell::new(Vec::<u32>::new()));
    let publisher: Entity<Emitter> = cx.new(|_| Emitter);
    cx.update(|cx| {
        let got = Rc::clone(&got);
        use_event(
            &publisher,
            move |event, _cx| got.borrow_mut().push(event.0),
            cx,
        );
    });

    cx.update(|cx| publisher.update(cx, |_, cx| cx.emit(Ping(1))));
    assert_eq!(*got.borrow(), vec![1]);

    drop(publisher);
    cx.run_until_parked();
    assert_eq!(*got.borrow(), vec![1]);
}
// #endregion

// #region use_async
struct AsyncThing;
struct AsyncThingWithValue(u32);

#[gpui_kit::test]
async fn use_async_returns_task_with_value(cx: &mut TestAppContext) {
    let thing: Entity<AsyncThing> = cx.new(|_| AsyncThing);
    let task = thing.update(cx, |_, cx| use_async(async move |_weak, _cx| 40u32 + 2, cx));
    assert_eq!(task.await, 42);
}

#[gpui_kit::test]
async fn use_async_tasks_can_update_their_entity(cx: &mut TestAppContext) {
    let thing: Entity<AsyncThingWithValue> = cx.new(|_| AsyncThingWithValue(0));
    let task = thing.update(cx, |_, cx| {
        use_async(
            async move |weak, cx| {
                weak.update(cx, |state, cx| {
                    state.0 = 9;
                    cx.notify();
                })
                .unwrap();
                9u32
            },
            cx,
        )
    });
    assert_eq!(task.await, 9);
    assert_eq!(cx.read(|cx| thing.read(cx).0), 9);
}
// #endregion
