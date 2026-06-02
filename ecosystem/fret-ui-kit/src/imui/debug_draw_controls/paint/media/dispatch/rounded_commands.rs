use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::{DebugDrawCommand, DebugDrawMediaCommand};

use super::super::rounded;

pub(super) fn paint_rounded_media_command(
    painter: &mut CanvasPainter<'_>,
    order: DrawOrder,
    command: &DebugDrawCommand,
) -> bool {
    match command {
        DebugDrawCommand::Media(DebugDrawMediaCommand::ImageRounded {
            rect,
            image,
            options,
            rounding,
            corners,
        }) => {
            rounded::paint_image_rounded_command(
                painter, order, *rect, *image, *options, *rounding, *corners,
            );
            true
        }
        DebugDrawCommand::Media(DebugDrawMediaCommand::ImageRegionRounded {
            rect,
            image,
            uv,
            options,
            rounding,
            corners,
        }) => {
            rounded::paint_image_region_rounded_command(
                painter, order, *rect, *image, *uv, *options, *rounding, *corners,
            );
            true
        }
        _ => false,
    }
}
