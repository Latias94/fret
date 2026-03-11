use crate::core::CanvasPoint;
use crate::ui::canvas::state::{ViewportAnimationEase, ViewportAnimationInterpolate};
use crate::ui::canvas::widget::*;

use super::super::timer_motion_shared::invalidate_motion;

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

    let (pan, zoom, done) = sample_animation_frame::<M>(&mut animation);

    canvas.update_view_state(cx.app, |state| {
        state.pan = pan;
        state.zoom = zoom;
    });

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

fn sample_animation_frame<M: NodeGraphCanvasMiddleware>(
    animation: &mut crate::ui::canvas::state::ViewportAnimationState,
) -> (CanvasPoint, f32, bool) {
    let (now, dt) = clamped_tick_delta::<M>(animation.last_tick_at);
    animation.last_tick_at = now;
    animation.elapsed = (animation.elapsed + dt).min(animation.duration);

    let t = normalized_progress(animation);
    let u = eased_progress(animation, t);
    let pan = CanvasPoint {
        x: animation.from_pan.x + (animation.to_pan.x - animation.from_pan.x) * u,
        y: animation.from_pan.y + (animation.to_pan.y - animation.from_pan.y) * u,
    };
    let zoom = animation.from_zoom + (animation.to_zoom - animation.from_zoom) * u;
    let done = t >= 1.0 - 1.0e-6;
    (pan, zoom, done)
}

fn clamped_tick_delta<M: NodeGraphCanvasMiddleware>(
    last_tick_at: fret_core::time::Instant,
) -> (fret_core::time::Instant, std::time::Duration) {
    let now = fret_core::time::Instant::now();
    let mut dt = now.checked_duration_since(last_tick_at).unwrap_or_default();
    if dt < NodeGraphCanvasWith::<M>::PAN_INERTIA_TICK_INTERVAL {
        dt = NodeGraphCanvasWith::<M>::PAN_INERTIA_TICK_INTERVAL;
    }
    (now, dt.min(std::time::Duration::from_millis(200)))
}

fn normalized_progress(animation: &crate::ui::canvas::state::ViewportAnimationState) -> f32 {
    let denom = animation.duration.as_secs_f32();
    if denom > 0.0 {
        (animation.elapsed.as_secs_f32() / denom).clamp(0.0, 1.0)
    } else {
        1.0
    }
}

fn eased_progress(animation: &crate::ui::canvas::state::ViewportAnimationState, t: f32) -> f32 {
    match animation.ease {
        Some(ease) => ease.apply(t),
        None => match animation.interpolate {
            ViewportAnimationInterpolate::Linear => t,
            ViewportAnimationInterpolate::Smooth => ViewportAnimationEase::Smoothstep.apply(t),
        },
    }
}
