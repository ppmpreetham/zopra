This example shows the declarative `<DataTable>` syntax with nested `<Col>` definitions.

## New Syntax

```rust
use gpui_kit::*;
use zopra::{component, view};

// define the columns here
#[derive(Clone)]
pub struct DemoRow {
    pub id: usize,
    pub method: String,
    pub url: String,
    pub status: u16,
}

#[component]
pub fn col_smoke_test() {
    // the rows that will be displayed in the table
    let rows = vec![
        DemoRow { id: 1, method: "GET".to_string(), url: "/a".to_string(), status: 200 },
        DemoRow { id: 2, method: "POST".to_string(), url: "/b".to_string(), status: 404 },
    ];

    view! {
        <div class="flex flex-col size-full p-4">
            <DataTable rows={rows}>
                <Col r={|row| row.id.to_string()} title="#" width={50.} sortable />
                <Col r={|row| row.method.clone()} title="Method" width={80.} sortable />
                <Col r={|row| row.url.clone()} title="URL" />
                <Col r={|row| row.status.to_string()} title="Status" width={80.} />
            </DataTable>
        </div>
    }
}

```

## Legacy API

The former manual delegate example used the pre-Snap state API and no longer compiles. Use the `<DataTable rows={...}>` and nested `<Col ... />` syntax shown above.
