use std::sync::Arc;

use fret_core::Color;

use super::ColorEditPopupNumericInputs;

mod hue_wheel;
pub(in crate::controls::color_edit) use hue_wheel::{
    HueWheelDragTarget, HueWheelGeometry, HueWheelTriangle, hsv_with_hue_wheel_position,
    hue_wheel_geometry, hue_wheel_rotated_triangle, hue_wheel_sv_cursor_position,
    hue_wheel_target_from_local_position,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct HsvColor {
    pub(super) hue: f32,
    pub(super) saturation: f32,
    pub(super) value: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ColorNumericInputMode {
    Rgb,
    Hsv,
}

impl ColorNumericInputMode {
    pub(super) fn test_suffix(self) -> &'static str {
        match self {
            Self::Rgb => "rgb",
            Self::Hsv => "hsv",
        }
    }

    pub(super) fn a11y_label(self) -> Arc<str> {
        match self {
            Self::Rgb => Arc::from("RGB color channels"),
            Self::Hsv => Arc::from("HSV color channels"),
        }
    }

    pub(super) fn invalid_message(self) -> Arc<str> {
        match self {
            Self::Rgb => Arc::from("Invalid RGB color values"),
            Self::Hsv => Arc::from("Invalid HSV color values"),
        }
    }
}

const RGB_HSV_NUMERIC_INPUT_MODES: [ColorNumericInputMode; 2] =
    [ColorNumericInputMode::Rgb, ColorNumericInputMode::Hsv];
const RGB_NUMERIC_INPUT_MODES: [ColorNumericInputMode; 1] = [ColorNumericInputMode::Rgb];
const HSV_NUMERIC_INPUT_MODES: [ColorNumericInputMode; 1] = [ColorNumericInputMode::Hsv];

pub(super) fn color_numeric_input_modes(
    numeric_inputs: ColorEditPopupNumericInputs,
) -> &'static [ColorNumericInputMode] {
    match numeric_inputs {
        ColorEditPopupNumericInputs::RgbAndHsv => &RGB_HSV_NUMERIC_INPUT_MODES,
        ColorEditPopupNumericInputs::Rgb => &RGB_NUMERIC_INPUT_MODES,
        ColorEditPopupNumericInputs::Hsv => &HSV_NUMERIC_INPUT_MODES,
        ColorEditPopupNumericInputs::Hidden => &[],
    }
}

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

