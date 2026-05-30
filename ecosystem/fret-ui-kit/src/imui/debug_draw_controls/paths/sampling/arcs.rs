use fret_core::{Point, Px, Size};

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
