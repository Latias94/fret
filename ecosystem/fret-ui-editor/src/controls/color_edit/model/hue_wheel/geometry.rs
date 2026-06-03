#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::controls::color_edit) struct HueWheelGeometry {
    pub(in crate::controls::color_edit) center_x: f32,
    pub(in crate::controls::color_edit) center_y: f32,
    pub(in crate::controls::color_edit) wheel_r_outer: f32,
    pub(in crate::controls::color_edit) wheel_r_inner: f32,
    pub(in crate::controls::color_edit) wheel_thickness: f32,
    pub(in crate::controls::color_edit) triangle_r: f32,
}

pub(in crate::controls::color_edit) fn hue_wheel_geometry(
    width: f32,
    height: f32,
) -> HueWheelGeometry {
    let width = finite_positive_or_zero(width);
    let height = finite_positive_or_zero(height);
    let side = width.min(height);
    let wheel_r_outer = side * 0.5;
    let wheel_thickness = side * 0.08;
    let wheel_r_inner = (wheel_r_outer - wheel_thickness).max(0.0);
    let triangle_r = (wheel_r_inner - (side * 0.027).trunc()).max(0.0);

    HueWheelGeometry {
        center_x: width * 0.5,
        center_y: height * 0.5,
        wheel_r_outer,
        wheel_r_inner,
        wheel_thickness,
        triangle_r,
    }
}

fn finite_positive_or_zero(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        0.0
    }
}
