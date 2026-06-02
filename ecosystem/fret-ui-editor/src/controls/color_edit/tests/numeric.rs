use super::*;

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
    for entry in default_color_edit_palette().iter() {
        let hsv = rgb_to_hsv(entry.rgb);
        assert_eq!(hsv_to_rgb(hsv), entry.rgb);
    }
}
