use fret_core::{Color, DrawOrder, FillStyle, PathStyle, Point, Px, StrokeStyle};
use fret_ui::canvas::{CanvasKey, CanvasPainter};

use super::super::super::super::model::{
    HsvColor, HueWheelGeometry, hsv_to_color_preserving_alpha, hue_wheel_sv_cursor_position,
};
use super::path::{absolute_point, circle_path};

pub(super) fn paint_hue_wheel_cursors(
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
