use fret_core::{PathCommand, Point};

pub(in crate::imui::debug_draw_controls) fn bezier_quadratic_path(
    from: Point,
    ctrl: Point,
    to: Point,
) -> [PathCommand; 2] {
    [PathCommand::MoveTo(from), PathCommand::QuadTo { ctrl, to }]
}

pub(in crate::imui::debug_draw_controls) fn bezier_cubic_path(
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
