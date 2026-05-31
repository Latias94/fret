use std::f32::consts::PI;

use super::{HsvColor, sanitize_hue, sanitize_unit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::controls::color_edit) enum HueWheelDragTarget {
    Hue,
    SaturationValue,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::controls::color_edit) struct HueWheelGeometry {
    pub(in crate::controls::color_edit) center_x: f32,
    pub(in crate::controls::color_edit) center_y: f32,
    pub(in crate::controls::color_edit) wheel_r_outer: f32,
    pub(in crate::controls::color_edit) wheel_r_inner: f32,
    pub(in crate::controls::color_edit) wheel_thickness: f32,
    pub(in crate::controls::color_edit) triangle_r: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::controls::color_edit) struct HueWheelTriangle {
    pub(in crate::controls::color_edit) hue: (f32, f32),
    pub(in crate::controls::color_edit) black: (f32, f32),
    pub(in crate::controls::color_edit) white: (f32, f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Barycentric {
    u: f32,
    v: f32,
    w: f32,
}

const HUE_WHEEL_SV_MIN: f32 = 0.0001;
const TAU: f32 = PI * 2.0;

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

pub(in crate::controls::color_edit) fn hue_wheel_rotated_triangle(
    geometry: HueWheelGeometry,
    hue: f32,
) -> HueWheelTriangle {
    let triangle = hue_wheel_unrotated_triangle(geometry);
    let angle = sanitize_hue(hue) * TAU;
    let cos = angle.cos();
    let sin = angle.sin();
    HueWheelTriangle {
        hue: rotate_and_translate(triangle.hue, geometry, cos, sin),
        black: rotate_and_translate(triangle.black, geometry, cos, sin),
        white: rotate_and_translate(triangle.white, geometry, cos, sin),
    }
}

pub(in crate::controls::color_edit) fn hue_wheel_sv_cursor_position(
    hsv: HsvColor,
    width: f32,
    height: f32,
) -> (f32, f32) {
    let geometry = hue_wheel_geometry(width, height);
    let triangle = hue_wheel_rotated_triangle(geometry, hsv.hue);
    let saturation = sanitize_unit(hsv.saturation);
    let value = sanitize_unit(hsv.value);
    let hue_white = lerp_point(triangle.white, triangle.hue, saturation);
    lerp_point(hue_white, triangle.black, 1.0 - value)
}

pub(in crate::controls::color_edit) fn hue_wheel_target_from_local_position(
    current: HsvColor,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> Option<HueWheelDragTarget> {
    if !x.is_finite() || !y.is_finite() {
        return None;
    }

    let geometry = hue_wheel_geometry(width, height);
    if geometry.wheel_r_outer <= f32::EPSILON {
        return None;
    }

    let local = (x - geometry.center_x, y - geometry.center_y);
    let dist2 = local.0 * local.0 + local.1 * local.1;
    let inner = (geometry.wheel_r_inner - 1.0).max(0.0);
    let outer = geometry.wheel_r_outer + 1.0;
    if dist2 >= inner * inner && dist2 <= outer * outer {
        return Some(HueWheelDragTarget::Hue);
    }

    if geometry.triangle_r <= f32::EPSILON {
        return None;
    }

    let unrotated = hue_wheel_unrotate_local_point(current.hue, local);
    let triangle = hue_wheel_unrotated_triangle(geometry);
    triangle_contains_point(triangle, unrotated).then_some(HueWheelDragTarget::SaturationValue)
}

pub(in crate::controls::color_edit) fn hsv_with_hue_wheel_position(
    current: HsvColor,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    target: HueWheelDragTarget,
) -> HsvColor {
    if !x.is_finite() || !y.is_finite() {
        return current;
    }

    let geometry = hue_wheel_geometry(width, height);
    if geometry.wheel_r_outer <= f32::EPSILON {
        return current;
    }

    let local = (x - geometry.center_x, y - geometry.center_y);
    match target {
        HueWheelDragTarget::Hue => HsvColor {
            hue: sanitize_hue(local.1.atan2(local.0) / TAU),
            saturation: sanitize_unit(current.saturation),
            value: sanitize_unit(current.value),
        },
        HueWheelDragTarget::SaturationValue => {
            if geometry.triangle_r <= f32::EPSILON {
                return current;
            }
            let triangle = hue_wheel_unrotated_triangle(geometry);
            let mut unrotated = hue_wheel_unrotate_local_point(current.hue, local);
            if !triangle_contains_point(triangle, unrotated) {
                unrotated = triangle_closest_point(triangle, unrotated);
            }
            let barycentric = triangle_barycentric_coords(triangle, unrotated);
            let value = (1.0 - barycentric.v).clamp(HUE_WHEEL_SV_MIN, 1.0);
            let saturation = (barycentric.u / value).clamp(HUE_WHEEL_SV_MIN, 1.0);
            HsvColor {
                hue: sanitize_hue(current.hue),
                saturation,
                value,
            }
        }
    }
}

fn finite_positive_or_zero(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        0.0
    }
}

fn hue_wheel_unrotated_triangle(geometry: HueWheelGeometry) -> HueWheelTriangle {
    let r = geometry.triangle_r;
    HueWheelTriangle {
        hue: (r, 0.0),
        black: (r * -0.5, r * -0.866_025_4),
        white: (r * -0.5, r * 0.866_025_4),
    }
}

fn hue_wheel_unrotate_local_point(hue: f32, point: (f32, f32)) -> (f32, f32) {
    let angle = -sanitize_hue(hue) * TAU;
    rotate_point(point, angle.cos(), angle.sin())
}

fn rotate_and_translate(
    point: (f32, f32),
    geometry: HueWheelGeometry,
    cos: f32,
    sin: f32,
) -> (f32, f32) {
    let rotated = rotate_point(point, cos, sin);
    (geometry.center_x + rotated.0, geometry.center_y + rotated.1)
}

fn rotate_point(point: (f32, f32), cos: f32, sin: f32) -> (f32, f32) {
    (point.0 * cos - point.1 * sin, point.0 * sin + point.1 * cos)
}

fn lerp_point(a: (f32, f32), b: (f32, f32), t: f32) -> (f32, f32) {
    let t = sanitize_unit(t);
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
}

fn triangle_contains_point(triangle: HueWheelTriangle, point: (f32, f32)) -> bool {
    let barycentric = triangle_barycentric_coords(triangle, point);
    barycentric.u >= -0.0001 && barycentric.v >= -0.0001 && barycentric.w >= -0.0001
}

fn triangle_barycentric_coords(triangle: HueWheelTriangle, point: (f32, f32)) -> Barycentric {
    let a = triangle.hue;
    let b = triangle.black;
    let c = triangle.white;
    let v0 = (b.0 - a.0, b.1 - a.1);
    let v1 = (c.0 - a.0, c.1 - a.1);
    let v2 = (point.0 - a.0, point.1 - a.1);
    let d00 = dot(v0, v0);
    let d01 = dot(v0, v1);
    let d11 = dot(v1, v1);
    let d20 = dot(v2, v0);
    let d21 = dot(v2, v1);
    let denom = d00 * d11 - d01 * d01;
    if denom.abs() <= f32::EPSILON {
        return Barycentric {
            u: 1.0,
            v: 0.0,
            w: 0.0,
        };
    }
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;
    Barycentric { u, v, w }
}

fn triangle_closest_point(triangle: HueWheelTriangle, point: (f32, f32)) -> (f32, f32) {
    let candidates = [
        closest_point_on_segment(point, triangle.hue, triangle.black),
        closest_point_on_segment(point, triangle.black, triangle.white),
        closest_point_on_segment(point, triangle.white, triangle.hue),
    ];
    candidates
        .into_iter()
        .min_by(|a, b| {
            distance2(*a, point)
                .partial_cmp(&distance2(*b, point))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(point)
}

fn closest_point_on_segment(point: (f32, f32), a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
    let ab = (b.0 - a.0, b.1 - a.1);
    let denom = dot(ab, ab);
    if denom <= f32::EPSILON {
        return a;
    }
    let ap = (point.0 - a.0, point.1 - a.1);
    let t = (dot(ap, ab) / denom).clamp(0.0, 1.0);
    (a.0 + ab.0 * t, a.1 + ab.1 * t)
}

fn dot(a: (f32, f32), b: (f32, f32)) -> f32 {
    a.0 * b.0 + a.1 * b.1
}

fn distance2(a: (f32, f32), b: (f32, f32)) -> f32 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    dx * dx + dy * dy
}
