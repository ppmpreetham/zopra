# Zopra

<p align="center">
  <img src="readme/Zopra.svg" width="300px" alt="Zopra Logo"/>
</p>

<h1 align="center">
  Zopra
</h1>

<p align="center">
  A reactive framework for GPUI
</p>

## Install

```bash
cargo generate ppmpreetham/create-zopra-app
```

## Get Started

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

## License

The source is [MIT licensed](LICENSE) and can be built and run for free, for both personal and commercial use.
