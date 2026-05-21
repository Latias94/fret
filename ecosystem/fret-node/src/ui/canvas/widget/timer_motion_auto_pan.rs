mod dispatch;

use super::timer_motion_shared::invalidate_motion;
use super::*;

pub(super) fn handle_auto_pan_tick<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    token: fret_core::TimerToken,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: timer_motion_cx::TimerMotionCx<H>,
{
    if canvas.interaction.auto_pan_timer != Some(token) {
        return false;
    }

    if !canvas.auto_pan_should_tick(snapshot, auto_pan_timer_cx::AutoPanTimerCx::bounds(cx)) {
        canvas.stop_auto_pan_timer(auto_pan_timer_cx::AutoPanTimerCx::host(cx));
        return true;
    }

    let position = canvas.interaction.last_pos.unwrap_or_default();
    let modifiers = canvas.interaction.last_modifiers;
    let zoom = snapshot.zoom;

    dispatch::dispatch_auto_pan_move(canvas, cx, snapshot, position, modifiers, zoom);

    let snapshot = canvas.sync_view_state(auto_pan_timer_cx::AutoPanTimerCx::host(cx));
    let window = auto_pan_timer_cx::AutoPanTimerCx::window(cx);
    let bounds = auto_pan_timer_cx::AutoPanTimerCx::bounds(cx);
    canvas.sync_auto_pan_timer(
        auto_pan_timer_cx::AutoPanTimerCx::host(cx),
        window,
        &snapshot,
        bounds,
    );
    invalidate_motion(cx);
    true
}
