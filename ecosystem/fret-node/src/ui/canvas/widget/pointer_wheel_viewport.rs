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
    canvas.zoom_about_pointer_factor(position, factor);
    let pan = canvas.cached_pan;
    let zoom = canvas.cached_zoom;
    canvas.update_view_state(cx.app, |s| {
        s.pan = pan;
        s.zoom = zoom;
    });
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
    true
}

pub(super) fn handle_scroll_pan<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    delta: Point,
    modifiers: fret_core::Modifiers,
) -> bool {
    if !(snapshot.interaction.pan_on_scroll
        || (snapshot.interaction.space_to_pan && canvas.interaction.pan_activation_key_held))
    {
        return false;
    }

    canvas.bump_viewport_move_debounce(cx.app, cx.window, snapshot, ViewportMoveKind::PanScroll);
    let mode = snapshot.interaction.pan_on_scroll_mode;
    let speed = snapshot.interaction.pan_on_scroll_speed.max(0.0);
    let dy_for_shift = delta.y.0;

    let mut dx = delta.x.0;
    let mut dy = delta.y.0;
    match mode {
        crate::io::NodeGraphPanOnScrollMode::Free => {}
        crate::io::NodeGraphPanOnScrollMode::Horizontal => {
            dy = 0.0;
        }
        crate::io::NodeGraphPanOnScrollMode::Vertical => {
            dx = 0.0;
        }
    }

    if cx.input_ctx.platform != fret_runtime::Platform::Macos
        && modifiers.shift
        && !matches!(mode, crate::io::NodeGraphPanOnScrollMode::Vertical)
    {
        dx = dy_for_shift;
        dy = 0.0;
    }
    canvas.update_view_state(cx.app, |s| {
        s.pan.x += dx * speed;
        s.pan.y += dy * speed;
    });
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
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
    let delta = delta.clamp(-0.95, 10.0);
    let factor = (1.0 + delta * speed).max(0.01);
    canvas.zoom_about_pointer_factor(position, factor);
    let pan = canvas.cached_pan;
    let zoom = canvas.cached_zoom;
    canvas.update_view_state(cx.app, |s| {
        s.pan = pan;
        s.zoom = zoom;
    });
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
    true
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
