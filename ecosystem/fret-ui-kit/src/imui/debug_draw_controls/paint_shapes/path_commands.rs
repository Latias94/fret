use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use super::super::DebugDrawCommand;
use super::paths;

pub(super) fn paint_path_shape_command(
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
        DebugDrawCommand::ConvexPolyFilled { points, color } => {
            paths::paint_convex_poly_filled(painter, key, order, points, *color, scale);
            true
        }
        DebugDrawCommand::ConcavePolyFilled { points, color } => {
            paths::paint_concave_poly_filled(painter, key, order, points, *color, scale);
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
        DebugDrawCommand::TriangleFilled { p1, p2, p3, color } => {
            paths::paint_triangle_filled(painter, key, order, [*p1, *p2, *p3], *color, scale);
            true
        }
        DebugDrawCommand::Circle {
            center,
            radius,
            color,
            style,
        } => {
            paths::paint_circle(painter, key, order, *center, *radius, *color, *style, scale);
            true
        }
        DebugDrawCommand::CircleFilled {
            center,
            radius,
            color,
        } => {
            paths::paint_circle_filled(painter, key, order, *center, *radius, *color, scale);
            true
        }
        DebugDrawCommand::Ngon {
            center,
            radius,
            segments,
            color,
            style,
        } => {
            paths::paint_ngon(
                painter, key, order, *center, *radius, *segments, *color, *style, scale,
            );
            true
        }
        DebugDrawCommand::NgonFilled {
            center,
            radius,
            segments,
            color,
        } => {
            paths::paint_ngon_filled(
                painter, key, order, *center, *radius, *segments, *color, scale,
            );
            true
        }
        DebugDrawCommand::Ellipse {
            center,
            radius,
            rotation_radians,
            segments,
            color,
            style,
        } => {
            paths::paint_ellipse(
                painter,
                key,
                order,
                *center,
                *radius,
                *rotation_radians,
                *segments,
                *color,
                *style,
                scale,
            );
            true
        }
        DebugDrawCommand::EllipseFilled {
            center,
            radius,
            rotation_radians,
            segments,
            color,
        } => {
            paths::paint_ellipse_filled(
                painter,
                key,
                order,
                *center,
                *radius,
                *rotation_radians,
                *segments,
                *color,
                scale,
            );
            true
        }
        DebugDrawCommand::BezierQuadratic {
            from,
            ctrl,
            to,
            color,
            style,
        } => {
            paths::paint_bezier_quadratic(
                painter, key, order, *from, *ctrl, *to, *color, *style, scale,
            );
            true
        }
        DebugDrawCommand::BezierCubic {
            from,
            ctrl1,
            ctrl2,
            to,
            color,
            style,
        } => {
            paths::paint_bezier_cubic(
                painter, key, order, *from, *ctrl1, *ctrl2, *to, *color, *style, scale,
            );
            true
        }
        _ => false,
    }
}
