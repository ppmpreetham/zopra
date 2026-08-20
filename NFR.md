This is how the lib.rs file gotta be:

```rust
fn main() {
    App::production().run(|cx: &mut AppContext| {
        cx.open_window(WindowOptions::default(), |cx| {
            // We create and return the RootView (our main layout container)
            cx.new_view(|cx| RootView::new(cx))
        })
        .unwrap();
    });
}
```

Components can be like this:

```rust
// =========================================================================
// 1. CHILD VIEW A: Sidebar
// =========================================================================
struct Sidebar;

impl Render for Sidebar {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .w_64()
            .h_full()
            .bg(rgb(0x11111b))
            .p_4()
            .text_color(rgb(0xa6adc8))
            .child("📁 Explorer")
    }
}
```

```rust
// =========================================================================
// 2. CHILD VIEW B: Main Content Area
// =========================================================================
struct MainContent;

impl Render for MainContent {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex_1() // Take up remaining horizontal space
            .h_full()
            .bg(rgb(0x1e1e2e)) // Lighter background
            .p_4()
            .text_color(rgb(0xcdd6f4))
            .child("Welcome to your workspace!")
    }

```

```rust
// =========================================================================
// 3. THE PARENT VIEW: Combines them both
// =========================================================================
struct RootView {
    sidebar: View<Sidebar>,
    content: View<MainContent>,
}

impl RootView {
    // A constructor function to initialize child views
    fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            sidebar: cx.new_view(|_| Sidebar),
            content: cx.new_view(|_| MainContent),
        }
    }
```

```rust
impl Render for RootView {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        // Lay them out horizontally side-by-side
        div()
            .flex()
            .flex_row()
            .size_full()
            // We pass the child view handles directly into the tree!
            .child(self.sidebar.clone())
            .child(self.content.clone())
    }
}
```

## Setter and getter functions

This is like the interface from TS

```rust
pub struct Counter {
    count: i32,
}
```

and initializing, setter and getter happens like this:

```rust
impl Counter {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self { count: 0 }
    }

    // GETTER FUNCTION
    pub fn get_count(&self) -> i32 {
        self.count
    }

    // SETTER FUNCTION
    pub fn set_count(&mut self, val: i32, cx: &mut Context<Self>) {
        self.count = val;
        cx.notify();
    }
}
```

`cx.notify()` is the setState function
For one component and for all componenets

or even better use read and update blocks!!
