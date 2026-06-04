use super::super::*;

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
