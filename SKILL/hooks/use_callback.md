# `use_callback`

`use_callback(&entity, handler)` returns a `'static` callback that attempts to update the target entity when called. It holds a weak entity reference, so the callback safely does nothing if the target has already been dropped.

```rust
let on_message = use_callback(&model, |state, message, window, cx| {
    state.handle_message(message, window, cx);
});

view! {
    <button onClick={|| on_message(&message, window, cx)}>"Handle"</button>
}
```

The callback handler receives mutable target state, the event value, the current window, and the target entity context. In a component, Zopra injects context arguments into supported hook calls and normalizes supported view-event closures.
