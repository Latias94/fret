mod dispatch;

use super::timer_motion_shared::invalidate_motion;
use super::*;

pub(super) fn handle_auto_pan_tick<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    token: fret_core::TimerToken,
) -> bool {
    if canvas.interaction.auto_pan_timer != Some(token) {
        return false;
    }

    if !canvas.auto_pan_should_tick(snapshot, cx.bounds) {
        canvas.stop_auto_pan_timer(cx.app);
        return true;
    }

    let position = canvas.interaction.last_pos.unwrap_or_default();
    let modifiers = canvas.interaction.last_modifiers;
    let zoom = snapshot.zoom;

    dispatch::dispatch_auto_pan_move(canvas, cx, snapshot, position, modifiers, zoom);

    let snapshot = canvas.sync_view_state(cx.app);
    canvas.sync_auto_pan_timer(cx.app, cx.window, &snapshot, cx.bounds);
    invalidate_motion(cx);
    true
}
