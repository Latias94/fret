use fret_core::{PathCommand, Point};

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
