# `use_effect!`

`use_effect!(effect, [deps...])` defers the effect until after rendering. It runs on mount and again when the dependency tuple changes. Dependency values must be `Clone + PartialEq + 'static`; capture values used by the effect explicitly. Call it unconditionally at the top level of a component, not inside a branch or closure.

```rust
use_effect!(|| println!("Count is now {count}"), [count]);
```

The component macro supplies `window` and `cx`. Outside a component, `use_effect!` emits a focused compile error; call `zopra::hooks::run_effect(deps, effect, window, cx)` directly when working with GPUI contexts manually. The effect closure is one-shot for each dependency change. Use `use_effect!(effect, [])` when the effect should run only after the first render. See [`use_mount!`](use_mount.md) for the inline macro behavior.
