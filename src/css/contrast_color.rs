use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds, Context, SharedString, Window,
    WindowBounds, WindowOptions,
};

// gives color based off of the background color (white/black)
pub fn contrast_color(bg: rgb) -> rgb {
pub fn contrast_color(bg: rgb) -> rgb {
    let to_linear = |v: u8| -> f32 {
        let v_f = f32::from(v) / 255.0;
        if v_f <= 0.03928 {
            v_f / 12.92
        } else {
            ((v_f + 0.055) / 1.055).powf(2.4)
        }
    };

    let luminance = 0.2126 * to_linear(bg.r)
        + 0.7152 * to_linear(bg.g)
        + 0.0722 * to_linear(bg.b);

    if luminance > 0.1791 {
        gpui::black()
    } else {
        gpui::white()
    }
}
