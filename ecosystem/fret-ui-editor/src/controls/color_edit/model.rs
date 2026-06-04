use std::sync::Arc;

use fret_core::Color;

mod hsv;
mod hue_wheel;
mod numeric;
pub(super) use hsv::{
    HsvColor, color_from_rgb_preserving_alpha, hsv_from_color, hsv_to_color_preserving_alpha,
    hsv_with_sv_from_local_position, hue_from_local_y, hue_percent_text, sanitize_hue,
    sanitize_unit, sv_picker_a11y_text, unit_from_step,
};
#[cfg(test)]
pub(super) use hsv::{hsv_to_rgb, rgb_to_hsv};
pub(in crate::controls::color_edit) use hue_wheel::{
    HueWheelDragTarget, HueWheelGeometry, HueWheelTriangle, hsv_with_hue_wheel_position,
    hue_wheel_geometry, hue_wheel_rotated_triangle, hue_wheel_sv_cursor_position,
    hue_wheel_target_from_local_position,
};
pub(super) use numeric::{
    ColorNumericInputMode, color_numeric_input_modes, color_numeric_text, hsv_numeric_text,
    parse_color_numeric_input, rgb_numeric_text,
};

pub(super) fn format_hex(color: Color, show_alpha: bool) -> Arc<str> {
    let rgb = fret_ui_kit::colors::hex_rgb_from_linear(color);
    let r = ((rgb >> 16) & 0xff) as u8;
    let g = ((rgb >> 8) & 0xff) as u8;
    let b = (rgb & 0xff) as u8;
    let a = (color.a.clamp(0.0, 1.0) * 255.0).round() as u8;
    if show_alpha {
        Arc::from(format!("#{r:02X}{g:02X}{b:02X}{a:02X}"))
    } else {
        Arc::from(format!("#{r:02X}{g:02X}{b:02X}"))
    }
}

pub(super) fn parse_hex(text: &str, show_alpha: bool, current: Color) -> Option<Color> {
    let s = text.trim().trim_start_matches('#');
    let s = s.trim();

    if s.len() != 6 && !(show_alpha && s.len() == 8) {
        return None;
    }

    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    let a = if s.len() == 8 {
        u8::from_str_radix(&s[6..8], 16).ok()?
    } else {
        (current.a.clamp(0.0, 1.0) * 255.0).round() as u8
    };

    let mut out =
        fret_ui_kit::colors::linear_from_hex_rgb(((r as u32) << 16) | ((g as u32) << 8) | b as u32);
    out.a = a as f32 / 255.0;
    Some(out)
}
