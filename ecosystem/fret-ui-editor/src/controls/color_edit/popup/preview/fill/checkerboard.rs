use fret_core::{Color, DrawOrder, Edges, Paint, Point, Px, Rect, SceneOp, Size};
use fret_ui::canvas::CanvasPainter;
use fret_ui::element::{AnyElement, CanvasProps};
use fret_ui::{ElementContext, UiHost};

use super::super::super::super::{CHECKERBOARD_DARK_RGB, CHECKERBOARD_LIGHT_RGB};
use super::fill_preview_layout;

pub(in crate::controls::color_edit::popup) fn checkerboard_grid<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> AnyElement {
    cx.canvas(
        CanvasProps {
            layout: fill_preview_layout(),
            ..Default::default()
        },
        move |painter| {
            paint_checkerboard_grid(painter);
        },
    )
}

fn paint_checkerboard_grid(painter: &mut CanvasPainter<'_>) {
    let bounds = painter.bounds();
    let half_width = bounds.size.width / 2.0;
    let half_height = bounds.size.height / 2.0;
    let x0 = bounds.origin.x;
    let y0 = bounds.origin.y;
    let x1 = x0 + half_width;
    let y1 = y0 + half_height;

    for (row, y) in [(0usize, y0), (1usize, y1)] {
        for (col, x) in [(0usize, x0), (1usize, x1)] {
            let cell = Rect::new(Point::new(x, y), Size::new(half_width, half_height));
            push_solid_rect(
                painter,
                cell,
                checkerboard_cell_color(row, col),
                DrawOrder(0),
            );
        }
    }
}

fn push_solid_rect(painter: &mut CanvasPainter<'_>, rect: Rect, color: Color, order: DrawOrder) {
    painter.scene().push(SceneOp::Quad {
        order,
        rect,
        background: Paint::Solid(color).into(),
        border: Edges::all(Px(0.0)),
        border_paint: Paint::Solid(Color::TRANSPARENT).into(),
        corner_radii: Default::default(),
    });
}

pub(in crate::controls::color_edit) fn checkerboard_cell_color(row: usize, col: usize) -> Color {
    let rgb = if (row + col).is_multiple_of(2) {
        CHECKERBOARD_LIGHT_RGB
    } else {
        CHECKERBOARD_DARK_RGB
    };
    Color::from_srgb_hex_rgb(rgb)
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Rect};
    use fret_ui::element::ElementKind;

    #[test]
    fn checkerboard_grid_returns_canvas_root_directly() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let element = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            Rect::default(),
            "checkerboard-grid",
            |cx| checkerboard_grid(cx),
        );

        assert!(matches!(element.kind, ElementKind::Canvas(_)));
        assert!(element.children.is_empty());
    }
}
