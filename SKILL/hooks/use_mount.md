# `use_mount!` and `use_mount_cx!`

Despite its name, the current `use_mount!(effect)` macro evaluates the expression inline on every component render. It does not retain a mounted flag or schedule deferred work. Use `use_effect!(effect, [])` for an effect that should run after the initial render only, or [`use_effect!`](use_effect.md) with dependencies for reruns.

```rust
use_mount! {
    println!("Component initialized");
}
```

`use_mount_cx!(effect)` also evaluates inline on every render and passes `cx` to the expression:

```rust
use_mount_cx!(|cx| {
    let _ = cx; // Access the component's GPUI App context here.
});
```

This macro is exported at the crate root. `use_mount_cx!` is exported as a macro too, but is not currently re-exported from `zopra::prelude`.
