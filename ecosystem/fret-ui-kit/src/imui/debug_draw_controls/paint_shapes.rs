use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use super::DebugDrawCommand;
use super::paint_helpers::{paint_image_triangle_mesh, paint_triangle_mesh};

mod paths;
mod rects;
mod text;

pub(super) fn paint_debug_draw_shape_command(
    painter: &mut CanvasPainter<'_>,
    index: usize,
    command: &DebugDrawCommand,
    scale: f32,
) {
    let order = DrawOrder(index as u32);
    let key = painter.key(&("fret-ui-kit.imui.debug_draw.command", index));
    match command {
        DebugDrawCommand::Line {
            from,
            to,
            color,
            style,
        } => paths::paint_line(painter, key, order, *from, *to, *color, *style, scale),
        DebugDrawCommand::Polyline {
            points,
            color,
            style,
            closed,
        } => paths::paint_polyline(painter, key, order, points, *color, *style, *closed, scale),
        DebugDrawCommand::ConvexPolyFilled { points, color } => {
            paths::paint_convex_poly_filled(painter, key, order, points, *color, scale)
        }
        DebugDrawCommand::ConcavePolyFilled { points, color } => {
            paths::paint_concave_poly_filled(painter, key, order, points, *color, scale)
        }
        DebugDrawCommand::Rect { rect, color, style } => {
            paths::paint_rect(painter, key, order, *rect, *color, *style, scale)
        }
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
        DebugDrawCommand::Quad {
            p1,
            p2,
            p3,
            p4,
            color,
            style,
        } => paths::paint_quad(
            painter,
            key,
            order,
            [*p1, *p2, *p3, *p4],
            *color,
            *style,
            scale,
        ),
        DebugDrawCommand::QuadFilled {
            p1,
            p2,
            p3,
            p4,
            color,
        } => paths::paint_quad_filled(painter, key, order, [*p1, *p2, *p3, *p4], *color, scale),
        DebugDrawCommand::Triangle {
            p1,
            p2,
            p3,
            color,
            style,
        } => paths::paint_triangle(painter, key, order, [*p1, *p2, *p3], *color, *style, scale),
        DebugDrawCommand::TriangleFilled { p1, p2, p3, color } => {
            paths::paint_triangle_filled(painter, key, order, [*p1, *p2, *p3], *color, scale)
        }
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
        DebugDrawCommand::Circle {
            center,
            radius,
            color,
            style,
        } => paths::paint_circle(painter, key, order, *center, *radius, *color, *style, scale),
        DebugDrawCommand::CircleFilled {
            center,
            radius,
            color,
        } => paths::paint_circle_filled(painter, key, order, *center, *radius, *color, scale),
        DebugDrawCommand::Ngon {
            center,
            radius,
            segments,
            color,
            style,
        } => paths::paint_ngon(
            painter, key, order, *center, *radius, *segments, *color, *style, scale,
        ),
        DebugDrawCommand::NgonFilled {
            center,
            radius,
            segments,
            color,
        } => paths::paint_ngon_filled(
            painter, key, order, *center, *radius, *segments, *color, scale,
        ),
        DebugDrawCommand::Ellipse {
            center,
            radius,
            rotation_radians,
            segments,
            color,
            style,
        } => paths::paint_ellipse(
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
        ),
        DebugDrawCommand::EllipseFilled {
            center,
            radius,
            rotation_radians,
            segments,
            color,
        } => paths::paint_ellipse_filled(
            painter,
            key,
            order,
            *center,
            *radius,
            *rotation_radians,
            *segments,
            *color,
            scale,
        ),
        DebugDrawCommand::BezierQuadratic {
            from,
            ctrl,
            to,
            color,
            style,
        } => paths::paint_bezier_quadratic(
            painter, key, order, *from, *ctrl, *to, *color, *style, scale,
        ),
        DebugDrawCommand::BezierCubic {
            from,
            ctrl1,
            ctrl2,
            to,
            color,
            style,
        } => paths::paint_bezier_cubic(
            painter, key, order, *from, *ctrl1, *ctrl2, *to, *color, *style, scale,
        ),
        DebugDrawCommand::Text {
            origin,
            text,
            color,
            size,
        } => text::paint_text(painter, order, *origin, text, *color, *size, scale),
        DebugDrawCommand::PushClipRect { .. }
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
