use std::collections::BTreeSet;
use std::sync::Arc;

use fret_core::Color;

use super::drag_drop::apply_color_drop_payload;
use super::model::{
    ColorNumericInputMode, HsvColor, HueWheelDragTarget, color_from_rgb_preserving_alpha,
    color_numeric_input_modes, format_hex, hsv_from_color, hsv_numeric_text,
    hsv_to_color_preserving_alpha, hsv_to_rgb, hsv_with_hue_wheel_position,
    hsv_with_sv_from_local_position, hue_from_local_y, hue_wheel_geometry,
    hue_wheel_rotated_triangle, hue_wheel_sv_cursor_position, hue_wheel_target_from_local_position,
    parse_color_numeric_input, parse_hex, rgb_numeric_text, rgb_to_hsv,
};
use super::popup::copy::{ColorEditCopyFormat, color_copy_entries};
use super::popup::picker::alpha::{alpha_from_local_x, alpha_from_local_y, alpha_percent_text};
use super::popup::preview::{
    SIDE_PREVIEW_SWATCH_HEIGHT, SIDE_PREVIEW_SWATCH_WIDTH, checkerboard_cell_color,
    opaque_preview_color, preview_color_for_alpha_visibility, restore_reference_color,
};
use super::popup::tooltip::color_tooltip_lines;
use super::*;

mod affordances;
mod drag_drop;
mod numeric;
mod palette;
mod picker;
mod popup_policy;

fn assert_hsv_close(actual: HsvColor, hue: f32, saturation: f32, value: f32) {
    assert!(
        (actual.hue - hue).abs() < 0.002,
        "hue mismatch: actual {:?}, expected {hue}",
        actual
    );
    assert!(
        (actual.saturation - saturation).abs() < 0.002,
        "saturation mismatch: actual {:?}, expected {saturation}",
        actual
    );
    assert!(
        (actual.value - value).abs() < 0.002,
        "value mismatch: actual {:?}, expected {value}",
        actual
    );
}
