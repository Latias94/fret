use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use super::super::paint_helpers::{paint_image_triangle_mesh, paint_triangle_mesh};
use super::super::{DebugDrawCommand, DebugDrawLinearCommand, DebugDrawMeshCommand};
use super::{rects, text};

pub(super) fn paint_residual_shape_command(
    painter: &mut CanvasPainter<'_>,
    order: DrawOrder,
    command: &DebugDrawCommand,
    scale: f32,
) {
    match command {
        DebugDrawCommand::Linear(DebugDrawLinearCommand::RectFilled { rect, color }) => {
            rects::paint_rect_filled(painter, order, *rect, *color)
        }
        DebugDrawCommand::Linear(DebugDrawLinearCommand::RectFilledMultiColor {
            rect,
            upper_left,
            upper_right,
            bottom_right,
            bottom_left,
        }) => rects::paint_rect_filled_multi_color(
            painter,
            order,
            *rect,
            [*upper_left, *upper_right, *bottom_right, *bottom_left],
        ),
        DebugDrawCommand::Mesh(DebugDrawMeshCommand::TriangleMesh { vertices, indices }) => {
            paint_triangle_mesh(painter, order, vertices, indices);
        }
        DebugDrawCommand::Mesh(DebugDrawMeshCommand::ImageTriangleMesh {
            image,
            vertices,
            indices,
            options,
        }) => {
            paint_image_triangle_mesh(painter, order, *image, vertices, indices, *options);
        }
        DebugDrawCommand::Text {
            origin,
            text,
            color,
            size,
        } => text::paint_text(painter, order, *origin, text, *color, *size, scale),
        DebugDrawCommand::Linear(_)
        | DebugDrawCommand::Round(_)
        | DebugDrawCommand::BezierQuadratic { .. }
        | DebugDrawCommand::BezierCubic { .. }
        | DebugDrawCommand::Clip(_)
        | DebugDrawCommand::Media(_) => {}
    }
}
