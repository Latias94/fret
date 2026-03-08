use super::*;

pub(super) fn stop_scroll_viewport_motion<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
) {
    if canvas.interaction.viewport_animation.is_some() {
        canvas.stop_viewport_animation_timer(cx.app);
    }
    stop_pan_inertia(canvas, cx, snapshot);
}

pub(super) fn stop_pinch_viewport_motion<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
) {
    stop_pan_inertia(canvas, cx, snapshot);
}

fn stop_pan_inertia<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
) {
    if canvas.interaction.pan_inertia.is_some() {
        canvas.stop_pan_inertia_timer(cx.app);
        canvas.emit_move_end(
            snapshot,
            ViewportMoveKind::PanInertia,
            ViewportMoveEndOutcome::Ended,
        );
    }
}
