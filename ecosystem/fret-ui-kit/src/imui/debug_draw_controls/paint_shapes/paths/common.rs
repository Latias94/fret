use fret_core::{Color, DrawOrder, PathCommand, PathStyle, Point, Px};
use fret_ui::canvas::CanvasPainter;

pub(super) fn paint_path(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    commands: &[PathCommand],
    style: PathStyle,
    color: Color,
    scale: f32,
) {
    painter.path(
        key,
        order,
        Point::new(Px(0.0), Px(0.0)),
        commands,
        style,
        color,
        scale,
    );
}
