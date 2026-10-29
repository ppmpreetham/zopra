use gpui_kit::gpui::{self, Rgba};

// gives color based off of the background color (white/black)
pub fn contrast_color(bg: Rgba) -> Rgba {
    let to_linear = |v_f: f32| -> f32 {
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
        gpui::black().into()
    } else {
        gpui::white().into()
    }
}

