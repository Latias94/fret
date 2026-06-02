use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::{DebugDrawCommand, DebugDrawMediaCommand};

use super::super::raster;

pub(super) fn paint_raster_media_command(
    painter: &mut CanvasPainter<'_>,
    order: DrawOrder,
    command: &DebugDrawCommand,
) -> bool {
    match command {
        DebugDrawCommand::Media(DebugDrawMediaCommand::Image {
            rect,
            image,
            options,
        }) => {
            raster::paint_image_command(painter, order, *rect, *image, *options);
            true
        }
        DebugDrawCommand::Media(DebugDrawMediaCommand::ImageRegion {
            rect,
            image,
            uv,
            options,
        }) => {
            raster::paint_image_region_command(painter, order, *rect, *image, *uv, *options);
            true
        }
        DebugDrawCommand::Media(DebugDrawMediaCommand::ImageQuad {
            image,
            points,
            uvs,
            options,
        }) => {
            raster::paint_image_quad_command(painter, order, *image, *points, *uvs, *options);
            true
        }
        _ => false,
    }
}
