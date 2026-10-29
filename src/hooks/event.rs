use gpui_kit::{App, Context, Entity, EntityId, EventEmitter, Subscription, Window};

struct EventHook<F> {
    publisher: EntityId,
    handler: F,
    _subscription: Subscription,
}

/// # Use Event
/// Custom Hook that calls `handler` for every event `publisher` emits.
///
/// It subscribes once per call site (again only if `publisher` changes), while the handler
/// is swapped on every render, so it always sees the latest captured values.
///
/// # Example
///```ignore
/// #[component]
/// pub fn current_count(counter: &Entity<Counter>){
///   use_event(&counter, |event| {
///     println!("{:?}", event);
///   });
/// }
///```
#[track_caller]
pub fn use_event<T, Evt, F>(publisher: &Entity<T>, handler: F, window: &mut Window, cx: &mut App)
where
    T: EventEmitter<Evt>,
    Evt: 'static,
    F: FnMut(&Evt, &mut App) + 'static,
{
    let mut handler = Some(handler);
    let hook = window.use_state(cx, |_window, cx| EventHook {
        publisher: publisher.entity_id(),
        _subscription: subscribe(publisher, cx),
        handler: handler.take().unwrap(),
    });
    hook.update(cx, |hook, cx| {
        if let Some(handler) = handler {
            hook.handler = handler;
        }
        if hook.publisher != publisher.entity_id() {
            hook.publisher = publisher.entity_id();
            hook._subscription = subscribe(publisher, cx);
        }
    });
}

fn subscribe<T, Evt, F>(publisher: &Entity<T>, cx: &mut Context<EventHook<F>>) -> Subscription
where
    T: EventEmitter<Evt>,
    Evt: 'static,
    F: FnMut(&Evt, &mut App) + 'static,
{
    cx.subscribe(publisher, |hook, _publisher, event, cx| (hook.handler)(event, cx))
}
