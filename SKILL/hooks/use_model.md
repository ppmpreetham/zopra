# `use_model`

`use_model(init)` stores an entity-backed value and returns a cloneable `Model<T>`. It does not require `T: Clone` or `T: PartialEq`, and it does not produce render snapshots. Choose it for large or frequently updated data that should be mutated directly.

```rust
let rows = use_model(|| Vec::<Row>::new());
rows.update(|rows| rows.push(Row::default()));
let row_count = rows.read().len();
```

`read()` returns a guard tied to the current GPUI app context. `update()` mutates the value and notifies GPUI. In async code or other contexts without injected hook arguments, pass the needed `cx` explicitly to these methods.

For small state where cheap immutable snapshots and equality-based update skipping are useful, use [`use_state`](use_state.md).
