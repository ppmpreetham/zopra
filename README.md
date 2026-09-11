# Zopra

A reactive framework for GPUI

# Example

```rust
#[component]
fn profile(username: &'static str) {
    let (likes, set_likes) = use_state(0);

    use_effect!(move || {
        println!("{username} now has {} likes!", likes());
    }, [likes]);

    view! {
        <div
            class="flex items-center gap-4 p-4 bg-zinc-900 rounded-xl"
            on_click={move || set_likes(likes() + 1)}
        >
            <span class="text-white font-bold">{username}</span>
            <span class="text-zinc-400">"Likes: " {likes()}</span>
        </div>
    }
}

#[component]
fn app() {
    view! {
        <div class="flex flex-col gap-2 p-6">
            <profile username="Alice" />
            <profile username="Bob" />
        </div>
    }
}

```
