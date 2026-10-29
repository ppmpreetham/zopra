# use_input

## Usage

```rust
#[component]
fn input_example() {
    let input = use_input();

    view! {
        <div class="flex flex-col gap-2 p-4">
            <input state={&input} class="py-2 px-4 bg-zinc-800 text-white rounded-md" placeholder="Type here..." />
            <button onClick={|_, _, cx| println!("User typed: {}", input.read(cx).text())}>
                "Submit"
            </button>
        </div>
    }
}
```

The input handle is refcounted; a closure that uses it gets its own copy automatically.
