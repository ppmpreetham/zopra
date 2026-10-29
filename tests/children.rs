//! Children, typed children, callback props and element props.

use std::{cell::RefCell, rc::Rc};

use gpui_kit::{
    AnyElement, App, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, StatefulInteractiveElement,
    TestAppContext, Window, div, px,
};
use zopra::hooks::{Signal, provide_context, try_use_context, use_signal};
use zopra::ui::{Resizable, ResizablePanel, Tab, TabBar};
use zopra::{Children, component, view};

type Log<T> = Rc<RefCell<Vec<T>>>;

// #region children render inside the parent
#[component]
fn theme(name: SharedString, children: Children) {
    provide_context(name);
    view! { <div>{children}</div> }
}

#[component]
fn themed(log: Log<Option<SharedString>>) {
    log.borrow_mut().push(try_use_context::<SharedString>());
    div()
}

#[component]
fn themed_tree(log: Log<Option<SharedString>>) {
    view! {
        <div>
            <Theme name="dark">
                <Themed log={log.clone()} />
                "text children work too"
            </Theme>
            <Themed log={log.clone()} />
        </div>
    }
}

struct ThemeView(Log<Option<SharedString>>);

impl Render for ThemeView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        themed_tree(self.0.clone(), window, cx)
    }
}

#[gpui_kit::test]
async fn children_see_the_context_of_the_component_that_places_them(cx: &mut TestAppContext) {
    let log = Log::default();
    let (_view, cx) = cx.add_window_view(|_, _| ThemeView(log.clone()));
    cx.run_until_parked();
    assert_eq!(*log.borrow(), [Some("dark".into()), None]);
}
// #endregion

// #region typed children, callbacks, element props, context names
#[component]
fn toolbar<F: Fn(&usize, &mut Window, &mut App) + 'static>(
    on_pick: Option<F>,
    badge: Option<AnyElement>,
    picked: Signal<usize>,
) {
    view! {
        <div>
            <TabBar selected_index={picked()} on_click={|index, _, app_cx| picked(*index)}>
                <Tab label="One" />
                <Tab label="Two" suffix={view! { <div>"2"</div> }} />
            </TabBar>
            <Resizable vertical>
                <ResizablePanel size={px(100.)}>"top"</ResizablePanel>
                <ResizablePanel>{...badge}</ResizablePanel>
            </Resizable>
            {...on_pick.map(|_| div().child("has callback"))}
        </div>
    }
}

struct ToolbarView {
    clicks: Option<Signal<usize>>,
}

impl Render for ToolbarView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let picked = use_signal(|| 0usize, window, cx);
        self.clicks = Some(picked);
        div()
            // an optional callback left unset, and one given
            .child(toolbar(None::<fn(&usize, &mut Window, &mut App)>, None, picked, window, cx))
            .child(view! {
                <Toolbar
                    picked={picked}
                    on_pick={move |index: &usize, _: &mut Window, app: &mut App| picked.set(*index, app)}
                    badge={view! { <div>"badge"</div> }}
                />
            })
            .child(view! { <Toolbar picked={picked} /> })
    }
}

#[gpui_kit::test]
async fn wrappers_callbacks_and_element_props_render(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    let (view, cx) = cx.add_window_view(|_, _| ToolbarView { clicks: None });
    cx.run_until_parked();
    let picked = cx.cx.read(|cx| view.read(cx).clicks).unwrap();
    assert_eq!(cx.cx.read(|cx| picked.get(cx)), 0);
}

#[component]
fn counter_button(count: Signal<usize>) {
    // the context parameter can have any name, or be `_` (zopra names it)
    view! {
        <div>
            <div id="a" on_click={|_: &ClickEvent, _, app_cx| count(count() + 1)} />
            <div id="b" on_click={|_, _, _| count(|n| *n += 1)} />
            <div id="c" on_click={|| count(0)} />
        </div>
    }
}

struct CounterView;

impl Render for CounterView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count = use_signal(|| 0usize, window, cx);
        counter_button(count, window, cx)
    }
}

#[gpui_kit::test]
async fn any_context_parameter_name_works(cx: &mut TestAppContext) {
    let (_view, cx) = cx.add_window_view(|_, _| CounterView);
    cx.run_until_parked();
}
// #endregion
