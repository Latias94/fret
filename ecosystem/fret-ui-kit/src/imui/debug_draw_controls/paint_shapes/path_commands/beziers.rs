use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use super::super::super::DebugDrawCommand;
use super::super::paths;

pub(super) fn paint_bezier_path_shape_command(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    command: &DebugDrawCommand,
    scale: f32,
) -> bool {
    match command {
        DebugDrawCommand::BezierQuadratic {
            from,
            ctrl,
            to,
            color,
            style,
        } => {
            paths::paint_bezier_quadratic(
                painter, key, order, *from, *ctrl, *to, *color, *style, scale,
            );
            true
        }
        DebugDrawCommand::BezierCubic {
            from,
            ctrl1,
            ctrl2,
            to,
            color,
            style,
        } => {
            paths::paint_bezier_cubic(
                painter, key, order, *from, *ctrl1, *ctrl2, *to, *color, *style, scale,
            );
            true
        }
        _ => false,
    }
}
