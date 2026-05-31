use super::*;

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
fn vertical_hue_bar_position_maps_local_y_to_clamped_hue() {
    assert_eq!(hue_from_local_y(-10.0, 100.0), 0.0);
    assert!((hue_from_local_y(37.5, 100.0) - 0.375).abs() < f32::EPSILON);
    assert_eq!(hue_from_local_y(120.0, 100.0), 1.0);
    assert_eq!(hue_from_local_y(10.0, 0.0), 0.0);
}

#[test]
fn hue_wheel_ring_maps_screen_angle_to_hue() {
    let current = HsvColor {
        hue: 0.0,
        saturation: 0.4,
        value: 0.6,
    };
    let width = 138.0;
    let height = 120.0;
    let geometry = hue_wheel_geometry(width, height);
    let radius = (geometry.wheel_r_inner + geometry.wheel_r_outer) * 0.5;

    let right = (geometry.center_x + radius, geometry.center_y);
    assert_eq!(
        hue_wheel_target_from_local_position(current, right.0, right.1, width, height),
        Some(HueWheelDragTarget::Hue)
    );
    assert_hsv_close(
        hsv_with_hue_wheel_position(
            current,
            right.0,
            right.1,
            width,
            height,
            HueWheelDragTarget::Hue,
        ),
        0.0,
        0.4,
        0.6,
    );

    let down = (geometry.center_x, geometry.center_y + radius);
    assert_hsv_close(
        hsv_with_hue_wheel_position(
            current,
            down.0,
            down.1,
            width,
            height,
            HueWheelDragTarget::Hue,
        ),
        0.25,
        0.4,
        0.6,
    );

    let left = (geometry.center_x - radius, geometry.center_y);
    assert_hsv_close(
        hsv_with_hue_wheel_position(
            current,
            left.0,
            left.1,
            width,
            height,
            HueWheelDragTarget::Hue,
        ),
        0.5,
        0.4,
        0.6,
    );

    let up = (geometry.center_x, geometry.center_y - radius);
    assert_hsv_close(
        hsv_with_hue_wheel_position(current, up.0, up.1, width, height, HueWheelDragTarget::Hue),
        0.75,
        0.4,
        0.6,
    );
}

#[test]
fn hue_wheel_triangle_maps_imgui_barycentric_sv() {
    let current = HsvColor {
        hue: 0.0,
        saturation: 0.5,
        value: 0.5,
    };
    let width = 138.0;
    let height = 120.0;
    let geometry = hue_wheel_geometry(width, height);
    let triangle = hue_wheel_rotated_triangle(geometry, current.hue);

    assert_eq!(
        hue_wheel_target_from_local_position(
            current,
            triangle.hue.0,
            triangle.hue.1,
            width,
            height
        ),
        Some(HueWheelDragTarget::SaturationValue)
    );
    assert_hsv_close(
        hsv_with_hue_wheel_position(
            current,
            triangle.hue.0,
            triangle.hue.1,
            width,
            height,
            HueWheelDragTarget::SaturationValue,
        ),
        0.0,
        1.0,
        1.0,
    );

    assert_hsv_close(
        hsv_with_hue_wheel_position(
            current,
            triangle.white.0,
            triangle.white.1,
            width,
            height,
            HueWheelDragTarget::SaturationValue,
        ),
        0.0,
        0.0001,
        1.0,
    );
    assert_hsv_close(
        hsv_with_hue_wheel_position(
            current,
            triangle.black.0,
            triangle.black.1,
            width,
            height,
            HueWheelDragTarget::SaturationValue,
        ),
        0.0,
        0.0001,
        0.0001,
    );

    let centroid = (
        (triangle.hue.0 + triangle.black.0 + triangle.white.0) / 3.0,
        (triangle.hue.1 + triangle.black.1 + triangle.white.1) / 3.0,
    );
    assert_hsv_close(
        hsv_with_hue_wheel_position(
            current,
            centroid.0,
            centroid.1,
            width,
            height,
            HueWheelDragTarget::SaturationValue,
        ),
        0.0,
        0.5,
        2.0 / 3.0,
    );
}

#[test]
fn hue_wheel_triangle_rotates_with_hue() {
    let hsv = HsvColor {
        hue: 0.25,
        saturation: 1.0,
        value: 1.0,
    };
    let width = 138.0;
    let height = 120.0;
    let geometry = hue_wheel_geometry(width, height);
    let triangle = hue_wheel_rotated_triangle(geometry, hsv.hue);
    let cursor = hue_wheel_sv_cursor_position(hsv, width, height);

    assert_eq!(
        hue_wheel_target_from_local_position(hsv, triangle.hue.0, triangle.hue.1, width, height),
        Some(HueWheelDragTarget::SaturationValue)
    );
    assert!((cursor.0 - triangle.hue.0).abs() < 0.002);
    assert!((cursor.1 - triangle.hue.1).abs() < 0.002);
    assert_hsv_close(
        hsv_with_hue_wheel_position(
            hsv,
            triangle.hue.0,
            triangle.hue.1,
            width,
            height,
            HueWheelDragTarget::SaturationValue,
        ),
        0.25,
        1.0,
        1.0,
    );
}

#[test]
fn hue_wheel_target_rejects_outside_or_empty_geometry() {
    let current = HsvColor {
        hue: 0.0,
        saturation: 0.5,
        value: 0.5,
    };

    assert_eq!(
        hue_wheel_target_from_local_position(current, 0.0, 0.0, 138.0, 120.0),
        None
    );
    assert_eq!(
        hue_wheel_target_from_local_position(current, 10.0, 10.0, 0.0, 120.0),
        None
    );
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
fn alpha_bar_position_maps_local_x_to_clamped_alpha() {
    assert_eq!(alpha_from_local_x(-10.0, 100.0), 0.0);
    assert_eq!(alpha_from_local_x(0.0, 100.0), 0.0);
    assert!((alpha_from_local_x(37.5, 100.0) - 0.375).abs() < f32::EPSILON);
    assert_eq!(alpha_from_local_x(120.0, 100.0), 1.0);
    assert_eq!(alpha_from_local_x(10.0, 0.0), 0.0);
}

#[test]
fn vertical_alpha_bar_position_maps_local_y_to_inverted_alpha() {
    assert_eq!(alpha_from_local_y(-10.0, 100.0), 1.0);
    assert_eq!(alpha_from_local_y(0.0, 100.0), 1.0);
    assert!((alpha_from_local_y(37.5, 100.0) - 0.625).abs() < f32::EPSILON);
    assert_eq!(alpha_from_local_y(120.0, 100.0), 0.0);
    assert_eq!(alpha_from_local_y(10.0, 0.0), 1.0);
}

#[test]
fn alpha_percent_text_rounds_for_a11y_value() {
    assert_eq!(alpha_percent_text(0.0).as_ref(), "0%");
    assert_eq!(alpha_percent_text(0.374).as_ref(), "37%");
    assert_eq!(alpha_percent_text(1.2).as_ref(), "100%");
}
