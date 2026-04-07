use fret_core::{
    Color, Corners, DrawOrder, Edges, Point, Px, Rect, SceneOp, Size, TextBlobId, TextConstraints,
    TextStyle,
};
use fret_ui::{UiHost, retained_bridge::PaintCx};

pub(super) fn centered_text_origin(rect: Rect, text_size: Size) -> Point {
    Point::new(
        Px(rect.origin.x.0 + 0.5 * (rect.size.width.0 - text_size.width.0)),
        Px(rect.origin.y.0 + 0.5 * (rect.size.height.0 - text_size.height.0)),
    )
}

pub(super) fn leading_text_origin(rect: Rect, text_size: Size, padding_px: f32) -> Point {
    Point::new(
        Px(rect.origin.x.0 + padding_px),
        Px(rect.origin.y.0 + 0.5 * (rect.size.height.0 - text_size.height.0)),
    )
}

pub(super) fn paint_panel_button<H: UiHost>(
    cx: &mut PaintCx<'_, H>,
    text_blobs: &mut Vec<TextBlobId>,
    rect: Rect,
    label: &str,
    text_style: &TextStyle,
    constraints: TextConstraints,
    background: Color,
    text_color: Color,
    corner_px: f32,
    background_order: DrawOrder,
    text_order: DrawOrder,
) {
    cx.scene.push(SceneOp::Quad {
        order: background_order,
        rect,
        background: fret_core::Paint::Solid(background).into(),

        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT.into(),

        corner_radii: Corners::all(Px(corner_px.max(4.0))),
    });

    let (id, metrics) = cx
        .services
        .text()
        .prepare_str(label, text_style, constraints);
    text_blobs.push(id);
    cx.scene.push(SceneOp::Text {
        order: text_order,
        text: id,
        origin: centered_text_origin(rect, metrics.size),
        paint: text_color.into(),
        outline: None,
        shadow: None,
    });
}

pub(super) fn paint_panel_label<H: UiHost>(
    cx: &mut PaintCx<'_, H>,
    text_blobs: &mut Vec<TextBlobId>,
    rect: Rect,
    label: &str,
    text_style: &TextStyle,
    constraints: TextConstraints,
    text_color: Color,
    padding_px: f32,
    text_order: DrawOrder,
) {
    let (id, metrics) = cx
        .services
        .text()
        .prepare_str(label, text_style, constraints);
    text_blobs.push(id);
    cx.scene.push(SceneOp::Text {
        order: text_order,
        text: id,
        origin: leading_text_origin(rect, metrics.size, padding_px),
        paint: text_color.into(),
        outline: None,
        shadow: None,
    });
}

#[cfg(test)]
mod tests {
    use super::{centered_text_origin, leading_text_origin};
    use fret_core::{Point, Px, Rect, Size};

    #[test]
    fn centered_text_origin_centers_within_button_rect() {
        let rect = Rect::new(
            Point::new(Px(100.0), Px(50.0)),
            Size::new(Px(40.0), Px(20.0)),
        );
        let origin = centered_text_origin(rect, Size::new(Px(10.0), Px(8.0)));
        assert_eq!(origin, Point::new(Px(115.0), Px(56.0)));
    }

    #[test]
    fn leading_text_origin_keeps_padding_and_vertical_centering() {
        let rect = Rect::new(
            Point::new(Px(20.0), Px(30.0)),
            Size::new(Px(100.0), Px(24.0)),
        );
        let origin = leading_text_origin(rect, Size::new(Px(40.0), Px(10.0)), 6.0);
        assert_eq!(origin, Point::new(Px(26.0), Px(37.0)));
    }
}
