use std::collections::BTreeSet;

use super::model::{
    ColorNumericInputMode, HsvColor, color_from_rgb_preserving_alpha, color_numeric_input_modes,
    hsv_from_color, hsv_numeric_text, hsv_to_color_preserving_alpha, hsv_to_rgb,
    hsv_with_sv_from_local_position, hue_from_local_x, parse_color_numeric_input, rgb_numeric_text,
    rgb_to_hsv,
};
use super::popup::picker::{alpha_from_local_x, alpha_percent_text};
use super::popup::preview::checkerboard_cell_color;
use super::*;

#[test]
fn color_presets_are_unique_and_hex_formattable() {
    let mut seen = BTreeSet::new();
    for (name, rgb) in COLOR_PRESETS {
        assert!(seen.insert(rgb), "duplicate preset rgb for {name}");
        let formatted = format_hex(Color::from_srgb_hex_rgb(rgb), false);
        assert_eq!(formatted.len(), 7);
        assert!(formatted.starts_with('#'));
    }
}

#[test]
fn popup_options_default_to_imgui_like_hue_bar_surface() {
    let options = ColorEditPopupOptions::default();

    assert_eq!(options.picker, ColorEditPopupPicker::HsvHueBar);
    assert_eq!(
        options.numeric_inputs,
        ColorEditPopupNumericInputs::RgbAndHsv
    );
    assert!(options.presets);
    assert!(options.alpha_bar);
    assert!(options.has_visible_content(false));
    assert!(options.has_visible_content(true));
    assert!(!options.shows_alpha_bar(false));
    assert!(options.shows_alpha_bar(true));
}

#[test]
fn popup_options_can_hide_every_popup_affordance() {
    let options = ColorEditPopupOptions {
        picker: ColorEditPopupPicker::Hidden,
        numeric_inputs: ColorEditPopupNumericInputs::Hidden,
        presets: false,
        alpha_bar: false,
    };

    assert!(!options.has_visible_content(false));
    assert!(!options.has_visible_content(true));
}

#[test]
fn popup_numeric_input_modes_are_explicit_and_ordered() {
    assert_eq!(
        color_numeric_input_modes(ColorEditPopupNumericInputs::RgbAndHsv),
        &[ColorNumericInputMode::Rgb, ColorNumericInputMode::Hsv]
    );
    assert_eq!(
        color_numeric_input_modes(ColorEditPopupNumericInputs::Rgb),
        &[ColorNumericInputMode::Rgb]
    );
    assert_eq!(
        color_numeric_input_modes(ColorEditPopupNumericInputs::Hsv),
        &[ColorNumericInputMode::Hsv]
    );
    assert!(color_numeric_input_modes(ColorEditPopupNumericInputs::Hidden).is_empty());
}

#[test]
fn rgb_hex_parse_preserves_alpha_when_alpha_is_not_explicit() {
    let mut current = Color::from_srgb_hex_rgb(0x11_22_33);
    current.a = 0.375;

    let parsed = parse_hex("#EF4444", false, current).expect("rgb hex should parse");

    assert_eq!(parsed.to_srgb_hex_rgb(), 0xef_44_44);
    assert!((parsed.a - current.a).abs() < 0.002);
}

#[test]
fn rgba_hex_parse_is_only_available_when_alpha_is_visible() {
    let current = Color::from_srgb_hex_rgb(0x11_22_33);

    assert!(parse_hex("#EF444480", false, current).is_none());

    let parsed =
        parse_hex("#EF444480", true, current).expect("rgba hex should parse when alpha shows");

    assert_eq!(parsed.to_srgb_hex_rgb(), 0xef_44_44);
    assert!((parsed.a - (0x80 as f32 / 255.0)).abs() < f32::EPSILON);
}

#[test]
fn rgb_presets_preserve_current_alpha() {
    let color = color_from_rgb_preserving_alpha(0x3b_82_f6, 0.25);

    assert_eq!(color.to_srgb_hex_rgb(), 0x3b_82_f6);
    assert_eq!(format_hex(color, true).as_ref(), "#3B82F640");
}

