use fret_core::scene::Paint;
use fret_core::{
    Color, Corners, DrawOrder, Edges, FillStyle, PathCommand, PathStyle, Point, Px, TextOverflow,
    TextStyle, TextWrap,
};
use fret_ui::canvas::{CanvasPainter, CanvasTextConstraints};

use super::{
    DebugDrawCommand, bezier_cubic_path, bezier_quadratic_path, circle_path,
    concave_poly_fill_path, convex_poly_fill_path, corner_radii_are_visible, ellipse_path,
    ngon_path, normalized_opacity, paint_image, paint_image_region, paint_image_triangle_mesh,
    paint_triangle_mesh, points_are_finite, polyline_path, quad_path, rect_is_empty,
    rect_is_finite, rect_path, rect_quad_points, rounded_rect_corner_radii, triangle_is_degenerate,
    triangle_path, uv_points_are_finite, uv_rect_is_valid,
};

pub(super) fn paint_debug_draw_commands(
    painter: &mut CanvasPainter<'_>,
    commands: &[DebugDrawCommand],
) {
    let scale = painter.scale_factor().max(1.0);
    let mut open_clip_depth = 0usize;
    for (index, command) in commands.iter().enumerate() {
        let order = DrawOrder(index as u32);
        let key = painter.key(&("fret-ui-kit.imui.debug_draw.command", index));
        match command {
            DebugDrawCommand::PushClipRect { rect } => {
                if rect_is_empty(*rect) {
                    continue;
                }
                painter
                    .scene()
                    .push(fret_core::SceneOp::PushClipRect { rect: *rect });
                open_clip_depth += 1;
            }
            DebugDrawCommand::PopClipRect => {
                if open_clip_depth == 0 {
                    continue;
                }
                painter.scene().push(fret_core::SceneOp::PopClip);
                open_clip_depth -= 1;
            }
            DebugDrawCommand::Image {
                rect,
                image,
                options,
            } => {
                let opacity = normalized_opacity(options.opacity);
                if opacity <= 0.0 || rect_is_empty(*rect) {
                    continue;
                }
                paint_image(painter, order, *rect, *image, *options, opacity);
            }
            DebugDrawCommand::ImageRegion {
                rect,
                image,
                uv,
                options,
            } => {
                let opacity = normalized_opacity(options.opacity);
                if opacity <= 0.0 || rect_is_empty(*rect) || !uv_rect_is_valid(*uv) {
                    continue;
                }
                paint_image_region(painter, order, *rect, *image, *uv, *options, opacity);
            }
            DebugDrawCommand::ImageQuad {
                image,
                points,
                uvs,
                options,
            } => {
                let opacity = normalized_opacity(options.opacity);
                if opacity <= 0.0
                    || options.tint.a <= 0.0
                    || !points_are_finite(points)
                    || !uv_points_are_finite(uvs)
                {
                    continue;
                }
                painter.scene().push(fret_core::SceneOp::ImageQuad {
                    order,
                    points: *points,
                    image: *image,
                    uvs: *uvs,
                    sampling: options.sampling,
                    tint: options.tint,
                    opacity,
                });
            }
            DebugDrawCommand::ImageRounded {
                rect,
                image,
                options,
                rounding,
                corners,
            } => {
                let opacity = normalized_opacity(options.opacity);
                if opacity <= 0.0 || rect_is_empty(*rect) || !rect_is_finite(*rect) {
                    continue;
                }
                let corner_radii = rounded_rect_corner_radii(*rect, *rounding, *corners);
                if corner_radii_are_visible(corner_radii) {
                    painter.scene().push(fret_core::SceneOp::PushClipRRect {
                        rect: *rect,
                        corner_radii,
                    });
                    paint_image(painter, order, *rect, *image, *options, opacity);
                    painter.scene().push(fret_core::SceneOp::PopClip);
                } else {
                    paint_image(painter, order, *rect, *image, *options, opacity);
                }
            }
            DebugDrawCommand::ImageRegionRounded {
                rect,
                image,
                uv,
                options,
                rounding,
                corners,
            } => {
                let opacity = normalized_opacity(options.opacity);
                if opacity <= 0.0
                    || rect_is_empty(*rect)
                    || !rect_is_finite(*rect)
                    || !uv_rect_is_valid(*uv)
                {
                    continue;
                }
                let corner_radii = rounded_rect_corner_radii(*rect, *rounding, *corners);
                if corner_radii_are_visible(corner_radii) {
                    painter.scene().push(fret_core::SceneOp::PushClipRRect {
                        rect: *rect,
                        corner_radii,
                    });
                    paint_image_region(painter, order, *rect, *image, *uv, *options, opacity);
                    painter.scene().push(fret_core::SceneOp::PopClip);
                } else {
                    paint_image_region(painter, order, *rect, *image, *uv, *options, opacity);
                }
            }
            DebugDrawCommand::SvgImage { rect, svg, options } => {
                let opacity = normalized_opacity(options.opacity);
                if opacity <= 0.0 || rect_is_empty(*rect) {
                    continue;
                }
                painter.svg_image(key, order, *rect, svg, options.fit, opacity);
            }
            DebugDrawCommand::SvgMaskIcon {
                rect,
                svg,
                color,
                options,
            } => {
                let opacity = normalized_opacity(options.opacity);
                if opacity <= 0.0 || color.a <= 0.0 || rect_is_empty(*rect) {
                    continue;
                }
                painter.svg_mask_icon(key, order, *rect, svg, options.fit, *color, opacity);
            }
            DebugDrawCommand::Line {
                from,
                to,
                color,
                style,
            } => {
                if color.a <= 0.0 || !style.is_visible() {
                    continue;
                }
                let commands = [PathCommand::MoveTo(*from), PathCommand::LineTo(*to)];
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style.path_style(),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::Polyline {
                points,
                color,
                style,
                closed,
            } => {
                if color.a <= 0.0 || !style.is_visible() {
                    continue;
                }
                let Some(commands) = polyline_path(points, *closed) else {
                    continue;
                };
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style.path_style(),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::ConvexPolyFilled { points, color } => {
                if color.a <= 0.0 {
                    continue;
                }
                let Some(commands) = convex_poly_fill_path(points) else {
                    continue;
                };
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    PathStyle::Fill(FillStyle::default()),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::ConcavePolyFilled { points, color } => {
                if color.a <= 0.0 {
                    continue;
                }
                let Some(commands) = concave_poly_fill_path(points) else {
                    continue;
                };
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    PathStyle::Fill(FillStyle::default()),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::Rect { rect, color, style } => {
                if color.a <= 0.0 || !style.is_visible() || rect_is_empty(*rect) {
                    continue;
                }
                let commands = rect_path(*rect);
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style.path_style(),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::RectFilled { rect, color } => {
                if color.a <= 0.0 || rect_is_empty(*rect) {
                    continue;
                }
                painter.scene().push(fret_core::SceneOp::Quad {
                    order,
                    rect: *rect,
                    background: Paint::Solid(*color).into(),
                    border: Edges::all(Px(0.0)),
                    border_paint: Paint::Solid(Color::TRANSPARENT).into(),
                    corner_radii: Corners::all(Px(0.0)),
                });
            }
            DebugDrawCommand::RectFilledMultiColor {
                rect,
                upper_left,
                upper_right,
                bottom_right,
                bottom_left,
            } => {
                let colors = [*upper_left, *upper_right, *bottom_right, *bottom_left];
                if rect_is_empty(*rect)
                    || !rect_is_finite(*rect)
                    || colors.iter().all(|color| color.a <= 0.0)
                {
                    continue;
                }
                painter.scene().push(fret_core::SceneOp::VertexColorQuad {
                    order,
                    points: rect_quad_points(*rect),
                    colors,
                });
            }
            DebugDrawCommand::Quad {
                p1,
                p2,
                p3,
                p4,
                color,
                style,
            } => {
                if color.a <= 0.0 || !style.is_visible() {
                    continue;
                }
                let commands = quad_path(*p1, *p2, *p3, *p4);
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style.path_style(),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::QuadFilled {
                p1,
                p2,
                p3,
                p4,
                color,
            } => {
                if color.a <= 0.0 {
                    continue;
                }
                let commands = quad_path(*p1, *p2, *p3, *p4);
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    PathStyle::Fill(FillStyle::default()),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::Triangle {
                p1,
                p2,
                p3,
                color,
                style,
            } => {
                if color.a <= 0.0 || !style.is_visible() || triangle_is_degenerate(*p1, *p2, *p3) {
                    continue;
                }
                let commands = triangle_path(*p1, *p2, *p3);
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style.path_style(),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::TriangleFilled { p1, p2, p3, color } => {
                if color.a <= 0.0 || triangle_is_degenerate(*p1, *p2, *p3) {
                    continue;
                }
                let commands = triangle_path(*p1, *p2, *p3);
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    PathStyle::Fill(FillStyle::default()),
                    *color,
                    scale,
                );
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
            } => {
                if color.a <= 0.0 || !style.is_visible() || radius.0 <= 0.0 {
                    continue;
                }
                let commands = circle_path(*center, *radius);
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style.path_style(),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::CircleFilled {
                center,
                radius,
                color,
            } => {
                if color.a <= 0.0 || radius.0 <= 0.0 {
                    continue;
                }
                let commands = circle_path(*center, *radius);
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    PathStyle::Fill(FillStyle::default()),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::Ngon {
                center,
                radius,
                segments,
                color,
                style,
            } => {
                if color.a <= 0.0 || !style.is_visible() {
                    continue;
                }
                let Some(commands) = ngon_path(*center, *radius, *segments) else {
                    continue;
                };
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style.path_style(),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::NgonFilled {
                center,
                radius,
                segments,
                color,
            } => {
                if color.a <= 0.0 {
                    continue;
                }
                let Some(commands) = ngon_path(*center, *radius, *segments) else {
                    continue;
                };
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    PathStyle::Fill(FillStyle::default()),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::Ellipse {
                center,
                radius,
                rotation_radians,
                segments,
                color,
                style,
            } => {
                if color.a <= 0.0 || !style.is_visible() {
                    continue;
                }
                let Some(commands) = ellipse_path(*center, *radius, *rotation_radians, *segments)
                else {
                    continue;
                };
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style.path_style(),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::EllipseFilled {
                center,
                radius,
                rotation_radians,
                segments,
                color,
            } => {
                if color.a <= 0.0 {
                    continue;
                }
                let Some(commands) = ellipse_path(*center, *radius, *rotation_radians, *segments)
                else {
                    continue;
                };
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    PathStyle::Fill(FillStyle::default()),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::BezierQuadratic {
                from,
                ctrl,
                to,
                color,
                style,
            } => {
                if color.a <= 0.0 || !style.is_visible() {
                    continue;
                }
                let commands = bezier_quadratic_path(*from, *ctrl, *to);
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style.path_style(),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::BezierCubic {
                from,
                ctrl1,
                ctrl2,
                to,
                color,
                style,
            } => {
                if color.a <= 0.0 || !style.is_visible() {
                    continue;
                }
                let commands = bezier_cubic_path(*from, *ctrl1, *ctrl2, *to);
                painter.path(
                    key,
                    order,
                    Point::new(Px(0.0), Px(0.0)),
                    &commands,
                    style.path_style(),
                    *color,
                    scale,
                );
            }
            DebugDrawCommand::Text {
                origin,
                text,
                color,
                size,
            } => {
                if color.a <= 0.0 || size.0 <= 0.0 {
                    continue;
                }
                painter.shared_text(
                    order,
                    *origin,
                    text.clone(),
                    TextStyle {
                        size: *size,
                        line_height: Some(Px(size.0 * 1.2)),
                        ..Default::default()
                    },
                    *color,
                    CanvasTextConstraints {
                        max_width: None,
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                    },
                    scale,
                );
            }
        }
    }

    for _ in 0..open_clip_depth {
        painter.scene().push(fret_core::SceneOp::PopClip);
    }
}
