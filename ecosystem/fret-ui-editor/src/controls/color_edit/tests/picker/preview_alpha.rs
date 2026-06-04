use super::super::*;

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
fn opaque_alpha_preview_keeps_rgb_and_forces_preview_alpha() {
    let mut color = Color::from_srgb_hex_rgb(0x40_80_c0);
    color.a = 0.25;
    let opaque = opaque_preview_color(color);

    assert_eq!(opaque.r, color.r);
    assert_eq!(opaque.g, color.g);
    assert_eq!(opaque.b, color.b);
    assert_eq!(opaque.a, 1.0);
}

#[test]
fn popup_preview_hides_alpha_when_alpha_editing_is_not_visible() {
    let mut color = Color::from_srgb_hex_rgb(0x40_80_c0);
    color.a = 0.25;

    assert_eq!(preview_color_for_alpha_visibility(color, true), color);

    let opaque = preview_color_for_alpha_visibility(color, false);
    assert_eq!(opaque.r, color.r);
    assert_eq!(opaque.g, color.g);
    assert_eq!(opaque.b, color.b);
    assert_eq!(opaque.a, 1.0);
}

#[test]
fn popup_original_restore_matches_imgui_component_count_rules() {
    let mut current = Color::from_srgb_hex_rgb(0x11_22_33);
    current.a = 0.25;
    let mut original = Color::from_srgb_hex_rgb(0xef_44_44);
    original.a = 0.75;

    let restored_rgb = restore_reference_color(original, current, false);
    assert_eq!(restored_rgb.to_srgb_hex_rgb(), original.to_srgb_hex_rgb());
    assert!((restored_rgb.a - current.a).abs() < f32::EPSILON);

    let restored_rgba = restore_reference_color(original, current, true);
    assert_eq!(restored_rgba, original);
}

#[test]
fn alpha_percent_text_rounds_for_a11y_value() {
    assert_eq!(alpha_percent_text(0.0).as_ref(), "0%");
    assert_eq!(alpha_percent_text(0.374).as_ref(), "37%");
    assert_eq!(alpha_percent_text(1.2).as_ref(), "100%");
}
