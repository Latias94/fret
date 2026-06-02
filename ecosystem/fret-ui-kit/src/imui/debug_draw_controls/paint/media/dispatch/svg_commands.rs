use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::{DebugDrawCommand, DebugDrawMediaCommand};

use super::super::{MediaPaintKey, svg};

pub(super) fn paint_svg_media_command(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    command: &DebugDrawCommand,
) -> bool {
    match command {
        DebugDrawCommand::Media(DebugDrawMediaCommand::SvgImage { rect, svg, options }) => {
            svg::paint_svg_image_command(
                painter,
                MediaPaintKey {
                    key,
                    order,
                    rect: *rect,
                },
                svg,
                *options,
            );
            true
        }
        DebugDrawCommand::Media(DebugDrawMediaCommand::SvgMaskIcon {
            rect,
            svg,
            color,
            options,
        }) => {
            svg::paint_svg_mask_icon_command(
                painter,
                MediaPaintKey {
                    key,
                    order,
                    rect: *rect,
                },
                svg,
                *color,
                *options,
            );
            true
        }
        _ => false,
    }
}
