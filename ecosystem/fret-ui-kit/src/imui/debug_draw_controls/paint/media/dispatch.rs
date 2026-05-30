use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::DebugDrawCommand;

mod non_media;
mod raster_commands;
mod rounded_commands;
mod svg_commands;

pub(in crate::imui::debug_draw_controls::paint) fn paint_debug_draw_media_command(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    command: &DebugDrawCommand,
) {
    if raster_commands::paint_raster_media_command(painter, order, command) {
        return;
    }
    if rounded_commands::paint_rounded_media_command(painter, order, command) {
        return;
    }
    if svg_commands::paint_svg_media_command(painter, key, order, command) {
        return;
    }
    debug_assert!(non_media::is_non_media_command(command));
}
