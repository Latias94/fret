use super::super::*;

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
