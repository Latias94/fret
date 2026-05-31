mod path;

use fret_core::{
    Color, ColorSpace, DrawOrder, FillStyle, GradientStop, MAX_STOPS, Paint, PathStyle, Point, Px,
    StrokeCapV1, StrokeJoinV1, StrokeStyle, StrokeStyleV2, SweepGradient, TileMode,
};
use fret_ui::canvas::{CanvasKey, CanvasPainter};
use fret_ui::element::{AnyElement, CanvasProps};
use fret_ui::{ElementContext, UiHost};

use super::super::super::model::{
    HsvColor, HueWheelGeometry, HueWheelTriangle, hsv_to_color_preserving_alpha,
    hue_wheel_geometry, hue_wheel_rotated_triangle, hue_wheel_sv_cursor_position,
};
use super::super::preview::fill_preview_layout;

use path::{
    HUE_WHEEL_TRIANGLE_STEPS, absolute_point, circle_path, point_from_triangle_barycentric,
    triangle_grid_barycentric, triangle_path,
};

pub(in crate::controls::color_edit::popup) fn hue_wheel_canvas<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    hsv: HsvColor,
) -> AnyElement {
    cx.canvas(
        CanvasProps {
            layout: fill_preview_layout(),
            ..Default::default()
        },
        move |painter| paint_hue_wheel_canvas(painter, hsv),
    )
}

fn paint_hue_wheel_canvas(painter: &mut CanvasPainter<'_>, hsv: HsvColor) {
    let bounds = painter.bounds();
    let geometry = hue_wheel_geometry(bounds.size.width.0, bounds.size.height.0);
    if geometry.wheel_r_outer <= f32::EPSILON || geometry.wheel_thickness <= f32::EPSILON {
        return;
    }

    let scale = painter.scale_factor().max(1.0);
    let origin = bounds.origin;
    let base = painter.key_scope(&"fret-ui-editor.color_edit.hue_wheel");
    paint_hue_wheel_ring(painter, base, origin, geometry, scale);
    paint_hue_wheel_triangle(painter, base, origin, geometry, hsv, scale);
    paint_hue_wheel_cursors(painter, base, origin, geometry, hsv, scale);
}

fn paint_hue_wheel_ring(
    painter: &mut CanvasPainter<'_>,
    base: CanvasKey,
    origin: Point,
    geometry: HueWheelGeometry,
    scale: f32,
) {
    let center = absolute_point(origin, (geometry.center_x, geometry.center_y));
    let radius = (geometry.wheel_r_inner + geometry.wheel_r_outer) * 0.5;
    let path = circle_path(center, radius);
    let mut stops = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
    stops[0] = GradientStop::new(0.0, Color::from_srgb_hex_rgb(0xff_00_00));
    stops[1] = GradientStop::new(1.0 / 6.0, Color::from_srgb_hex_rgb(0xff_ff_00));
    stops[2] = GradientStop::new(2.0 / 6.0, Color::from_srgb_hex_rgb(0x00_ff_00));
    stops[3] = GradientStop::new(3.0 / 6.0, Color::from_srgb_hex_rgb(0x00_ff_ff));
    stops[4] = GradientStop::new(4.0 / 6.0, Color::from_srgb_hex_rgb(0x00_00_ff));
    stops[5] = GradientStop::new(5.0 / 6.0, Color::from_srgb_hex_rgb(0xff_00_ff));
    stops[6] = GradientStop::new(1.0, Color::from_srgb_hex_rgb(0xff_00_00));

    painter.path_paint(
        u64::from(painter.child_key(base, &"ring")),
        DrawOrder(0),
        Point::new(Px(0.0), Px(0.0)),
        &path,
        PathStyle::StrokeV2(StrokeStyleV2 {
            width: Px(geometry.wheel_thickness),
            join: StrokeJoinV1::Round,
            cap: StrokeCapV1::Butt,
            ..Default::default()
        }),
        Paint::SweepGradient(SweepGradient {
            center,
            start_angle_turns: 0.0,
            end_angle_turns: 1.0,
            tile_mode: TileMode::Clamp,
            color_space: ColorSpace::Srgb,
            stop_count: 7,
            stops,
        })
        .into(),
        scale,
    );
}

fn paint_hue_wheel_triangle(
    painter: &mut CanvasPainter<'_>,
    base: CanvasKey,
    origin: Point,
    geometry: HueWheelGeometry,
    hsv: HsvColor,
    scale: f32,
) {
    let triangle = hue_wheel_rotated_triangle(geometry, hsv.hue);
    let mut order = 10u32;
    for i in 0..HUE_WHEEL_TRIANGLE_STEPS {
        for j in 0..(HUE_WHEEL_TRIANGLE_STEPS - i) {
            let p0 = triangle_grid_barycentric(i, j);
            let p1 = triangle_grid_barycentric(i + 1, j);
            let p2 = triangle_grid_barycentric(i, j + 1);
            paint_hue_wheel_triangle_cell(
                painter, base, origin, triangle, hsv.hue, p0, p1, p2, order, scale,
            );
            order += 1;

            if j < HUE_WHEEL_TRIANGLE_STEPS - i - 1 {
                let p3 = triangle_grid_barycentric(i + 1, j + 1);
                paint_hue_wheel_triangle_cell(
                    painter, base, origin, triangle, hsv.hue, p1, p3, p2, order, scale,
                );
                order += 1;
            }
        }
    }

    let border_path = triangle_path(
        absolute_point(origin, triangle.hue),
        absolute_point(origin, triangle.black),
        absolute_point(origin, triangle.white),
    );
    painter.path(
        u64::from(painter.child_key(base, &"triangle.border")),
        DrawOrder(order),
        Point::new(Px(0.0), Px(0.0)),
        &border_path,
        PathStyle::Stroke(StrokeStyle { width: Px(1.5) }),
        Color::from_srgb_hex_rgb(0x80_80_80),
        scale,
    );
}

