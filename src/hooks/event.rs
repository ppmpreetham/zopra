use gpui::{App, Entity, EventEmitter};

/// # Custom Hook
/// Subscribes to events from a publisher and calls the function when an event is emitted.
///
/// # Example
///```rust
/// #[component]
/// pub fn current_count(counter: &Entity<Counter>){
///   use_event(&counter, |event| {
///     println!("{:?}", event);
///   });
/// }
///```
pub fn use_event<T, Evt>(
    publisher: &Entity<T>,
    mut on_event: impl FnMut(&Evt, &mut App) + 'static,
    cx: &mut App,
) where
    T: 'static + EventEmitter<Evt>,
    Evt: 'static,
{
    cx.subscribe(publisher, move |_publisher, event, cx| {
        on_event(event, cx);
    })
    .detach();
}
