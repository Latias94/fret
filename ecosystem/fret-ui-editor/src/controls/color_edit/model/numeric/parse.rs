use fret_core::Color;

use super::super::{
    HsvColor, color_from_rgb_preserving_alpha, hsv_to_color_preserving_alpha, sanitize_hue,
};
use super::mode::ColorNumericInputMode;

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
