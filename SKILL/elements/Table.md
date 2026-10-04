# Table

- This is only for static tables (which don't change, and are rendered once), and don't have virtualization. Never ever use it unless this is the use case. For actual Tables, use `DataTable.md`.

```rust
<table border_0 rounded_none>
    <thead>
        <tr bg={cx.theme().table_even}>
            <th w={px(80.0)}>"ID"</th>
            <th>"Name"</th>
            <th text_right>"Amount"</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td>"INV001"</td>
            <td px_4>"Paid"</td>
            <td text_right>"$250.00"</td>
        </tr>
    </tbody>
    <caption text_center>"A list of recent invoices."</caption>
</table>
```

### Here is the exact internal mapping I just added:

- `<table>` ➔ `Table::new()`
- `<thead>` / `<table_header>` ➔ `TableHeader::new()`
- `<tbody>` / `<table_body>` ➔ `TableBody::new()`
- `<tfoot>` / `<table_footer>` ➔ `TableFooter::new()`
- `<tr>` / `<table_row>` ➔ `TableRow::new()`
- `<th>` / `<table_head>` ➔ `TableHead::new()`
- `<td>` / `<table_cell>` ➔ `TableCell::new()`
- `<caption>` / `<table_caption>` ➔ `TableCaption::new()`

Because all of these sub-components implement the `Styled` trait, all of the standard tailwind classes and builder methods (like `.w(px(80.))`, `.text_right()`, and `.bg()`) map flawlessly to their tag attributes without you needing to do anything else.

`gpui-rsx` is fully compiled, and all unit tests and `syzygy` builds passed perfectly. Your `view!` macros are looking more like clean HTML every single commit!
