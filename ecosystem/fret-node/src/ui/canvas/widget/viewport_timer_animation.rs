use super::*;
use crate::ui::canvas::state::{
    ViewportAnimationEase, ViewportAnimationInterpolate, ViewportAnimationState,
};

pub(super) fn stop_viewport_animation_timer<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
) {
    let Some(anim) = canvas.interaction.viewport_animation.take() else {
        return;
    };
    host.push_effect(Effect::CancelTimer { token: anim.timer });
}

pub(super) fn start_viewport_animation_to<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    from_pan: CanvasPoint,
    from_zoom: f32,
    to_pan: CanvasPoint,
    to_zoom: f32,
    duration: std::time::Duration,
    interpolate: ViewportAnimationInterpolate,
    ease: Option<ViewportAnimationEase>,
) -> bool {
    stop_viewport_animation_timer(canvas, host);

    if duration.is_zero() {
        return false;
    }

    let timer = host.next_timer_token();
    host.push_effect(Effect::SetTimer {
        window,
        token: timer,
        after: NodeGraphCanvasWith::<M>::PAN_INERTIA_TICK_INTERVAL,
        repeat: Some(NodeGraphCanvasWith::<M>::PAN_INERTIA_TICK_INTERVAL),
    });

    let now = fret_core::time::Instant::now();
    canvas.interaction.viewport_animation = Some(ViewportAnimationState {
        timer,
        from_pan,
        from_zoom,
        to_pan,
        to_zoom,
        interpolate,
        ease,
        duration,
        elapsed: std::time::Duration::ZERO,
        last_tick_at: now,
    });
    true
}

pub(super) fn bump_viewport_move_debounce<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    snapshot: &ViewSnapshot,
    kind: ViewportMoveKind,
) {
    if let Some(prev) = canvas.interaction.viewport_move_debounce.take() {
        host.push_effect(Effect::CancelTimer { token: prev.timer });
        if prev.kind != kind {
            canvas.emit_move_end(snapshot, prev.kind, ViewportMoveEndOutcome::Ended);
            canvas.emit_move_start(snapshot, kind);
        }
    } else {
        canvas.emit_move_start(snapshot, kind);
    }

    let timer = host.next_timer_token();
    host.push_effect(Effect::SetTimer {
        window,
        token: timer,
        after: NodeGraphCanvasWith::<M>::VIEWPORT_MOVE_END_DEBOUNCE,
        repeat: None,
    });
    canvas.interaction.viewport_move_debounce = Some(ViewportMoveDebounceState { kind, timer });
}
