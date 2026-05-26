use fret_core::{Point, Px, Size};

use super::super::{
    DEFAULT_PATH_ARC_SEGMENTS, DEFAULT_PATH_BEZIER_SEGMENTS, DEFAULT_PATH_ELLIPTICAL_ARC_SEGMENTS,
};

pub(in crate::imui::debug_draw_controls) fn path_arc_segments(segments: usize) -> usize {
    if segments == 0 {
        DEFAULT_PATH_ARC_SEGMENTS
    } else {
        segments
    }
}

pub(in crate::imui::debug_draw_controls) fn path_bezier_segments(segments: usize) -> usize {
    if segments == 0 {
        DEFAULT_PATH_BEZIER_SEGMENTS
    } else {
        segments
    }
}

pub(in crate::imui::debug_draw_controls) fn path_elliptical_arc_segments(segments: usize) -> usize {
    if segments == 0 {
        DEFAULT_PATH_ELLIPTICAL_ARC_SEGMENTS
    } else {
        segments
    }
}

pub(in crate::imui::debug_draw_controls) fn append_arc_points(
    points: &mut Vec<Point>,
    center: Point,
    radius: Px,
    a_min: f32,
    a_max: f32,
    segments: usize,
) {
    for step in 0..=segments {
        let t = if segments == 0 {
            0.0
        } else {
            step as f32 / segments as f32
        };
        points.push(arc_point(center, radius, a_min + t * (a_max - a_min)));
    }
}

fn arc_point(center: Point, radius: Px, angle: f32) -> Point {
    let (sin, cos) = angle.sin_cos();
    Point::new(
        Px(center.x.0 + cos * radius.0),
        Px(center.y.0 + sin * radius.0),
    )
}

pub(in crate::imui::debug_draw_controls) fn append_elliptical_arc_points(
    points: &mut Vec<Point>,
    center: Point,
    radius: Size,
    rotation_radians: f32,
    a_min: f32,
    a_max: f32,
    segments: usize,
) {
    for step in 0..=segments {
        let t = if segments == 0 {
            0.0
        } else {
            step as f32 / segments as f32
        };
        points.push(elliptical_arc_point(
            center,
            radius,
            rotation_radians,
            a_min + t * (a_max - a_min),
        ));
    }
}

fn elliptical_arc_point(center: Point, radius: Size, rotation_radians: f32, angle: f32) -> Point {
    let (angle_sin, angle_cos) = angle.sin_cos();
    let (rot_sin, rot_cos) = rotation_radians.sin_cos();
    let x = angle_cos * radius.width.0;
    let y = angle_sin * radius.height.0;
    Point::new(
        Px(center.x.0 + x * rot_cos - y * rot_sin),
        Px(center.y.0 + x * rot_sin + y * rot_cos),
    )
}

pub(in crate::imui::debug_draw_controls) fn quadratic_bezier_point(
    from: Point,
    ctrl: Point,
    to: Point,
    t: f32,
) -> Point {
    let u = 1.0 - t;
    Point::new(
        Px(u * u * from.x.0 + 2.0 * u * t * ctrl.x.0 + t * t * to.x.0),
        Px(u * u * from.y.0 + 2.0 * u * t * ctrl.y.0 + t * t * to.y.0),
    )
}

pub(in crate::imui::debug_draw_controls) fn cubic_bezier_point(
    from: Point,
    ctrl1: Point,
    ctrl2: Point,
    to: Point,
    t: f32,
) -> Point {
    let u = 1.0 - t;
    let uu = u * u;
    let tt = t * t;
    Point::new(
        Px(uu * u * from.x.0
            + 3.0 * uu * t * ctrl1.x.0
            + 3.0 * u * tt * ctrl2.x.0
            + tt * t * to.x.0),
        Px(uu * u * from.y.0
            + 3.0 * uu * t * ctrl1.y.0
            + 3.0 * u * tt * ctrl2.y.0
            + tt * t * to.y.0),
    )
}
