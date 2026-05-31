use fret_core::{
    Color, ColorSpace, DrawOrder, GradientStop, MAX_STOPS, Paint, PathStyle, Point, Px,
    StrokeCapV1, StrokeJoinV1, StrokeStyleV2, SweepGradient, TileMode,
};
use fret_ui::canvas::{CanvasKey, CanvasPainter};

use super::super::super::super::model::HueWheelGeometry;
use super::path::{absolute_point, circle_path};

pub(super) fn paint_hue_wheel_ring(
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