pub(super) fn rgb_numeric_text(color: Color, show_alpha: bool) -> Arc<str> {
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

pub(super) fn hsv_numeric_text(color: Color) -> Arc<str> {
    let hsv = hsv_from_color(color);
    Arc::from(format!(
        "HSV {}deg | S {}% | V {}%",
        (sanitize_hue(hsv.hue) * 360.0).round() as u16,
        (sanitize_unit(hsv.saturation) * 100.0).round() as u8,
        (sanitize_unit(hsv.value) * 100.0).round() as u8
    ))
}

pub(super) fn color_numeric_text(
    color: Color,
    show_alpha: bool,
    mode: ColorNumericInputMode,
) -> Arc<str> {
    match mode {
        ColorNumericInputMode::Rgb => rgb_numeric_text(color, show_alpha),
        ColorNumericInputMode::Hsv => hsv_numeric_text(color),
    }
}

pub(super) fn parse_color_numeric_input(
    mode: ColorNumericInputMode,
    text: &str,
    show_alpha: bool,
    current: Color,
) -> Option<Color> {
    match mode {
        ColorNumericInputMode::Rgb => parse_rgb_numeric_input(text, show_alpha, current),
        ColorNumericInputMode::Hsv => parse_hsv_numeric_input(text, current),
    }
}

fn parse_rgb_numeric_input(text: &str, show_alpha: bool, current: Color) -> Option<Color> {
    let values = numeric_values_from_text(text)?;
    let expected = if show_alpha { 3..=4 } else { 3..=3 };
    if !expected.contains(&values.len()) {
        return None;
    }

    let r = parse_u8_channel(values[0])?;
    let g = parse_u8_channel(values[1])?;
    let b = parse_u8_channel(values[2])?;
    let alpha = if show_alpha && values.len() == 4 {
        parse_percent_unit(values[3])?
    } else {
        current.a
    };

    Some(color_from_rgb_preserving_alpha(
        ((r as u32) << 16) | ((g as u32) << 8) | b as u32,
        alpha,
    ))
}

fn parse_hsv_numeric_input(text: &str, current: Color) -> Option<Color> {
    let values = numeric_values_from_text(text)?;
    if values.len() != 3 {
        return None;
    }

    let hsv = HsvColor {
        hue: parse_hue_degrees(values[0])?,
        saturation: parse_percent_unit(values[1])?,
        value: parse_percent_unit(values[2])?,
    };
    Some(hsv_to_color_preserving_alpha(hsv, current.a))
}

fn numeric_values_from_text(text: &str) -> Option<Vec<f32>> {
    let normalized = text
        .chars()
        .map(|ch| {
            if ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '+' {
                ch
            } else {
                ' '
            }
        })
        .collect::<String>();
    let values = normalized
        .split_whitespace()
        .map(str::parse::<f32>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if values.is_empty() {
        None
    } else {
        Some(values)
    }
}

fn parse_u8_channel(value: f32) -> Option<u8> {
    if value.is_finite() && (0.0..=255.0).contains(&value) {
        Some(value.round() as u8)
    } else {
        None
    }
}

fn parse_percent_unit(value: f32) -> Option<f32> {
    if value.is_finite() && (0.0..=100.0).contains(&value) {
        Some((value / 100.0).clamp(0.0, 1.0))
    } else {
        None
    }
}

fn parse_hue_degrees(value: f32) -> Option<f32> {
    if value.is_finite() && (0.0..=360.0).contains(&value) {
        Some(sanitize_hue(value / 360.0))
    } else {
        None
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

pub(super) fn hsv_from_color(color: Color) -> HsvColor {
    rgb_to_hsv(fret_ui_kit::colors::hex_rgb_from_linear(color))
}

pub(super) fn rgb_to_hsv(rgb: u32) -> HsvColor {
    let r = ((rgb >> 16) & 0xff) as f32 / 255.0;
    let g = ((rgb >> 8) & 0xff) as f32 / 255.0;
    let b = (rgb & 0xff) as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let chroma = max - min;
    let hue = if chroma <= f32::EPSILON {
        0.0
    } else if (max - r).abs() <= f32::EPSILON {
        ((g - b) / chroma).rem_euclid(6.0) / 6.0
    } else if (max - g).abs() <= f32::EPSILON {
        (((b - r) / chroma) + 2.0) / 6.0
    } else {
        (((r - g) / chroma) + 4.0) / 6.0
    };
    let saturation = if max <= f32::EPSILON {
        0.0
    } else {
        chroma / max
    };

    HsvColor {
        hue: sanitize_hue(hue),
        saturation: saturation.clamp(0.0, 1.0),
        value: max.clamp(0.0, 1.0),
    }
}

pub(super) fn hsv_to_rgb(hsv: HsvColor) -> u32 {
    let hue = sanitize_hue(hsv.hue);
    let saturation = sanitize_unit(hsv.saturation);
    let value = sanitize_unit(hsv.value);

    if saturation <= f32::EPSILON {
        let v = unit_to_u8(value);
        return ((v as u32) << 16) | ((v as u32) << 8) | v as u32;
    }

    let scaled_hue = hue * 6.0;
    let sector = scaled_hue.floor() as i32;
    let fraction = scaled_hue - sector as f32;
    let p = value * (1.0 - saturation);
    let q = value * (1.0 - saturation * fraction);
    let t = value * (1.0 - saturation * (1.0 - fraction));
    let (r, g, b) = match sector {
        0 => (value, t, p),
        1 => (q, value, p),
        2 => (p, value, t),
        3 => (p, q, value),
        4 => (t, p, value),
        _ => (value, p, q),
    };

    ((unit_to_u8(r) as u32) << 16) | ((unit_to_u8(g) as u32) << 8) | unit_to_u8(b) as u32
}

pub(super) fn hsv_to_color_preserving_alpha(hsv: HsvColor, alpha: f32) -> Color {
    color_from_rgb_preserving_alpha(hsv_to_rgb(hsv), alpha)
}

pub(super) fn color_from_rgb_preserving_alpha(rgb: u32, alpha: f32) -> Color {
    let mut out = Color::from_srgb_hex_rgb(rgb);
    out.a = alpha.clamp(0.0, 1.0);
    out
}

pub(super) fn hsv_with_sv_from_local_position(
    current: HsvColor,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> HsvColor {
    HsvColor {
        hue: current.hue,
        saturation: unit_from_local_x(x, width),
        value: 1.0 - unit_from_local_y(y, height),
    }
}

pub(super) fn hue_from_local_y(y: f32, height: f32) -> f32 {
    unit_from_local_y(y, height)
}

pub(super) fn unit_from_local_x(x: f32, width: f32) -> f32 {
    if !width.is_finite() || width <= f32::EPSILON {
        return 0.0;
    }
    sanitize_unit(x / width)
}

pub(super) fn unit_from_local_y(y: f32, height: f32) -> f32 {
    if !height.is_finite() || height <= f32::EPSILON {
        return 0.0;
    }
    sanitize_unit(y / height)
}

pub(super) fn unit_from_step(index: usize, steps: usize) -> f32 {
    if steps <= 1 {
        return 0.0;
    }
    (index as f32 / (steps - 1) as f32).clamp(0.0, 1.0)
}

pub(super) fn sanitize_hue(hue: f32) -> f32 {
    if !hue.is_finite() {
        return 0.0;
    }
    hue.rem_euclid(1.0)
}

pub(super) fn sanitize_unit(value: f32) -> f32 {
    if !value.is_finite() {
        return 0.0;
    }
    value.clamp(0.0, 1.0)
}

fn unit_to_u8(value: f32) -> u8 {
    (sanitize_unit(value) * 255.0).round() as u8
}

pub(super) fn sv_picker_a11y_text(hsv: HsvColor) -> Arc<str> {
    Arc::from(format!(
        "S {}%, V {}%",
        (sanitize_unit(hsv.saturation) * 100.0).round() as u8,
        (sanitize_unit(hsv.value) * 100.0).round() as u8
    ))
}

pub(super) fn hue_percent_text(hue: f32) -> Arc<str> {
    Arc::from(format!("{}%", (sanitize_hue(hue) * 100.0).round() as u8))
}