#[allow(clippy::too_many_arguments)]
fn paint_hue_wheel_triangle_cell(
    painter: &mut CanvasPainter<'_>,
    base: CanvasKey,
    origin: Point,
    triangle: HueWheelTriangle,
    hue: f32,
    a: (f32, f32, f32),
    b: (f32, f32, f32),
    c: (f32, f32, f32),
    order: u32,
    scale: f32,
) {
    let centroid = (
        (a.0 + b.0 + c.0) / 3.0,
        (a.1 + b.1 + c.1) / 3.0,
        (a.2 + b.2 + c.2) / 3.0,
    );
    let value = (1.0 - centroid.1).clamp(0.0, 1.0);
    let saturation = if value <= f32::EPSILON {
        0.0
    } else {
        (centroid.0 / value).clamp(0.0, 1.0)
    };
    let color = hsv_to_color_preserving_alpha(
        HsvColor {
            hue,
            saturation,
            value,
        },
        1.0,
    );
    let path = triangle_path(
        absolute_point(origin, point_from_triangle_barycentric(triangle, a)),
        absolute_point(origin, point_from_triangle_barycentric(triangle, b)),
        absolute_point(origin, point_from_triangle_barycentric(triangle, c)),
    );
    painter.path(
        u64::from(painter.child_key(base, &("triangle.cell", order))),
        DrawOrder(order),
        Point::new(Px(0.0), Px(0.0)),
        &path,
        PathStyle::Fill(FillStyle::default()),
        color,
        scale,
    );
}

fn paint_hue_wheel_cursors(
    painter: &mut CanvasPainter<'_>,
    base: CanvasKey,
    origin: Point,
    geometry: HueWheelGeometry,
    hsv: HsvColor,
    scale: f32,
) {
    let hue_angle = hsv.hue.rem_euclid(1.0) * std::f32::consts::PI * 2.0;
    let hue_radius = (geometry.wheel_r_inner + geometry.wheel_r_outer) * 0.5;
    let hue_cursor = absolute_point(
        origin,
        (
            geometry.center_x + hue_angle.cos() * hue_radius,
            geometry.center_y + hue_angle.sin() * hue_radius,
        ),
    );
    let hue_color = hsv_to_color_preserving_alpha(
        HsvColor {
            hue: hsv.hue,
            saturation: 1.0,
            value: 1.0,
        },
        1.0,
    );
    paint_cursor_circle(
        painter,
        base,
        "hue.cursor",
        DrawOrder(320),
        hue_cursor,
        geometry.wheel_thickness * 0.55,
        hue_color,
        scale,
    );

    let sv_cursor = absolute_point(
        origin,
        hue_wheel_sv_cursor_position(hsv, geometry.center_x * 2.0, geometry.center_y * 2.0),
    );
    paint_cursor_circle(
        painter,
        base,
        "sv.cursor",
        DrawOrder(324),
        sv_cursor,
        geometry.wheel_thickness * 0.40,
        hsv_to_color_preserving_alpha(hsv, 1.0),
        scale,
    );
}

fn paint_cursor_circle(
    painter: &mut CanvasPainter<'_>,
    base: CanvasKey,
    key: &'static str,
    order: DrawOrder,
    center: Point,
    radius: f32,
    color: Color,
    scale: f32,
) {
    let fill = circle_path(center, radius.max(1.0));
    painter.path(
        u64::from(painter.child_key(base, &(key, "fill"))),
        order,
        Point::new(Px(0.0), Px(0.0)),
        &fill,
        PathStyle::Fill(FillStyle::default()),
        color,
        scale,
    );
    let outer = circle_path(center, (radius + 1.0).max(1.0));
    painter.path(
        u64::from(painter.child_key(base, &(key, "outer"))),
        DrawOrder(order.0 + 1),
        Point::new(Px(0.0), Px(0.0)),
        &outer,
        PathStyle::Stroke(StrokeStyle { width: Px(1.0) }),
        Color::from_srgb_hex_rgb(0x80_80_80),
        scale,
    );
    let inner = circle_path(center, radius.max(1.0));
    painter.path(
        u64::from(painter.child_key(base, &(key, "inner"))),
        DrawOrder(order.0 + 2),
        Point::new(Px(0.0), Px(0.0)),
        &inner,
        PathStyle::Stroke(StrokeStyle { width: Px(1.0) }),
        Color::from_srgb_hex_rgb(0xff_ff_ff),
        scale,
    );
}
