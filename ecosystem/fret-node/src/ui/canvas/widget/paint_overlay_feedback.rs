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
    super::paint_overlay_toast::paint_toast(
        canvas,
        cx,
        toast,
        zoom,
        viewport_origin_x,
        viewport_origin_y,
        viewport_h,
    );
}

pub(super) fn paint_wire_drag_hint<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    _snapshot: &ViewSnapshot,
    wire_drag: &WireDrag,
    zoom: f32,
) {
    super::paint_overlay_wire_hint::paint_wire_drag_hint(canvas, cx, wire_drag, zoom);
}
