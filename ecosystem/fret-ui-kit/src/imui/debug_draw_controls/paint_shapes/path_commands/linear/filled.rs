use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use super::super::super::super::DebugDrawCommand;
use super::super::super::paths;

pub(super) fn paint_filled_linear_path_shape_command(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    command: &DebugDrawCommand,
    scale: f32,
) -> bool {
    match command {
        DebugDrawCommand::ConvexPolyFilled { points, color } => {
            paths::paint_convex_poly_filled(painter, key, order, points, *color, scale);
            true
        }
        DebugDrawCommand::ConcavePolyFilled { points, color } => {
            paths::paint_concave_poly_filled(painter, key, order, points, *color, scale);
            true
        }
        DebugDrawCommand::QuadFilled {
            p1,
            p2,
            p3,
            p4,
            color,
        } => {
            paths::paint_quad_filled(painter, key, order, [*p1, *p2, *p3, *p4], *color, scale);
            true
        }
        DebugDrawCommand::TriangleFilled { p1, p2, p3, color } => {
            paths::paint_triangle_filled(painter, key, order, [*p1, *p2, *p3], *color, scale);
            true
        }
        _ => false,
    }
}
