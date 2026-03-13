mod advance;
mod guards;

use super::timer_motion_shared::invalidate_motion;
use super::*;

pub(super) fn handle_pan_inertia_tick<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    token: fret_core::TimerToken,
) -> bool {
    if !canvas
        .interaction
        .pan_inertia
        .as_ref()
        .is_some_and(|inertia| inertia.timer == token)
    {
        return false;
    }

    let tuning = snapshot.interaction.pan_inertia.clone();
    let zoom = snapshot.zoom;
    let before = snapshot.pan;

    let Some(mut inertia) = canvas.interaction.pan_inertia.take() else {
        return true;
    };
    let timer = inertia.timer;

    let end_move = if guards::should_stop_pan_inertia(canvas, &tuning, zoom) {
        cx.app.push_effect(Effect::CancelTimer { token: timer });
        true
    } else if advance::advance_pan_inertia_frame(
        canvas,
        cx.app,
        before,
        zoom,
        &tuning,
        &mut inertia,
    ) {
        cx.app.push_effect(Effect::CancelTimer { token: timer });
        true
    } else {
        canvas.interaction.pan_inertia = Some(inertia);
        false
    };

    invalidate_motion(cx);
    if end_move {
        let snapshot = canvas.sync_view_state(cx.app);
        canvas.emit_move_end(
            &snapshot,
            ViewportMoveKind::PanInertia,
            ViewportMoveEndOutcome::Ended,
        );
    }
    true
}
