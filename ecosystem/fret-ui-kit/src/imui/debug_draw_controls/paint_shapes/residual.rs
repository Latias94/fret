use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use super::super::DebugDrawCommand;
use super::super::paint_helpers::{paint_image_triangle_mesh, paint_triangle_mesh};
use super::{rects, text};

pub(super) fn paint_residual_shape_command(
    painter: &mut CanvasPainter<'_>,
    order: DrawOrder,
    command: &DebugDrawCommand,
    scale: f32,
) {
    match command {
        DebugDrawCommand::RectFilled { rect, color } => {
            rects::paint_rect_filled(painter, order, *rect, *color)
        }
        DebugDrawCommand::RectFilledMultiColor {
            rect,
            upper_left,
            upper_right,
            bottom_right,
            bottom_left,
        } => rects::paint_rect_filled_multi_color(
            painter,
            order,
            *rect,
            [*upper_left, *upper_right, *bottom_right, *bottom_left],
        ),
        DebugDrawCommand::TriangleMesh { vertices, indices } => {
            paint_triangle_mesh(painter, order, vertices, indices);
        }
        DebugDrawCommand::ImageTriangleMesh {
            image,
            vertices,
            indices,
            options,
        } => {
            paint_image_triangle_mesh(painter, order, *image, vertices, indices, *options);
        }
        DebugDrawCommand::Text {
            origin,
            text,
            color,
            size,
        } => text::paint_text(painter, order, *origin, text, *color, *size, scale),
        DebugDrawCommand::Line { .. }
        | DebugDrawCommand::Polyline { .. }
        | DebugDrawCommand::ConvexPolyFilled { .. }
        | DebugDrawCommand::ConcavePolyFilled { .. }
        | DebugDrawCommand::Rect { .. }
        | DebugDrawCommand::Quad { .. }
        | DebugDrawCommand::QuadFilled { .. }
        | DebugDrawCommand::Triangle { .. }
        | DebugDrawCommand::TriangleFilled { .. }
        | DebugDrawCommand::Circle { .. }
        | DebugDrawCommand::CircleFilled { .. }
        | DebugDrawCommand::Ngon { .. }
        | DebugDrawCommand::NgonFilled { .. }
        | DebugDrawCommand::Ellipse { .. }
        | DebugDrawCommand::EllipseFilled { .. }
        | DebugDrawCommand::BezierQuadratic { .. }
        | DebugDrawCommand::BezierCubic { .. }
        | DebugDrawCommand::PushClipRect { .. }
        | DebugDrawCommand::PopClipRect
        | DebugDrawCommand::Image { .. }
        | DebugDrawCommand::ImageRegion { .. }
        | DebugDrawCommand::ImageQuad { .. }
        | DebugDrawCommand::ImageRounded { .. }
        | DebugDrawCommand::ImageRegionRounded { .. }
        | DebugDrawCommand::SvgImage { .. }
        | DebugDrawCommand::SvgMaskIcon { .. } => {}
    }
}
