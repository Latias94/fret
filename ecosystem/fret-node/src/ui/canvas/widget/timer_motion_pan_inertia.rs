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
    let mut end_move = false;

    if !tuning.enabled
        || !canvas.pan_inertia_should_tick()
        || !zoom.is_finite()
        || zoom <= 0.0
        || !tuning.decay_per_s.is_finite()
        || tuning.decay_per_s <= 0.0
    {
        cx.app.push_effect(Effect::CancelTimer { token: timer });
        end_move = true;
        invalidate_motion(cx);
        if end_move {
            let snapshot = canvas.sync_view_state(cx.app);
            canvas.emit_move_end(
                &snapshot,
                ViewportMoveKind::PanInertia,
                ViewportMoveEndOutcome::Ended,
            );
        }
        return true;
    }

    let now = Instant::now();
    let dt = (now - inertia.last_tick_at).as_secs_f32().clamp(0.0, 0.2);
    inertia.last_tick_at = now;

    if dt > 0.0 {
        let dx = inertia.velocity.x * dt;
        let dy = inertia.velocity.y * dt;
        canvas.update_view_state(cx.app, |state| {
            state.pan.x += dx;
            state.pan.y += dy;
        });
    }

    let after = canvas.sync_view_state(cx.app).pan;
    let moved_x = after.x - before.x;
    let moved_y = after.y - before.y;
    let moved = (moved_x * moved_x + moved_y * moved_y).sqrt();

    let decay = (-tuning.decay_per_s * dt).exp();
    inertia.velocity.x *= decay;
    inertia.velocity.y *= decay;

    let speed_screen =
        (inertia.velocity.x * inertia.velocity.x + inertia.velocity.y * inertia.velocity.y).sqrt()
            * zoom;
    let min_speed = tuning.min_speed.max(0.0);

    if moved <= 1.0e-6
        || !speed_screen.is_finite()
        || speed_screen <= min_speed
        || !inertia.velocity.x.is_finite()
        || !inertia.velocity.y.is_finite()
    {
        cx.app.push_effect(Effect::CancelTimer { token: timer });
        end_move = true;
    } else {
        canvas.interaction.pan_inertia = Some(inertia);
    }

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
