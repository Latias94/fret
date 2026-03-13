use super::WireHintPaintLayout;
use super::*;
use std::sync::Arc;

pub(super) fn paint_wire_drag_hint_content<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    wire_drag: &WireDrag,
    zoom: f32,
    text: Arc<str>,
    border_color: Color,
    layout: &WireHintPaintLayout,
) {
    let (blob, metrics) =
        canvas
            .paint_cache
            .text_blob(cx.services, text, &layout.text_style, layout.constraints);

    let box_w = (metrics.size.width.0 + 2.0 * layout.pad).clamp(72.0 / zoom, layout.max_w);
    let box_h = metrics.size.height.0 + 2.0 * layout.pad;
    let rect = Rect::new(
        Point::new(
            Px(wire_drag.pos.x.0 + layout.offset_x),
            Px(wire_drag.pos.y.0 + layout.offset_y),
        ),
        Size::new(Px(box_w), Px(box_h)),
    );

    cx.scene.push(SceneOp::Quad {
        order: DrawOrder(69),
        rect,
        background: fret_core::Paint::Solid(canvas.style.paint.context_menu_background).into(),
        border: Edges::all(Px(1.0 / zoom)),
        border_paint: fret_core::Paint::Solid(border_color).into(),
        corner_radii: Corners::all(Px(6.0 / zoom)),
    });

    let text_x = Px(rect.origin.x.0 + layout.pad);
    let text_y = Px(rect.origin.y.0 + layout.pad + metrics.baseline.0);
    cx.scene.push(SceneOp::Text {
        order: DrawOrder(70),
        origin: Point::new(text_x, text_y),
        text: blob,
        paint: canvas.style.paint.context_menu_text.into(),
        outline: None,
        shadow: None,
    });
}
