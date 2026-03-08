use super::timer_motion_shared::invalidate_motion;
use super::*;
use crate::ui::canvas::state::{ViewportAnimationEase, ViewportAnimationInterpolate};

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
