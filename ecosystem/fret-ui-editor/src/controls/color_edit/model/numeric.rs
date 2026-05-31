use std::sync::Arc;

use fret_core::Color;

use super::{
    HsvColor, color_from_rgb_preserving_alpha, hsv_from_color, hsv_to_color_preserving_alpha,
    sanitize_hue, sanitize_unit,
};
use crate::controls::color_edit::ColorEditPopupNumericInputs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::controls::color_edit) enum ColorNumericInputMode {
    Rgb,
    Hsv,
}

impl ColorNumericInputMode {
    pub(in crate::controls::color_edit) fn test_suffix(self) -> &'static str {
        match self {
            Self::Rgb => "rgb",
            Self::Hsv => "hsv",
        }
    }

    pub(in crate::controls::color_edit) fn a11y_label(self) -> Arc<str> {
        match self {
            Self::Rgb => Arc::from("RGB color channels"),
            Self::Hsv => Arc::from("HSV color channels"),
        }
    }

    pub(in crate::controls::color_edit) fn invalid_message(self) -> Arc<str> {
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

pub(in crate::controls::color_edit) fn color_numeric_input_modes(
    numeric_inputs: ColorEditPopupNumericInputs,
) -> &'static [ColorNumericInputMode] {
    match numeric_inputs {
        ColorEditPopupNumericInputs::RgbAndHsv => &RGB_HSV_NUMERIC_INPUT_MODES,
        ColorEditPopupNumericInputs::Rgb => &RGB_NUMERIC_INPUT_MODES,
        ColorEditPopupNumericInputs::Hsv => &HSV_NUMERIC_INPUT_MODES,
        ColorEditPopupNumericInputs::Hidden => &[],
    }
}

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

pub(in crate::controls::color_edit) fn parse_color_numeric_input(
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
