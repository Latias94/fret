use fret_core::{PathCommand, Point, Px, Rect, Size};

use super::{
    DEFAULT_ELLIPSE_SEGMENTS, DEFAULT_PATH_ARC_SEGMENTS, DEFAULT_PATH_BEZIER_SEGMENTS,
    DEFAULT_PATH_ELLIPTICAL_ARC_SEGMENTS, DebugDrawRoundCorners, effective_rect_rounding,
};

pub(super) fn path_stroke_required_points(closed: bool) -> usize {
    if closed { 3 } else { 2 }
}

pub(super) fn path_arc_segments(segments: usize) -> usize {
    if segments == 0 {
        DEFAULT_PATH_ARC_SEGMENTS
    } else {
        segments
    }
}

pub(super) fn path_bezier_segments(segments: usize) -> usize {
    if segments == 0 {
        DEFAULT_PATH_BEZIER_SEGMENTS
    } else {
        segments
    }
}

pub(super) fn path_elliptical_arc_segments(segments: usize) -> usize {
    if segments == 0 {
        DEFAULT_PATH_ELLIPTICAL_ARC_SEGMENTS
    } else {
        segments
    }
}

pub(super) fn append_arc_points(
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

pub(super) fn append_elliptical_arc_points(
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

pub(super) fn quadratic_bezier_point(from: Point, ctrl: Point, to: Point, t: f32) -> Point {
    let u = 1.0 - t;
    Point::new(
        Px(u * u * from.x.0 + 2.0 * u * t * ctrl.x.0 + t * t * to.x.0),
        Px(u * u * from.y.0 + 2.0 * u * t * ctrl.y.0 + t * t * to.y.0),
    )
}

pub(super) fn cubic_bezier_point(
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

pub(super) fn rect_path(rect: Rect) -> [PathCommand; 5] {
    let x0 = rect.origin.x;
    let y0 = rect.origin.y;
    let x1 = Px(rect.origin.x.0 + rect.size.width.0);
    let y1 = Px(rect.origin.y.0 + rect.size.height.0);
    [
        PathCommand::MoveTo(Point::new(x0, y0)),
        PathCommand::LineTo(Point::new(x1, y0)),
        PathCommand::LineTo(Point::new(x1, y1)),
        PathCommand::LineTo(Point::new(x0, y1)),
        PathCommand::Close,
    ]
}

pub(super) fn append_path_rect_points(
    points: &mut Vec<Point>,
    rect: Rect,
    rounding: Px,
    corners: DebugDrawRoundCorners,
) {
    let a = rect.origin;
    let b = rect_max_point(rect);
    let rounding = effective_rect_rounding(rect, rounding, corners);

    if rounding.0 < 0.5 {
        points.push(a);
        points.push(Point::new(b.x, a.y));
        points.push(b);
        points.push(Point::new(a.x, b.y));
        return;
    }

    let rounding_tl = if corners.contains(DebugDrawRoundCorners::TOP_LEFT) {
        rounding
    } else {
        Px(0.0)
    };
    let rounding_tr = if corners.contains(DebugDrawRoundCorners::TOP_RIGHT) {
        rounding
    } else {
        Px(0.0)
    };
    let rounding_br = if corners.contains(DebugDrawRoundCorners::BOTTOM_RIGHT) {
        rounding
    } else {
        Px(0.0)
    };
    let rounding_bl = if corners.contains(DebugDrawRoundCorners::BOTTOM_LEFT) {
        rounding
    } else {
        Px(0.0)
    };

    append_path_rect_corner_arc_points(
        points,
        Point::new(Px(a.x.0 + rounding_tl.0), Px(a.y.0 + rounding_tl.0)),
        rounding_tl,
        6,
        9,
    );
    append_path_rect_corner_arc_points(
        points,
        Point::new(Px(b.x.0 - rounding_tr.0), Px(a.y.0 + rounding_tr.0)),
        rounding_tr,
        9,
        12,
    );
    append_path_rect_corner_arc_points(
        points,
        Point::new(Px(b.x.0 - rounding_br.0), Px(b.y.0 - rounding_br.0)),
        rounding_br,
        0,
        3,
    );
    append_path_rect_corner_arc_points(
        points,
        Point::new(Px(a.x.0 + rounding_bl.0), Px(b.y.0 - rounding_bl.0)),
        rounding_bl,
        3,
        6,
    );
}

fn append_path_rect_corner_arc_points(
    points: &mut Vec<Point>,
    center: Point,
    radius: Px,
    a_min_of_12: i32,
    a_max_of_12: i32,
) {
    if radius.0 < 0.5 {
        points.push(center);
        return;
    }
    let a_min = a_min_of_12 as f32 * std::f32::consts::TAU / 12.0;
    let a_max = a_max_of_12 as f32 * std::f32::consts::TAU / 12.0;
    append_arc_points(
        points,
        center,
        radius,
        a_min,
        a_max,
        a_min_of_12.abs_diff(a_max_of_12) as usize,
    );
}

fn rect_max_point(rect: Rect) -> Point {
    Point::new(
        Px(rect.origin.x.0 + rect.size.width.0),
        Px(rect.origin.y.0 + rect.size.height.0),
    )
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

pub(super) fn triangle_is_degenerate(p1: Point, p2: Point, p3: Point) -> bool {
    let ax = p2.x.0 - p1.x.0;
    let ay = p2.y.0 - p1.y.0;
    let bx = p3.x.0 - p1.x.0;
    let by = p3.y.0 - p1.y.0;
    (ax * by - ay * bx).abs() <= f32::EPSILON
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
