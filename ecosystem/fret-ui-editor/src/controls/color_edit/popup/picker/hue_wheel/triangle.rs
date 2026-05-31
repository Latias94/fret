use fret_core::{Color, DrawOrder, FillStyle, PathStyle, Point, Px, StrokeStyle};
use fret_ui::canvas::{CanvasKey, CanvasPainter};

use super::super::super::super::model::{
    HsvColor, HueWheelGeometry, HueWheelTriangle, hsv_to_color_preserving_alpha,
    hue_wheel_rotated_triangle,
};
use super::path::{
    HUE_WHEEL_TRIANGLE_STEPS, absolute_point, point_from_triangle_barycentric,
    triangle_grid_barycentric, triangle_path,
};

pub(super) fn paint_hue_wheel_triangle(
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