#[test]
fn numeric_readout_formats_rgb_hsv_and_optional_alpha() {
    let mut color = Color::from_srgb_hex_rgb(0x33_66_99);
    color.a = 0.5;

    assert_eq!(rgb_numeric_text(color, false).as_ref(), "RGB 51 102 153");
    assert_eq!(
        rgb_numeric_text(color, true).as_ref(),
        "RGB 51 102 153 | A 50%"
    );
    assert_eq!(
        hsv_numeric_text(color).as_ref(),
        "HSV 210deg | S 67% | V 60%"
    );
}

#[test]
fn rgb_numeric_input_parses_channels_and_optional_alpha_percent() {
    let mut current = Color::from_srgb_hex_rgb(0x11_22_33);
    current.a = 0.375;

    let rgb_only =
        parse_color_numeric_input(ColorNumericInputMode::Rgb, "RGB 255 128 0", true, current)
            .expect("rgb values should parse");
    assert_eq!(rgb_only.to_srgb_hex_rgb(), 0xff_80_00);
    assert!((rgb_only.a - current.a).abs() < f32::EPSILON);

    let rgba = parse_color_numeric_input(
        ColorNumericInputMode::Rgb,
        "RGB 255 128 0 | A 25%",
        true,
        current,
    )
    .expect("rgba values should parse");
    assert_eq!(rgba.to_srgb_hex_rgb(), 0xff_80_00);
    assert!((rgba.a - 0.25).abs() < f32::EPSILON);
}

#[test]
fn hsv_numeric_input_parses_degrees_and_percentages_preserving_alpha() {
    let mut current = Color::from_srgb_hex_rgb(0x11_22_33);
    current.a = 0.625;

    let parsed = parse_color_numeric_input(
        ColorNumericInputMode::Hsv,
        "HSV 120deg | S 50% | V 25%",
        false,
        current,
    )
    .expect("hsv values should parse");

    let hsv = hsv_from_color(parsed);
    assert_hsv_close(hsv, 120.0 / 360.0, 0.5, 0.25);
    assert!((parsed.a - current.a).abs() < f32::EPSILON);
}

#[test]
fn numeric_input_rejects_out_of_range_or_incomplete_values() {
    let current = Color::from_srgb_hex_rgb(0x11_22_33);

    assert!(
        parse_color_numeric_input(ColorNumericInputMode::Rgb, "RGB 256 0 0", false, current)
            .is_none()
    );
    assert!(
        parse_color_numeric_input(ColorNumericInputMode::Rgb, "RGB 255 0", false, current)
            .is_none()
    );
    assert!(
        parse_color_numeric_input(ColorNumericInputMode::Hsv, "HSV 361 50 50", false, current)
            .is_none()
    );
    assert!(
        parse_color_numeric_input(ColorNumericInputMode::Hsv, "HSV 120 101 50", false, current)
            .is_none()
    );
}

#[test]
fn hsv_conversion_matches_primary_colors() {
    let red = rgb_to_hsv(0xff_00_00);
    assert_hsv_close(red, 0.0, 1.0, 1.0);

    let green = rgb_to_hsv(0x00_ff_00);
    assert_hsv_close(green, 1.0 / 3.0, 1.0, 1.0);

    let blue = rgb_to_hsv(0x00_00_ff);
    assert_hsv_close(blue, 2.0 / 3.0, 1.0, 1.0);

    assert_eq!(
        hsv_to_rgb(HsvColor {
            hue: 0.0,
            saturation: 1.0,
            value: 1.0,
        }),
        0xff_00_00
    );
    assert_eq!(
        hsv_to_rgb(HsvColor {
            hue: 1.0 / 3.0,
            saturation: 1.0,
            value: 1.0,
        }),
        0x00_ff_00
    );
    assert_eq!(
        hsv_to_rgb(HsvColor {
            hue: 2.0 / 3.0,
            saturation: 1.0,
            value: 1.0,
        }),
        0x00_00_ff
    );
}

