use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use super::super::DebugDrawCommand;

mod beziers;
mod linear;
mod round;

pub(super) fn paint_path_shape_command(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    command: &DebugDrawCommand,
    scale: f32,
) -> bool {
    if linear::paint_linear_path_shape_command(painter, key, order, command, scale) {
        return true;
    }
    if round::paint_round_path_shape_command(painter, key, order, command, scale) {
        return true;
    }
    beziers::paint_bezier_path_shape_command(painter, key, order, command, scale)
}
