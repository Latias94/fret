use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use super::super::super::super::DebugDrawCommand;
use super::super::super::paths;

pub(super) fn paint_stroked_linear_path_shape_command(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    command: &DebugDrawCommand,
    scale: f32,
) -> bool {
    match command {
        DebugDrawCommand::Line {
            from,
            to,
            color,
            style,
        } => {
            paths::paint_line(painter, key, order, *from, *to, *color, *style, scale);
            true
        }
        DebugDrawCommand::Polyline {
            points,
            color,
            style,
            closed,
        } => {
            paths::paint_polyline(painter, key, order, points, *color, *style, *closed, scale);
            true
        }
        DebugDrawCommand::Rect { rect, color, style } => {
            paths::paint_rect(painter, key, order, *rect, *color, *style, scale);
            true
        }
        DebugDrawCommand::Quad {
            p1,
            p2,
            p3,
            p4,
            color,
            style,
        } => {
            paths::paint_quad(
                painter,
                key,
                order,
                [*p1, *p2, *p3, *p4],
                *color,
                *style,
                scale,
            );
            true
        }
        DebugDrawCommand::Triangle {
            p1,
            p2,
            p3,
            color,
            style,
        } => {
            paths::paint_triangle(painter, key, order, [*p1, *p2, *p3], *color, *style, scale);
            true
        }
        _ => false,
    }
}
