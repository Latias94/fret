use super::super::*;

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
