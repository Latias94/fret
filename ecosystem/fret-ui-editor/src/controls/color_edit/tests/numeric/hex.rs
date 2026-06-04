use super::super::*;

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
