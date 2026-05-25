use std::sync::Arc;

use fret_core::{Color, DrawOrder, Point, Px, TextOverflow, TextStyle, TextWrap};
use fret_ui::canvas::{CanvasPainter, CanvasTextConstraints};

pub(super) fn paint_text(
    painter: &mut CanvasPainter<'_>,
    order: DrawOrder,
    origin: Point,
    text: &Arc<str>,
    color: Color,
    size: Px,
    scale: f32,
) {
    if color.a <= 0.0 || size.0 <= 0.0 {
        return;
    }
    painter.shared_text(
        order,
        origin,
        text.clone(),
        TextStyle {
            size,
            line_height: Some(Px(size.0 * 1.2)),
            ..Default::default()
        },
        color,
        CanvasTextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        },
        scale,
    );
}
