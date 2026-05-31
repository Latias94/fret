use super::*;

#[test]
fn eyedropper_defaults_to_app_owned_opt_in() {
    let options = ColorEditOptions::default();

    assert!(options.on_eyedropper.is_none());
    assert!(options.eyedropper_test_id.is_none());
}

#[test]
fn eyedropper_request_applies_sample_alpha_by_visibility() {
    let mut current = Color::from_srgb_hex_rgb(0x11_22_33);
    current.a = 0.25;
    let mut sampled = Color::from_srgb_hex_rgb(0xef_44_44);
    sampled.a = 0.75;

    let rgb_only = ColorEditEyedropperRequest::new(current, false).apply_sample(sampled);
    assert_eq!(rgb_only.to_srgb_hex_rgb(), 0xef_44_44);
    assert!((rgb_only.a - current.a).abs() < f32::EPSILON);

    let rgba = ColorEditEyedropperRequest::new(current, true).apply_sample(sampled);
    assert_eq!(rgba.to_srgb_hex_rgb(), 0xef_44_44);
    assert!((rgba.a - sampled.a).abs() < f32::EPSILON);
}

#[test]
fn color_tooltip_lines_match_imgui_hex_rgb_hsv_preview_text() {
    let mut color = Color::from_srgb_hex_rgb(0x33_66_99);
    color.a = 0.5;

    let rgb_lines = color_tooltip_lines(color, false);
    assert_eq!(rgb_lines.len(), 3);
    assert_eq!(rgb_lines[0].as_ref(), "#336699");
    assert_eq!(rgb_lines[1].as_ref(), "RGB 51 102 153");
    assert_eq!(rgb_lines[2].as_ref(), "HSV 210deg | S 67% | V 60%");

    let rgba_lines = color_tooltip_lines(color, true);
    assert_eq!(rgba_lines[0].as_ref(), "#33669980");
    assert_eq!(rgba_lines[1].as_ref(), "RGB 51 102 153 | A 50%");
}

#[test]
fn color_copy_entries_match_imgui_copy_as_payloads() {
    let mut color = Color::from_srgb_hex_rgb(0x33_66_99);
    color.a = 0.5;

    let rgb_entries = color_copy_entries(color, false);
    assert_eq!(rgb_entries.len(), 3);
    assert_eq!(rgb_entries[0].format, ColorEditCopyFormat::FloatTuple);
    assert!(rgb_entries[0].text.ends_with(", 1.000f)"));
    assert_eq!(rgb_entries[1].format, ColorEditCopyFormat::IntTuple);
    assert_eq!(rgb_entries[1].text.as_ref(), "(51,102,153,255)");
    assert_eq!(rgb_entries[2].format, ColorEditCopyFormat::HexRgb);
    assert_eq!(rgb_entries[2].text.as_ref(), "#336699");

    let rgba_entries = color_copy_entries(color, true);
    assert_eq!(rgba_entries.len(), 4);
    assert_eq!(rgba_entries[0].format, ColorEditCopyFormat::FloatTuple);
    assert!(rgba_entries[0].text.ends_with(", 0.500f)"));
    assert_eq!(rgba_entries[1].text.as_ref(), "(51,102,153,128)");
    assert_eq!(rgba_entries[2].text.as_ref(), "#336699");
    assert_eq!(rgba_entries[3].format, ColorEditCopyFormat::HexRgba);
    assert_eq!(rgba_entries[3].text.as_ref(), "#33669980");
}
