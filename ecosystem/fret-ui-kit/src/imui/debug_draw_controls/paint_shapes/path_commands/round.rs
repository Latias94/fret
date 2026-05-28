use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use super::super::super::DebugDrawCommand;

mod filled;
mod stroked;

pub(super) fn paint_round_path_shape_command(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    command: &DebugDrawCommand,
    scale: f32,
) -> bool {
    if stroked::paint_stroked_round_path_shape_command(painter, key, order, command, scale) {
        return true;
    }
    filled::paint_filled_round_path_shape_command(painter, key, order, command, scale)
}
