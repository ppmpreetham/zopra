// regression tests for #[component]

use gpui_kit::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, TestAppContext, Window, div,
};
use zopra::{component, hooks::use_state, signals};

#[component]
fn match_guard_component() {
    let (count, _set_count) = use_state(0);

    let label = match Some("big") {
        Some(name) if count() > 10 => format!("{name}: many"),
        Some(name) => format!("{name}: few"),
        None => String::new(),
    };

    div().child(label)
}


#[component]
fn shadowed_signal_component() {
    let (count, _set_count) = use_state(0);

    let label = {
        let _outer_read = count();
        let count = || 42i32;
        format!("inner: {}", count())
    };

    div().child(label).child(count().to_string())
}

mod third_party {
    pub fn use_state(value: i32) -> i32 {
        value * 2
    }

    pub fn use_callback(f: impl FnOnce()) {
        f();
    }
}

#[component]
fn collision_component() {
    let doubled = third_party::use_state(21);
    assert_eq!(doubled, 42);

    third_party::use_callback(|| {
        let _ = 42i32;
    });

    div()
}


#[component]
fn qualified_hook_component() {
    let (count, set_count) = zopra::hooks::use_state(7);
    set_count(count() + 1);
    div().child(count().to_string())
}


#[component]
fn macro_signal_component() {
    macro_rules! counter {
        ($get:ident, $set:ident) => {
            let ($get, $set) = use_state(0, window, cx);
        };
    }
    counter!(count, set_count);
    signals!(count, set_count);

    let label = format!("Count: {}", count());
    set_count(5);
    let label = format!("{label} -> {}", count());

    div().child(label)
}


#[component]
fn greet(name: String, count: u32) {
    div().child(format!("Hello, {name} x{count}!"))
}

use gpui_rsx::rsx;

#[component]
fn greet_click(name: String) {
    let (clicked, set_clicked) = use_state(0);
    let label = clicked().to_string();

    div()
        .flex()
        .flex_col()
        .child(format!("Hello, {name}!"))
        .child(
            div()
                .id("greet-btn")
                .cursor_pointer()
                .on_click(move |_, _, cx| set_clicked(clicked() + 1))
                .child(label),
        )
}

#[gpui_kit::test]
async fn props_builder_renders_immediately(cx: &mut TestAppContext) {
    let window = cx.add_empty_window();
    window.update(|window, cx| {
        let el = GreetProps::new().name("builder".into()).count(3).render(window, cx);
        let _: AnyElement = el.into_any_element();
    });

    let window2 = cx.add_empty_window();
    window2.update(|window, cx| {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = GreetProps::new().count(1).render(window, cx);
        }));
        assert!(result.is_err(), "missing prop `name` must panic immediately");
    });
}

#[component]
fn rsx_host_component() {
    div().child(rsx! {
        <GreetClick name={"zopra".to_string()} />
    })
}

#[component]
fn rsx_direct_component() {
    rsx! {
        <div>
            <GreetClick name={"zopra".to_string()} />
        </div>
    }
}

#[component]
fn rsx_nested_component() {
    rsx! {
        <div class="flex flex-col">
            <GreetClick name={"a".to_string()} />
            <GreetClick name={"b".to_string()} />
        </div>
    }
}

struct Case(&'static str);

impl Render for Case {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let element: AnyElement = match self.0 {
            "match_guard" => match_guard_component(window, cx).into_any_element(),
            "shadow" => shadowed_signal_component(window, cx).into_any_element(),
            "collision" => collision_component(window, cx).into_any_element(),
            "qualified" => qualified_hook_component(window, cx).into_any_element(),
            "macro_signal" => macro_signal_component(window, cx).into_any_element(),
            "rsx" => rsx_host_component(window, cx).into_any_element(),
            "rsx_direct" => rsx_direct_component(window, cx).into_any_element(),
            "rsx_nested" => rsx_nested_component(window, cx).into_any_element(),
            other => unreachable!("unknown case: {other}"),
        };
        element
    }
}

#[gpui_kit::test]
async fn components_render(cx: &mut TestAppContext) {
    for case in [
        "match_guard",
        "shadow",
        "collision",
        "qualified",
        "macro_signal",
        "rsx",
        "rsx_direct",
        "rsx_nested",
    ] {
        let (_view, cx) = cx.add_window_view(|_, _| Case(case));
        cx.run_until_parked();
    }
}
