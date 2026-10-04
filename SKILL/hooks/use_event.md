# `use_event`

`use_event(&entity, handler)` subscribes to events emitted by a GPUI entity implementing `EventEmitter<Evt>`. The subscription is tied to the current component's GPUI context.

```rust
use_event(&publisher, |event| {
    println!("Received: {event:?}");
});
```

Inside a component, Zopra supplies the app context to the hook and handler. Use this for GPUI entity events; use an element event attribute such as `onClick` for input originating from a view element.
