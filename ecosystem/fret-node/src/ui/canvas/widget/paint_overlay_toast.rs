mod draw;
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
    let toast_layout = layout::toast_layout(zoom, viewport_origin_x, viewport_origin_y, viewport_h);
    let constraints = layout::toast_text_constraints(cx.scale_factor, toast_layout, zoom);
    let border_color = style::toast_border_color(toast.severity);
    draw::paint_toast_content(
        canvas,
        cx,
        toast,
        zoom,
        &text_style,
        toast_layout,
        constraints,
        border_color,
    );
}
