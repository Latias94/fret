use fret_core::{Color, Corners, DrawOrder, Edges, Paint, Point, Px, Rect, SceneOp, Size};
use fret_ui::canvas::CanvasPainter;
use fret_ui::element::{
    AnyElement, CanvasProps, ContainerProps, LayoutStyle, Length, Overflow, SizeStyle,
};
use fret_ui::{ElementContext, UiHost};

use super::super::super::ColorEditAlphaPreview;

pub(in crate::controls::color_edit) mod checkerboard;

pub(in crate::controls::color_edit::popup) use checkerboard::checkerboard_grid;

pub(in crate::controls::color_edit) fn color_preview_stack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    color: Color,
    radius: Px,
    alpha_preview: ColorEditAlphaPreview,
) -> AnyElement {
    match alpha_preview {
        ColorEditAlphaPreview::Checkerboard => checkerboard_preview_fill(cx, color, radius),
        ColorEditAlphaPreview::Opaque => {
            solid_preview_fill(cx, opaque_preview_color(color), radius)
        }
        ColorEditAlphaPreview::NoBackground => solid_preview_fill(cx, color, radius),
        ColorEditAlphaPreview::Half => half_alpha_preview_fill(cx, color, radius),
    }
}

fn checkerboard_preview_fill<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    color: Color,
    radius: Px,
) -> AnyElement {
    cx.canvas(
        CanvasProps {
            layout: fill_preview_layout(),
            ..Default::default()
        },
        move |p| {
            paint_checkerboard_preview(p, color, radius);
        },
    )
}

fn solid_preview_fill<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    color: Color,
    radius: Px,
) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: fill_preview_layout(),
            background: Some(color),
            corner_radii: Corners::all(radius),
            ..Default::default()
        },
        |_cx| vec![],
    )
}

fn half_alpha_preview_fill<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    color: Color,
    radius: Px,
) -> AnyElement {
    cx.canvas(
        CanvasProps {
            layout: fill_preview_layout(),
            ..Default::default()
        },
        move |p| {
            paint_half_alpha_preview(p, color, radius);
        },
    )
}

pub(in crate::controls::color_edit::popup) fn fill_preview_layout() -> LayoutStyle {
    LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Fill,
            ..Default::default()
        },
        overflow: Overflow::Clip,
        ..Default::default()
    }
}

pub(in crate::controls::color_edit) fn opaque_preview_color(mut color: Color) -> Color {
    color.a = 1.0;
    color
}

fn paint_checkerboard_preview(painter: &mut CanvasPainter<'_>, color: Color, radius: Px) {
    let bounds = painter.bounds();
    painter.with_clip_rrect(bounds, Corners::all(radius), |painter| {
        paint_checkerboard_cells(painter, bounds);
        push_solid_rect(painter, bounds, color, DrawOrder(1));
    });
}

fn paint_half_alpha_preview(painter: &mut CanvasPainter<'_>, color: Color, radius: Px) {
    let bounds = painter.bounds();
    painter.with_clip_rrect(bounds, Corners::all(radius), |painter| {
        let half_width = bounds.size.width / 2.0;
        let left = Rect::new(bounds.origin, Size::new(half_width, bounds.size.height));
        let right = Rect::new(
            Point::new(bounds.origin.x + half_width, bounds.origin.y),
            Size::new(half_width, bounds.size.height),
        );

        push_solid_rect(painter, left, opaque_preview_color(color), DrawOrder(0));
        paint_checkerboard_cells(painter, right);
        push_solid_rect(painter, right, color, DrawOrder(1));
    });
}

fn paint_checkerboard_cells(painter: &mut CanvasPainter<'_>, bounds: Rect) {
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
                checkerboard::checkerboard_cell_color(row, col),
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
        corner_radii: Corners::default(),
    });
}

pub(in crate::controls::color_edit) fn preview_color_for_alpha_visibility(
    color: Color,
    show_alpha: bool,
) -> Color {
    if show_alpha {
        color
    } else {
        opaque_preview_color(color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Color, Px, Rect};
    use fret_ui::element::ElementKind;

    #[test]
    fn checkerboard_preview_returns_canvas_root_directly() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let element = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            Rect::default(),
            "color-preview-checkerboard",
            |cx| {
                color_preview_stack(
                    cx,
                    Color::from_srgb_hex_rgb(0x33_66_99),
                    Px(5.0),
                    ColorEditAlphaPreview::Checkerboard,
                )
            },
        );

        assert!(matches!(element.kind, ElementKind::Canvas(_)));
        assert!(element.children.is_empty());
    }

    #[test]
    fn half_alpha_preview_returns_canvas_root_directly() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let element = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            Rect::default(),
            "color-preview-half",
            |cx| {
                color_preview_stack(
                    cx,
                    Color::from_srgb_hex_rgb(0x33_66_99),
                    Px(5.0),
                    ColorEditAlphaPreview::Half,
                )
            },
        );

        assert!(matches!(element.kind, ElementKind::Canvas(_)));
        assert!(element.children.is_empty());
    }
}
