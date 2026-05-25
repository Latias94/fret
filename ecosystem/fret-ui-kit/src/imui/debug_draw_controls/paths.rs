use fret_core::{PathCommand, Point, Px, Size};

use super::DEFAULT_ELLIPSE_SEGMENTS;

mod rects;
mod sampling;

pub(super) use rects::{append_path_rect_points, rect_path};
pub(super) use sampling::{
    append_arc_points, append_elliptical_arc_points, cubic_bezier_point, path_arc_segments,
    path_bezier_segments, path_elliptical_arc_segments, quadratic_bezier_point,
};

pub(super) fn path_stroke_required_points(closed: bool) -> usize {
    if closed { 3 } else { 2 }
}

pub(super) fn polyline_path(points: &[Point], closed: bool) -> Option<Vec<PathCommand>> {
    if points.len() < path_stroke_required_points(closed) {
        return None;
    }

    let mut commands = Vec::with_capacity(points.len() + usize::from(closed));
    commands.push(PathCommand::MoveTo(points[0]));
    for point in &points[1..] {
        commands.push(PathCommand::LineTo(*point));
    }
    if closed {
        commands.push(PathCommand::Close);
    }
    Some(commands)
}

pub(super) fn convex_poly_fill_path(points: &[Point]) -> Option<Vec<PathCommand>> {
    polyline_path(points, true)
}

pub(super) fn concave_poly_fill_path(points: &[Point]) -> Option<Vec<PathCommand>> {
    polyline_path(points, true)
}

pub(super) fn triangle_path(p1: Point, p2: Point, p3: Point) -> [PathCommand; 4] {
    [
        PathCommand::MoveTo(p1),
        PathCommand::LineTo(p2),
        PathCommand::LineTo(p3),
        PathCommand::Close,
    ]
}

pub(super) fn quad_path(p1: Point, p2: Point, p3: Point, p4: Point) -> [PathCommand; 5] {
    [
        PathCommand::MoveTo(p1),
        PathCommand::LineTo(p2),
        PathCommand::LineTo(p3),
        PathCommand::LineTo(p4),
        PathCommand::Close,
    ]
}

pub(super) fn circle_path(center: Point, radius: Px) -> [PathCommand; 6] {
    let r = radius.0;
    let k = 0.552_284_8_f32 * r;
    let cx = center.x.0;
    let cy = center.y.0;
    [
        PathCommand::MoveTo(Point::new(Px(cx + r), Px(cy))),
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx + r), Px(cy + k)),
            ctrl2: Point::new(Px(cx + k), Px(cy + r)),
            to: Point::new(Px(cx), Px(cy + r)),
        },
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx - k), Px(cy + r)),
            ctrl2: Point::new(Px(cx - r), Px(cy + k)),
            to: Point::new(Px(cx - r), Px(cy)),
        },
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx - r), Px(cy - k)),
            ctrl2: Point::new(Px(cx - k), Px(cy - r)),
            to: Point::new(Px(cx), Px(cy - r)),
        },
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx + k), Px(cy - r)),
            ctrl2: Point::new(Px(cx + r), Px(cy - k)),
            to: Point::new(Px(cx + r), Px(cy)),
        },
        PathCommand::Close,
    ]
}

pub(super) fn ngon_path(center: Point, radius: Px, segments: usize) -> Option<Vec<PathCommand>> {
    if segments < 3 || radius.0 <= 0.0 || !radius.0.is_finite() {
        return None;
    }

    let mut commands = Vec::with_capacity(segments.checked_add(1)?);
    for index in 0..segments {
        let angle = std::f32::consts::TAU * index as f32 / segments as f32;
        let (sin, cos) = angle.sin_cos();
        let point = Point::new(
            Px(center.x.0 + cos * radius.0),
            Px(center.y.0 + sin * radius.0),
        );
        if index == 0 {
            commands.push(PathCommand::MoveTo(point));
        } else {
            commands.push(PathCommand::LineTo(point));
        }
    }
    commands.push(PathCommand::Close);
    Some(commands)
}

pub(super) fn ellipse_path(
    center: Point,
    radius: Size,
    rotation_radians: f32,
    segments: usize,
) -> Option<Vec<PathCommand>> {
    let segments = if segments == 0 {
        DEFAULT_ELLIPSE_SEGMENTS
    } else {
        segments
    };
    if segments < 3
        || radius.width.0 <= 0.0
        || radius.height.0 <= 0.0
        || !radius.width.0.is_finite()
        || !radius.height.0.is_finite()
        || !rotation_radians.is_finite()
    {
        return None;
    }

    let (rot_sin, rot_cos) = rotation_radians.sin_cos();
    let mut commands = Vec::with_capacity(segments.checked_add(1)?);
    for index in 0..segments {
        let angle = std::f32::consts::TAU * index as f32 / segments as f32;
        let (angle_sin, angle_cos) = angle.sin_cos();
        let x = angle_cos * radius.width.0;
        let y = angle_sin * radius.height.0;
        let point = Point::new(
            Px(center.x.0 + x * rot_cos - y * rot_sin),
            Px(center.y.0 + x * rot_sin + y * rot_cos),
        );
        if index == 0 {
            commands.push(PathCommand::MoveTo(point));
        } else {
            commands.push(PathCommand::LineTo(point));
        }
    }
    commands.push(PathCommand::Close);
    Some(commands)
}

pub(super) fn bezier_quadratic_path(from: Point, ctrl: Point, to: Point) -> [PathCommand; 2] {
    [PathCommand::MoveTo(from), PathCommand::QuadTo { ctrl, to }]
}

pub(super) fn bezier_cubic_path(
    from: Point,
    ctrl1: Point,
    ctrl2: Point,
    to: Point,
) -> [PathCommand; 2] {
    [
        PathCommand::MoveTo(from),
        PathCommand::CubicTo { ctrl1, ctrl2, to },
    ]
}
