use super::super::*;

pub(super) fn apply_background_zoom_double_click<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: fret_core::Modifiers,
) {
    if let Some(state) = canvas.interaction.viewport_move_debounce.take() {
        cx.app
            .push_effect(Effect::CancelTimer { token: state.timer });
        canvas.emit_move_end(snapshot, state.kind, ViewportMoveEndOutcome::Ended);
    }

    canvas.emit_move_start(snapshot, ViewportMoveKind::ZoomDoubleClick);
    let factor = if modifiers.shift { 0.5 } else { 2.0 };
    canvas.zoom_about_pointer_factor(position, factor);
    let pan = canvas.cached_pan;
    let zoom = canvas.cached_zoom;
    canvas.update_view_state(cx.app, |state| {
        state.pan = pan;
        state.zoom = zoom;
    });
    let snap = canvas.sync_view_state(cx.app);
    canvas.emit_move_end(
        &snap,
        ViewportMoveKind::ZoomDoubleClick,
        ViewportMoveEndOutcome::Ended,
    );
    cx.stop_propagation();
    paint_invalidation::invalidate_paint(cx);
}
