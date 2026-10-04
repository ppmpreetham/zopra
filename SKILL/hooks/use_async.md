# `use_async`

`use_async(async move |weak_entity, async_cx| ...)` spawns asynchronous work owned by the component entity and returns a GPUI `Task`. The task receives a weak reference to its owner and an `AsyncApp`; it is cancelled when the owner entity is dropped.

```rust
use_async(async move |_weak_entity, async_cx| {
    let result = fetch_data().await;
    async_cx.update(|cx| set_result(result, cx)).ok();
});
```

Use `Setter::current()` inside long-lived work when you need the latest state rather than the snapshot captured when the component rendered. After an `await`, use `AsyncApp::update` to access GPUI state.
