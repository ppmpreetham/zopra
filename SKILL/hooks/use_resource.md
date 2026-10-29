# `use_resource`

`use_resource(deps, load)` starts an async load on mount and again when the dependency value changes. `deps` must be `PartialEq + 'static`. The hook returns a `Signal<ResourceState<T, E>>`, where the state is `Loading`, `Ready(value)`, or `Failed(error)`.

```rust
let user = use_resource(user_id, || async move {
    fetch_user(user_id).await
});

match user.read() {
    ResourceState::Loading => view! { <div>"Loading…"</div> },
    ResourceState::Ready(user) => view! { <div>{user.name.clone()}</div> },
    ResourceState::Failed(error) => view! { <div>{error.to_string()}</div> },
}
```

When a dependency changes, the state returns to `Loading`. A completion from an older request is ignored after a newer request has started. Keep dependencies limited to values that should trigger a reload.
