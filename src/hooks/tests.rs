//! Unit tests for the hooks

//! NOTE: if you want to look at window dependent hooks (`use_state`, `use_callback`),
//! they are covered by the integration tests in `tests/hooks.rs`.

use std::{cell::RefCell, rc::Rc};
use gpui_kit::{App, AppContext, Entity, EventEmitter, TestAppContext};

use super::{use_async, use_event};
use crate::use_effect;

struct Counter(u32);

struct Emitter;

impl EventEmitter<Ping> for Emitter {}

#[derive(Clone, Copy, PartialEq, Debug)]
struct Ping(u32);

// #region use_effect
#[gpui_kit::test]
async fn effect_runs_when_dependency_notifies(cx: &mut TestAppContext) {
    let runs = Rc::new(RefCell::new(Vec::<u32>::new()));
    let dep: Entity<Counter> = cx.new(|_| Counter(0));
    let _observer: Entity<Counter> = cx.new(|cx| {
        let runs = Rc::clone(&runs);
        let dep_effect = dep.clone();
        use_effect!(
            move |cx: &mut App| {
                runs.borrow_mut().push(dep_effect.read(cx).0);
            },
            [dep.clone()],
            cx
        );
        Counter(0)
    });

    assert_eq!(*runs.borrow(), Vec::<u32>::new());
    let dep_clone = dep.clone();
    dep.update(cx, |state, cx| {
        state.0 = 7;
        cx.notify();
    });
    let _ = dep_clone;
    assert_eq!(*runs.borrow(), vec![7]);
    let other: Entity<Counter> = cx.new(|_| Counter(0));
    other.update(cx, |_, cx| cx.notify());
    assert_eq!(*runs.borrow(), vec![7]);
}

#[gpui_kit::test]
async fn each_dependency_gets_its_own_effect_runner(cx: &mut TestAppContext) {
    let runs = Rc::new(RefCell::new(0u32));
    let a: Entity<Counter> = cx.new(|_| Counter(0));
    let b: Entity<Counter> = cx.new(|_| Counter(0));
    let _observer: Entity<Counter> = cx.new(|cx| {
        let r1 = Rc::clone(&runs);
        use_effect!(move |_| *r1.borrow_mut() += 1, [a], cx);
        let r2 = Rc::clone(&runs);
        use_effect!(move |_| *r2.borrow_mut() += 1, [b], cx);
        Counter(0)
    });

    assert_eq!(*runs.borrow(), 0);
    a.update(cx, |_, cx| cx.notify());
    assert_eq!(*runs.borrow(), 1);
    b.update(cx, |_, cx| cx.notify());
    assert_eq!(*runs.borrow(), 2);
}

#[gpui_kit::test]
async fn effect_is_safe_after_observer_is_released(cx: &mut TestAppContext) {
    let runs = Rc::new(RefCell::new(0u32));
    let dep: Entity<Counter> = cx.new(|_| Counter(0));
    let observer: Entity<Counter> = cx.new(|cx| {
        let r = Rc::clone(&runs);
        use_effect!(move |_| *r.borrow_mut() += 1, [dep.clone()], cx);
        Counter(0)
    });

    dep.update(cx, |_, cx| cx.notify());
    assert_eq!(*runs.borrow(), 1);

    drop(observer);
    dep.update(cx, |_, cx| cx.notify());
    assert_eq!(*runs.borrow(), 1);
}
// #endregion

//#region use_event
#[gpui_kit::test]
async fn use_event_invokes_callback_for_each_emitted_event(cx: &mut TestAppContext) {
    let got = Rc::new(RefCell::new(Vec::<u32>::new()));
    let publisher: Entity<Emitter> = cx.new(|_| Emitter);
    cx.update(|cx| {
        let got = Rc::clone(&got);
        use_event(&publisher, move |event, _cx| got.borrow_mut().push(event.0), cx);
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
        use_event(&publisher, move |event, _cx| got.borrow_mut().push(event.0), cx);
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
    let task = thing.update(cx, |_, cx| {
        use_async(async move |_weak, _cx| 40u32 + 2, cx)
    });
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
