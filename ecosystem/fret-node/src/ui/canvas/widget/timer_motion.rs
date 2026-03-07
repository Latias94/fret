use super::*;
use crate::ui::canvas::state::{ViewportAnimationEase, ViewportAnimationInterpolate};

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

pub(super) fn handle_viewport_animation_tick<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    token: fret_core::TimerToken,
) -> bool {
    if !canvas
        .interaction
        .viewport_animation
        .as_ref()
        .is_some_and(|animation| animation.timer == token)
    {
        return false;
    }

    let Some(mut animation) = canvas.interaction.viewport_animation.take() else {
        return true;
    };

    if animation.duration.is_zero() {
        cx.app.push_effect(Effect::CancelTimer {
            token: animation.timer,
        });
        return true;
    }

    let now = fret_core::time::Instant::now();
    let mut dt = now
        .checked_duration_since(animation.last_tick_at)
        .unwrap_or_default();
    if dt < NodeGraphCanvasWith::<M>::PAN_INERTIA_TICK_INTERVAL {
        dt = NodeGraphCanvasWith::<M>::PAN_INERTIA_TICK_INTERVAL;
    }
    if dt > std::time::Duration::from_millis(200) {
        dt = std::time::Duration::from_millis(200);
    }
    animation.last_tick_at = now;
    animation.elapsed = (animation.elapsed + dt).min(animation.duration);

    let denom = animation.duration.as_secs_f32();
    let t = if denom > 0.0 {
        (animation.elapsed.as_secs_f32() / denom).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let u = match animation.ease {
        Some(ease) => ease.apply(t),
        None => match animation.interpolate {
            ViewportAnimationInterpolate::Linear => t,
            ViewportAnimationInterpolate::Smooth => ViewportAnimationEase::Smoothstep.apply(t),
        },
    };

    let pan = CanvasPoint {
        x: animation.from_pan.x + (animation.to_pan.x - animation.from_pan.x) * u,
        y: animation.from_pan.y + (animation.to_pan.y - animation.from_pan.y) * u,
    };
    let zoom = animation.from_zoom + (animation.to_zoom - animation.from_zoom) * u;

    canvas.update_view_state(cx.app, |state| {
        state.pan = pan;
        state.zoom = zoom;
    });

    let done = t >= 1.0 - 1.0e-6;
    if done {
        cx.app.push_effect(Effect::CancelTimer {
            token: animation.timer,
        });
    } else {
        canvas.interaction.viewport_animation = Some(animation);
    }

    invalidate_motion(cx);
    true
}

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

    dispatch_auto_pan_move(canvas, cx, snapshot, position, modifiers, zoom);

    let snapshot = canvas.sync_view_state(cx.app);
    canvas.sync_auto_pan_timer(cx.app, cx.window, &snapshot, cx.bounds);
    invalidate_motion(cx);
    true
}

pub(super) fn handle_viewport_move_debounce<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    token: fret_core::TimerToken,
) -> bool {
    if !canvas
        .interaction
        .viewport_move_debounce
        .as_ref()
        .is_some_and(|state| state.timer == token)
    {
        return false;
    }

    let Some(state) = canvas.interaction.viewport_move_debounce.take() else {
        return true;
    };
    let snapshot = canvas.sync_view_state(cx.app);
    canvas.emit_move_end(&snapshot, state.kind, ViewportMoveEndOutcome::Ended);
    invalidate_motion(cx);
    true
}

fn dispatch_auto_pan_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) {
    if canvas.interaction.wire_drag.is_some() {
        let _ = wire_drag::handle_wire_drag_move(canvas, cx, snapshot, position, modifiers, zoom);
    } else if canvas.interaction.node_drag.is_some() {
        let _ = node_drag::handle_node_drag_move(canvas, cx, snapshot, position, modifiers, zoom);
    } else if canvas.interaction.group_drag.is_some() {
        let _ = group_drag::handle_group_drag_move(canvas, cx, snapshot, position, modifiers, zoom);
    } else if canvas.interaction.group_resize.is_some() {
        let _ =
            group_resize::handle_group_resize_move(canvas, cx, snapshot, position, modifiers, zoom);
    }
}

fn invalidate_motion<H: UiHost>(cx: &mut EventCx<'_, H>) {
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
}
