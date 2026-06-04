use super::super::*;

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
