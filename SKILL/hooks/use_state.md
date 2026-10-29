# `use_state`

`use_state(init)` stores local component state and returns a `(getter, setter)` pair. Both halves are the same `Signal<T>`: a `Copy` handle, so closures capture it without cloning, and either half can read or write. The initializer runs on mount; `T` only needs `'static`.

```rust
let (count, set_count) = use_state(0);
let (draft, set_draft) = use_state(String::new());

view! {
    <button onClick={|| set_count(count() + 1)}>{count().to_string()}</button>
    <input value={draft()} onChange={set_draft} />
    <button onClick={|| set_draft(|text| text.push('!'))}>"Append"</button>
}
```

Inside a `#[component]` the macro passes `cx` for you:

| You write | It becomes | |
|---|---|---|
| `count()` | `count.get(cx)` | clone of the value (`T: Clone`) |
| `count.read()` | `count.read(cx)` | `&T`, no clone; drop it before writing |
| `set_count(v)` | `set_count.set(v, cx)` | replace and re-render, unless equal (`T: PartialEq`) |
| `set_count(\|v\| ..)` | `set_count.update(.., cx)` | mutate in place and re-render |

Reads always see the latest value, so handlers and async tasks never hold a stale snapshot. Outside a component, call the methods with `cx` yourself. `use_signal(init)` returns a single signal when you don't need the pair. Signals are not `PartialEq` themselves: compare and depend on values (`[count()]`), not handles.

A signal is disposed when its component unmounts: `read`/`get` then panic, `set`/`update` do nothing, and `try_update` returns `None` so a long-running task can stop when its component goes away. A string literal initializer is converted to `SharedString`; use `String::new()` to edit a `String` in place.
