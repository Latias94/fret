use super::super::*;
use super::layout::ToastLayout;

pub(in super::super) fn paint_toast_content<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    toast: &ToastState,
    zoom: f32,
    text_style: &fret_core::TextStyle,
    layout: ToastLayout,
    constraints: TextConstraints,
    border_color: Color,
) {
    let (blob, metrics) =
        canvas
            .paint_cache
            .text_blob(cx.services, toast.message.clone(), text_style, constraints);

    let rect = super::layout::toast_rect(layout, metrics.size.width.0, metrics.size.height.0);

    cx.scene.push(SceneOp::Quad {
        order: DrawOrder(70),
        rect,
        background: fret_core::Paint::Solid(canvas.style.paint.context_menu_background).into(),
        border: Edges::all(Px(1.0 / zoom)),
        border_paint: fret_core::Paint::Solid(border_color).into(),
        corner_radii: Corners::all(Px(6.0 / zoom)),
    });

    let text_x = Px(rect.origin.x.0 + layout.pad);
    let text_y = Px(rect.origin.y.0 + layout.pad + metrics.baseline.0);
    cx.scene.push(SceneOp::Text {
        order: DrawOrder(71),
        origin: Point::new(text_x, text_y),
        text: blob,
        paint: canvas.style.paint.context_menu_text.into(),
        outline: None,
        shadow: None,
    });
}
