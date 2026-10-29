# Zopra hooks

These pages document the public hooks. In a `#[component]` body the macro passes `window` and `cx` to hooks, so use the short forms shown here. Outside a component, pass them yourself.

Hooks must be called unconditionally at the top level of a component body. Zopra reports a targeted compile error if it finds a hook in a branch, nested block, or closure; create the hook first and use its handle from the branch or callback.

| Hook | Purpose |
|---|---|
| [`use_state`](use_state.md) | Component state as `Copy` signals; `use_signal` for a single handle |
| [`use_resource`](use_resource.md) | Async data loaded on mount and when its dependency changes |
| [`use_effect!`](use_effect.md) | A side effect after render, again when its dependencies change |
| [`use_event`](use_event.md) | Subscribe to a GPUI entity's events |
| `provide_context` / `use_context` | Share a value (usually a signal) with every component rendered inside this one |
| [`use_input`](use_input.md) | Text input state |
| [`use_window`](use_window.md) | Open another window from a component |
| [`use_webview`](use_webview.md) | Control the optional Wry webview |
