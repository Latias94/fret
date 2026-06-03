use std::f32::consts::PI;

use super::super::{HsvColor, sanitize_hue, sanitize_unit};
use super::geometry::HueWheelGeometry;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::controls::color_edit) struct HueWheelTriangle {
    pub(in crate::controls::color_edit) hue: (f32, f32),
    pub(in crate::controls::color_edit) black: (f32, f32),
    pub(in crate::controls::color_edit) white: (f32, f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Barycentric {
    pub(super) u: f32,
    pub(super) v: f32,
    pub(super) w: f32,
}

const TAU: f32 = PI * 2.0;

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
    let geometry = super::geometry::hue_wheel_geometry(width, height);
    let triangle = hue_wheel_rotated_triangle(geometry, hsv.hue);
    let saturation = sanitize_unit(hsv.saturation);
    let value = sanitize_unit(hsv.value);
    let hue_white = lerp_point(triangle.white, triangle.hue, saturation);
    lerp_point(hue_white, triangle.black, 1.0 - value)
}

pub(super) fn hue_wheel_unrotated_triangle(geometry: HueWheelGeometry) -> HueWheelTriangle {
    let r = geometry.triangle_r;
    HueWheelTriangle {
        hue: (r, 0.0),
        black: (r * -0.5, r * -0.866_025_4),
        white: (r * -0.5, r * 0.866_025_4),
    }
}

pub(super) fn hue_wheel_unrotate_local_point(hue: f32, point: (f32, f32)) -> (f32, f32) {
    let angle = -sanitize_hue(hue) * TAU;
    rotate_point(point, angle.cos(), angle.sin())
}

pub(super) fn triangle_contains_point(triangle: HueWheelTriangle, point: (f32, f32)) -> bool {
    let barycentric = triangle_barycentric_coords(triangle, point);
    barycentric.u >= -0.0001 && barycentric.v >= -0.0001 && barycentric.w >= -0.0001
}

pub(super) fn triangle_barycentric_coords(
    triangle: HueWheelTriangle,
    point: (f32, f32),
) -> Barycentric {
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

pub(super) fn triangle_closest_point(triangle: HueWheelTriangle, point: (f32, f32)) -> (f32, f32) {
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
