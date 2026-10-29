//! Tailwind v4 classes compile into the GPUI styles they describe.

use std::{cell::RefCell, rc::Rc};

use gpui_kit::{
    AbsoluteLength, Context, DefiniteLength, Div, InteractiveElement, IntoElement, Length, Overflow, Render,
    StyleRefinement, Styled, TestAppContext, Window, div, px, rems,
};
use zopra::view;

type Styles = Rc<RefCell<Vec<StyleRefinement>>>;

struct Classes(Styles);

fn style(mut el: Div) -> StyleRefinement {
    std::mem::take(el.style())
}

impl Render for Classes {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let wide = window.viewport_size().width >= window.rem_size() * 48.0;
        let styles = [
            view! { <div class="bg-red-500/50 text-theme-primary/25 border-[#ff000080]/50" /> },
            view! { <div class="bg-linear-to-r from-sky-500 from-10% to-indigo-500/50" /> },
            view! { <div class="shadow-md shadow-red-500 ring-2 inset-ring" /> },
            view! { <div class="grid grid-cols-3 grid-rows-2 col-span-2 col-start-2 row-end-3" /> },
            view! { <div class="text-sm/6 leading-tight font-mono" /> },
            view! { <div class="basis-1/2 grow-3 shrink-0 space-x-2 ps-4 me-auto -mt-2 start-px" /> },
            view! { <div class="place-items-center place-content-between place-self-end" /> },
            view! { <div class="overflow-x-visible overflow-y-clip scrollbar-[6px]" /> },
            view! { <div class="rounded rounded-tl-xs rounded-ee-4xl rounded-t-[3px]" /> },
            view! { <div class="p-1 md:p-2 max-md:p-3 dark:hover:bg-white" /> },
            view! { <div class="shadow-none shadow-inner inset-ring-2" /> },
        ]
        .map(style);
        self.0.borrow_mut().extend(styles);
        self.0.borrow_mut().push(StyleRefinement {
            padding: gpui_kit::EdgesRefinement {
                top: Some(DefiniteLength::from(rems(if wide { 0.5 } else { 0.75 }))),
                ..Default::default()
            },
            ..Default::default()
        });
        div()
    }
}

#[gpui_kit::test]
async fn classes_become_gpui_styles(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    let styles = Styles::default();
    let (_view, cx) = cx.add_window_view(|_, _| Classes(styles.clone()));
    cx.run_until_parked();
    let styles = styles.borrow();
    let [colors, gradient, shadows, grid, text, flex, place, overflow, rounded, variants, inset, expected] =
        &styles[..12]
    else {
        unreachable!()
    };

    // `/N` opacity: static colors fold it in, theme colors scale theirs.
    let bg = colors.background.clone().unwrap().color().unwrap().as_solid().unwrap();
    assert!((bg.a - 0.5).abs() < 0.01);
    assert!(colors.text.color.unwrap().a <= 0.25 + 0.01);
    assert!((colors.border_color.unwrap().a - 0.25).abs() < 0.01);

    assert!(gradient.background.clone().unwrap().color().unwrap().as_solid().is_none());

    // shadow-md is two red layers, then the 2px ring and the 1px inset ring.
    let layers = shadows.box_shadow.as_ref().unwrap();
    assert_eq!(layers.len(), 4);
    let red = layers[0].color;
    assert!((red.h < 0.05 || red.h > 0.95) && red.s > 0.5 && red.a == 1.0);
    assert_eq!((layers[2].spread_radius, layers[2].inset), (px(2.0), false));
    assert_eq!((layers[3].spread_radius, layers[3].inset), (px(1.0), true));

    assert!(grid.grid_cols.is_some() && grid.grid_rows.is_some());

    assert_eq!(text.text.line_height, Some(DefiniteLength::from(gpui_kit::relative(1.25))));
    assert!(text.text.font_family.is_some());

    assert_eq!(flex.flex_basis, Some(Length::from(gpui_kit::relative(0.5))));
    assert_eq!(flex.flex_grow, Some(3.0));
    assert_eq!(flex.flex_shrink, Some(0.0));
    assert_eq!(flex.gap.width, Some(DefiniteLength::from(rems(0.5))));
    assert_eq!(flex.padding.left, Some(DefiniteLength::from(rems(1.0))));
    assert_eq!(flex.margin.right, Some(Length::Auto));
    assert_eq!(flex.margin.top, Some(Length::from(rems(-0.5))));
    assert_eq!(flex.inset.left, Some(Length::from(px(1.0))));

    assert!(place.align_items.is_some() && place.justify_content.is_some());
    assert!(place.align_content.is_some() && place.align_self.is_some());

    assert_eq!(overflow.overflow.x, Some(Overflow::Visible));
    assert_eq!(overflow.overflow.y, Some(Overflow::Clip));
    assert_eq!(overflow.scrollbar_width, Some(AbsoluteLength::from(px(6.0))));

    let radii = &rounded.corner_radii;
    assert_eq!(radii.top_left, Some(AbsoluteLength::from(px(3.0))));
    assert_eq!(radii.top_right, Some(AbsoluteLength::from(px(3.0))));
    assert_eq!(radii.bottom_left, Some(AbsoluteLength::from(rems(0.25))));
    assert_eq!(radii.bottom_right, Some(AbsoluteLength::from(rems(2.0))));

    // Breakpoints follow the window; `dark:` follows the theme (light in tests).
    assert_eq!(variants.padding.top, expected.padding.top);

    assert_eq!(inset.box_shadow.as_ref().map(Vec::len), Some(2));
    assert!(inset.box_shadow.as_ref().unwrap().iter().all(|layer| layer.inset));
}
