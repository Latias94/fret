use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use super::DebugDrawCommand;

mod path_commands;
mod paths;
mod rects;
mod residual;
mod text;

pub(super) fn paint_debug_draw_shape_command(
    painter: &mut CanvasPainter<'_>,
    index: usize,
    command: &DebugDrawCommand,
    scale: f32,
) {
    let order = DrawOrder(index as u32);
    let key = painter.key(&("fret-ui-kit.imui.debug_draw.command", index));
    if path_commands::paint_path_shape_command(painter, key, order, command, scale) {
        return;
    }

    residual::paint_residual_shape_command(painter, order, command, scale);
}
