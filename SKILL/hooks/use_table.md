# `use_table` and `use_table_with`

`use_table(delegate)` creates table state once for the component and returns the same entity on later renders. The component macro wraps the initial delegate expression in a lazy initializer. The delegate must implement GPUI's `TableDelegate` and be `'static`.

```rust
let table = use_table(MyTableDelegate::new(rows));
```

Use `use_table_with(|| MyTableDelegate::new(), |state| configure(state))` to customize the initial `TableState`. The builder closure runs only when the table state is first created. Keep this hook for the imperative GPUI table API; for declarative columns and row data, prefer `<DataTable rows={rows}><Col ... /></DataTable>`.
