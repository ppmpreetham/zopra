# Zopra hooks

These pages document the public hooks and hook-like macros available in Zopra. In a `#[component]` body, the component macro injects `window` and `cx` into supported hook calls, so use the shorter syntax shown here. Calls made outside a component must supply the framework arguments required by the Rust function.

Hooks must be called unconditionally at the top level of a component body. Zopra reports a targeted compile error if it finds a hook in a conditional branch, nested block, or closure; create the hook first and use its handle from the branch or callback.

| Hook | Purpose |
|---|---|
| [`use_state`](use_state.md) | Small, comparable state with immutable render snapshots |
| [`use_model`](use_model.md) | Large or frequently updated entity-backed state |
| [`use_resource`](use_resource.md) | Async data loaded on mount and dependency changes |
| [`use_effect!`](use_effect.md) | Run a deferred effect when dependencies change |
| [`use_mount!`](use_mount.md) | Inline mount macro; currently evaluated on every render |
| [`use_event`](use_event.md) | Subscribe to GPUI entity events |
| [`use_async`](use_async.md) | Spawn async work tied to the component lifetime |
| [`use_callback`](use_callback.md) | Create a callback that updates a target entity |
| [`use_table`](use_table.md) | Keep a table delegate and state across component renders |
| [`use_window`](use_window.md) | Open another GPUI window from a component |
| [`use_webview`](use_webview.md) | Control the optional Wry webview |

`use_table_with` is documented on the [`use_table`](use_table.md) page. `use_mount_cx!` is documented with [`use_mount!`](use_mount.md).
