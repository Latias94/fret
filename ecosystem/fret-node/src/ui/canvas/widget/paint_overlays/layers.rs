use crate::ui::canvas::widget::*;

pub(super) fn paint_overlay_layers<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    snapshot: &ViewSnapshot,
    zoom: f32,
    viewport_origin_x: f32,
    viewport_origin_y: f32,
    viewport_w: f32,
    viewport_h: f32,
) {
    if let Some(wire_drag) = canvas.interaction.wire_drag.clone() {
        canvas.paint_wire_drag_hint(cx, snapshot, &wire_drag, zoom);
    }

    if let Some(marquee) = canvas.interaction.marquee.clone() {
        canvas.paint_marquee(cx, &marquee, zoom);
    }

    if let Some(guides) = canvas.interaction.snap_guides {
        canvas.paint_snap_guides(
            cx,
            &guides,
            zoom,
            viewport_origin_x,
            viewport_origin_y,
            viewport_w,
            viewport_h,
        );
    }

    if let Some(searcher) = canvas.interaction.searcher.clone() {
        canvas.paint_searcher(cx, &searcher, zoom);
    }

    if let Some(menu) = canvas.interaction.context_menu.clone() {
        canvas.paint_context_menu(cx, &menu, zoom);
    }

    if let Some(toast) = canvas.interaction.toast.clone() {
        canvas.paint_toast(
            cx,
            &toast,
            zoom,
            viewport_origin_x,
            viewport_origin_y,
            viewport_h,
        );
    }
}
