use super::*;

pub(super) fn auto_pan_delta(snapshot: &ViewSnapshot, pos: Point, bounds: Rect) -> CanvasPoint {
    let view = NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::viewport_from_snapshot(
        bounds, snapshot,
    )
    .view;
    let tuning = fret_canvas::view::AutoPanTuning {
        margin_screen_px: snapshot.interaction.auto_pan.margin,
        speed_screen_px_per_s: snapshot.interaction.auto_pan.speed,
    };

    let delta = fret_canvas::view::auto_pan_delta_per_tick(
        bounds,
        view,
        pos,
        tuning,
        NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::AUTO_PAN_TICK_HZ,
    );

    CanvasPoint {
        x: delta.x.0,
        y: delta.y.0,
    }
}

pub(super) fn stop_auto_pan_timer<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
) {
    let Some(timer) = canvas.interaction.auto_pan_timer.take() else {
        return;
    };
    host.push_effect(Effect::CancelTimer { token: timer });
}

pub(super) fn ensure_auto_pan_timer_running<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
) {
    if canvas.interaction.auto_pan_timer.is_some() {
        return;
    }
    let timer = host.next_timer_token();
    host.push_effect(Effect::SetTimer {
        window,
        token: timer,
        after: NodeGraphCanvasWith::<M>::AUTO_PAN_TICK_INTERVAL,
        repeat: Some(NodeGraphCanvasWith::<M>::AUTO_PAN_TICK_INTERVAL),
    });
    canvas.interaction.auto_pan_timer = Some(timer);
}

pub(super) fn auto_pan_should_tick<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    bounds: Rect,
) -> bool {
    if canvas.interaction.searcher.is_some() || canvas.interaction.context_menu.is_some() {
        return false;
    }
    let Some(pos) = canvas.interaction.last_pos else {
        return false;
    };

    let wants_node_drag = snapshot.interaction.auto_pan.on_node_drag
        && (canvas.interaction.node_drag.is_some()
            || canvas.interaction.group_drag.is_some()
            || canvas.interaction.group_resize.is_some());
    let wants_connect =
        snapshot.interaction.auto_pan.on_connect && canvas.interaction.wire_drag.is_some();

    if !wants_node_drag && !wants_connect {
        return false;
    }

    let delta = auto_pan_delta(snapshot, pos, bounds);
    delta.x != 0.0 || delta.y != 0.0
}

pub(super) fn sync_auto_pan_timer<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    snapshot: &ViewSnapshot,
    bounds: Rect,
) {
    if auto_pan_should_tick(canvas, snapshot, bounds) {
        ensure_auto_pan_timer_running(canvas, host, window);
    } else {
        stop_auto_pan_timer(canvas, host);
    }
}
