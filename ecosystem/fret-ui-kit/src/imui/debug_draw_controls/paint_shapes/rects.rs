use fret_core::scene::Paint;
use fret_core::{Color, Corners, DrawOrder, Edges, Px, Rect};
use fret_ui::canvas::CanvasPainter;

use super::super::geometry::{rect_is_empty, rect_is_finite, rect_quad_points};

pub(super) fn paint_rect_filled(
    painter: &mut CanvasPainter<'_>,
    order: DrawOrder,
    rect: Rect,
    color: Color,
) {
    if color.a <= 0.0 || rect_is_empty(rect) {
        return;
    }
    painter.scene().push(fret_core::SceneOp::Quad {
        order,
        rect,
        background: Paint::Solid(color).into(),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::Solid(Color::TRANSPARENT).into(),
        corner_radii: Corners::all(Px(0.0)),
    });
}

pub(super) fn paint_rect_filled_multi_color(
    painter: &mut CanvasPainter<'_>,
    order: DrawOrder,
    rect: Rect,
    colors: [Color; 4],
) {
    if rect_is_empty(rect) || !rect_is_finite(rect) || colors.iter().all(|color| color.a <= 0.0) {
        return;
    }
    painter.scene().push(fret_core::SceneOp::VertexColorQuad {
        order,
        points: rect_quad_points(rect),
        colors,
    });
}
