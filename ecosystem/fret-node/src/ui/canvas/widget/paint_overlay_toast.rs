mod layout;
mod style;

use super::*;

pub(super) fn paint_toast<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    toast: &ToastState,
    zoom: f32,
    viewport_origin_x: f32,
    viewport_origin_y: f32,
    viewport_h: f32,
) {
    let text_style = style::toast_text_style(canvas, zoom);
    let layout = layout::toast_layout(zoom, viewport_origin_x, viewport_origin_y, viewport_h);
    let constraints = layout::toast_text_constraints(cx.scale_factor, layout, zoom);

    let (blob, metrics) =
        canvas
            .paint_cache
            .text_blob(cx.services, toast.message.clone(), &text_style, constraints);

    let rect = layout::toast_rect(layout, metrics.size.width.0, metrics.size.height.0);
    let border_color = style::toast_border_color(toast.severity);

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
