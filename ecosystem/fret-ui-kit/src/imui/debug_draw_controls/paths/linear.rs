use fret_core::{PathCommand, Point};

pub(in crate::imui::debug_draw_controls) fn path_stroke_required_points(closed: bool) -> usize {
    if closed { 3 } else { 2 }
}

pub(in crate::imui::debug_draw_controls) fn polyline_path(
    points: &[Point],
    closed: bool,
) -> Option<Vec<PathCommand>> {
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

pub(in crate::imui::debug_draw_controls) fn convex_poly_fill_path(
    points: &[Point],
) -> Option<Vec<PathCommand>> {
    polyline_path(points, true)
}

pub(in crate::imui::debug_draw_controls) fn concave_poly_fill_path(
    points: &[Point],
) -> Option<Vec<PathCommand>> {
    polyline_path(points, true)
}

pub(in crate::imui::debug_draw_controls) fn triangle_path(
    p1: Point,
    p2: Point,
    p3: Point,
) -> [PathCommand; 4] {
    [
        PathCommand::MoveTo(p1),
        PathCommand::LineTo(p2),
        PathCommand::LineTo(p3),
        PathCommand::Close,
    ]
}

pub(in crate::imui::debug_draw_controls) fn quad_path(
    p1: Point,
    p2: Point,
    p3: Point,
    p4: Point,
) -> [PathCommand; 5] {
    [
        PathCommand::MoveTo(p1),
        PathCommand::LineTo(p2),
        PathCommand::LineTo(p3),
        PathCommand::LineTo(p4),
        PathCommand::Close,
    ]
}
