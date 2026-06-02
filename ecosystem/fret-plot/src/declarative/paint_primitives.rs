//! Declarative line-plot shared paint primitive owner.

use fret_core::{Color, Corners, DrawOrder, Edges, Paint, Point, Px, Rect, Size};
use fret_ui::canvas::CanvasPainter;

pub(super) fn push_vertical_line(
    painter: &mut CanvasPainter<'_>,
    x: Px,
    y: Px,
    height: Px,
    order: DrawOrder,
    color: Color,
) {
    if !x.0.is_finite() || !y.0.is_finite() || !height.0.is_finite() || height.0 <= 0.0 {
        return;
    }
    painter.scene().push(fret_core::SceneOp::Quad {
        order,
        rect: Rect::new(Point::new(x, y), Size::new(Px(1.0), height)),
        background: Paint::Solid(color).into(),
        border: Edges::default(),
        border_paint: Paint::Solid(Color::TRANSPARENT).into(),
        corner_radii: Corners::default(),
    });
}

pub(super) fn push_horizontal_line(
    painter: &mut CanvasPainter<'_>,
    x: Px,
    y: Px,
    width: Px,
    order: DrawOrder,
    color: Color,
) {
    if !x.0.is_finite() || !y.0.is_finite() || !width.0.is_finite() || width.0 <= 0.0 {
        return;
    }
    painter.scene().push(fret_core::SceneOp::Quad {
        order,
        rect: Rect::new(Point::new(x, y), Size::new(width, Px(1.0))),
        background: Paint::Solid(color).into(),
        border: Edges::default(),
        border_paint: Paint::Solid(Color::TRANSPARENT).into(),
        corner_radii: Corners::default(),
    });
}

pub(super) fn push_filled_rect(
    painter: &mut CanvasPainter<'_>,
    rect: Rect,
    order: DrawOrder,
    color: Color,
) {
    if !rect.origin.x.0.is_finite()
        || !rect.origin.y.0.is_finite()
        || !rect.size.width.0.is_finite()
        || !rect.size.height.0.is_finite()
        || rect.size.width.0 <= 0.0
        || rect.size.height.0 <= 0.0
    {
        return;
    }
    painter.scene().push(fret_core::SceneOp::Quad {
        order,
        rect,
        background: Paint::Solid(color).into(),
        border: Edges::default(),
        border_paint: Paint::Solid(Color::TRANSPARENT).into(),
        corner_radii: Corners::default(),
    });
}
