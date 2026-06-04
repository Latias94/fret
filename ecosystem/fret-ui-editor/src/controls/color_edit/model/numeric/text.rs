use std::sync::Arc;

use fret_core::Color;

use super::super::{hsv_from_color, sanitize_hue, sanitize_unit};
use super::mode::ColorNumericInputMode;

pub(in crate::controls::color_edit) fn rgb_numeric_text(
    color: Color,
    show_alpha: bool,
) -> Arc<str> {
    let rgb = fret_ui_kit::colors::hex_rgb_from_linear(color);
    let r = ((rgb >> 16) & 0xff) as u8;
    let g = ((rgb >> 8) & 0xff) as u8;
    let b = (rgb & 0xff) as u8;
    if show_alpha {
        Arc::from(format!(
            "RGB {r} {g} {b} | A {}%",
            (sanitize_unit(color.a) * 100.0).round() as u8
        ))
    } else {
        Arc::from(format!("RGB {r} {g} {b}"))
    }
}

pub(in crate::controls::color_edit) fn hsv_numeric_text(color: Color) -> Arc<str> {
    let hsv = hsv_from_color(color);
    Arc::from(format!(
        "HSV {}deg | S {}% | V {}%",
        (sanitize_hue(hsv.hue) * 360.0).round() as u16,
        (sanitize_unit(hsv.saturation) * 100.0).round() as u8,
        (sanitize_unit(hsv.value) * 100.0).round() as u8
    ))
}

pub(in crate::controls::color_edit) fn color_numeric_text(
    color: Color,
    show_alpha: bool,
    mode: ColorNumericInputMode,
) -> Arc<str> {
    match mode {
        ColorNumericInputMode::Rgb => rgb_numeric_text(color, show_alpha),
        ColorNumericInputMode::Hsv => hsv_numeric_text(color),
    }
}
