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
