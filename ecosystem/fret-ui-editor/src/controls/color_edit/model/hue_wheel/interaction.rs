use std::f32::consts::PI;

use super::super::{HsvColor, sanitize_hue, sanitize_unit};
use super::geometry::hue_wheel_geometry;
use super::triangle::{
    hue_wheel_unrotate_local_point, hue_wheel_unrotated_triangle, triangle_barycentric_coords,
    triangle_closest_point, triangle_contains_point,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::controls::color_edit) enum HueWheelDragTarget {
    Hue,
    SaturationValue,
}

const HUE_WHEEL_SV_MIN: f32 = 0.0001;
const TAU: f32 = PI * 2.0;

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
