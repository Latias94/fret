use super::*;

pub(super) fn handle_scroll_zoom<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    delta: Point,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) -> bool {
    let zoom_active = snapshot
        .interaction
        .zoom_activation_key
        .is_pressed(modifiers);
    if !(snapshot.interaction.zoom_on_scroll && zoom_active) {
        return false;
    }

    canvas.bump_viewport_move_debounce(cx.app, cx.window, snapshot, ViewportMoveKind::ZoomWheel);
    let speed = snapshot.interaction.zoom_on_scroll_speed.max(0.0);
    let delta_screen_y = delta.y.0 * zoom;
    let factor = fret_canvas::view::wheel_zoom_factor(
        delta_screen_y,
        fret_canvas::view::DEFAULT_WHEEL_ZOOM_BASE,
        fret_canvas::view::DEFAULT_WHEEL_ZOOM_STEP,
        speed,
    )
    .unwrap_or(1.0);
    apply_viewport_zoom(canvas, cx.app, position, factor);
    finish_viewport_zoom(cx);
    true
}

pub(super) fn handle_pinch_zoom<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    delta: f32,
) -> bool {
    if !snapshot.interaction.zoom_on_pinch || !delta.is_finite() {
        return false;
    }

    canvas.bump_viewport_move_debounce(cx.app, cx.window, snapshot, ViewportMoveKind::ZoomPinch);
    let speed = snapshot.interaction.zoom_on_pinch_speed.max(0.0);
    let factor = (1.0 + delta.clamp(-0.95, 10.0) * speed).max(0.01);
    apply_viewport_zoom(canvas, cx.app, position, factor);
    finish_viewport_zoom(cx);
    true
}

fn apply_viewport_zoom<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut impl UiHost,
    position: Point,
    factor: f32,
) {
    canvas.zoom_about_pointer_factor(position, factor);
    let pan = canvas.cached_pan;
    let zoom = canvas.cached_zoom;
    canvas.update_view_state(host, |state| {
        state.pan = pan;
        state.zoom = zoom;
    });
}

fn finish_viewport_zoom<H: UiHost>(cx: &mut EventCx<'_, H>) {
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
}