#[test]
fn hsv_conversion_handles_grayscale_without_unstable_hue() {
    assert_hsv_close(rgb_to_hsv(0x00_00_00), 0.0, 0.0, 0.0);
    assert_hsv_close(rgb_to_hsv(0x80_80_80), 0.0, 0.0, 128.0 / 255.0);
    assert_hsv_close(rgb_to_hsv(0xff_ff_ff), 0.0, 0.0, 1.0);

    assert_eq!(
        hsv_to_rgb(HsvColor {
            hue: 0.42,
            saturation: 0.0,
            value: 128.0 / 255.0,
        }),
        0x80_80_80
    );
}

#[test]
fn hsv_conversion_roundtrips_color_presets() {
    for (_name, rgb) in COLOR_PRESETS {
        let hsv = rgb_to_hsv(rgb);
        assert_eq!(hsv_to_rgb(hsv), rgb);
    }
}

#[test]
fn sv_picker_position_preserves_hue_and_clamps_sv() {
    let current = HsvColor {
        hue: 0.25,
        saturation: 0.4,
        value: 0.6,
    };

    let inside = hsv_with_sv_from_local_position(current, 25.0, 75.0, 100.0, 100.0);
    assert_hsv_close(inside, 0.25, 0.25, 0.25);

    let clamped = hsv_with_sv_from_local_position(current, 120.0, -10.0, 100.0, 100.0);
    assert_hsv_close(clamped, 0.25, 1.0, 1.0);
}

#[test]
fn hue_bar_position_maps_local_x_to_clamped_hue() {
    assert_eq!(hue_from_local_x(-10.0, 100.0), 0.0);
    assert!((hue_from_local_x(37.5, 100.0) - 0.375).abs() < f32::EPSILON);
    assert_eq!(hue_from_local_x(120.0, 100.0), 1.0);
    assert_eq!(hue_from_local_x(10.0, 0.0), 0.0);
}

#[test]
fn hsv_color_edits_preserve_current_alpha() {
    let color = hsv_to_color_preserving_alpha(
        HsvColor {
            hue: 1.0 / 3.0,
            saturation: 1.0,
            value: 1.0,
        },
        0.25,
    );

    assert_eq!(color.to_srgb_hex_rgb(), 0x00_ff_00);
    assert_eq!(format_hex(color, true).as_ref(), "#00FF0040");
}

#[test]
fn alpha_checkerboard_colors_are_stable_and_alternating() {
    let light = Color::from_srgb_hex_rgb(CHECKERBOARD_LIGHT_RGB);
    let dark = Color::from_srgb_hex_rgb(CHECKERBOARD_DARK_RGB);

    assert_ne!(light, dark);
    assert_eq!(checkerboard_cell_color(0, 0), light);
    assert_eq!(checkerboard_cell_color(0, 1), dark);
    assert_eq!(checkerboard_cell_color(1, 0), dark);
    assert_eq!(checkerboard_cell_color(1, 1), light);
}

#[test]
fn alpha_bar_position_maps_local_x_to_clamped_alpha() {
    assert_eq!(alpha_from_local_x(-10.0, 100.0), 0.0);
    assert_eq!(alpha_from_local_x(0.0, 100.0), 0.0);
    assert!((alpha_from_local_x(37.5, 100.0) - 0.375).abs() < f32::EPSILON);
    assert_eq!(alpha_from_local_x(120.0, 100.0), 1.0);
    assert_eq!(alpha_from_local_x(10.0, 0.0), 0.0);
}

#[test]
fn alpha_percent_text_rounds_for_a11y_value() {
    assert_eq!(alpha_percent_text(0.0).as_ref(), "0%");
    assert_eq!(alpha_percent_text(0.374).as_ref(), "37%");
    assert_eq!(alpha_percent_text(1.2).as_ref(), "100%");
}

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
