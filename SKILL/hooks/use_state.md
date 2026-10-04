# `use_state`

`use_state(init)` stores local component state and returns `(Snap<T>, Setter<T>)`. The initializer runs on mount. `T` must implement `Clone + PartialEq`: snapshots share an `Rc` until mutation, and `set` skips an equal replacement.

```rust
let (count, set_count) = use_state(0);
let (draft, set_draft) = use_state(String::new());

view! {
    <button onClick={|| set_count(*count + 1)}>{count}</button>
    <input value={&draft} onChange={set_draft} />
    <button onClick={|| set_draft(|text| text.push('!'))}>"Append"</button>
}
```

`Snap<T>` dereferences to `T`; field access, display, comparisons, iteration, and `len()` work directly. Arithmetic generally needs an explicit dereference because operators consume their operands, for example `*count + 1`.

`Setter::set(value)` replaces the value and notifies only when it differs. `Setter::update(|value| ...)` mutates with copy-on-write and notifies. A string literal initializer is converted to `SharedString`; use `String::new()` if the setter closure needs to edit a `String` in place. `Setter::current()` reads the latest value for long-lived async work.

Derive `Clone` and `PartialEq` on custom state types. Use [`use_model`](use_model.md) when those bounds are unsuitable or snapshots would be too costly.
