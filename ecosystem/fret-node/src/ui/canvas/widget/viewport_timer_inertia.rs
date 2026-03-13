use super::*;

pub(super) fn stop_pan_inertia_timer<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
) {
    let Some(inertia) = canvas.interaction.pan_inertia.take() else {
        return;
    };
    host.push_effect(Effect::CancelTimer {
        token: inertia.timer,
    });
}

pub(super) fn pan_inertia_should_tick<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
) -> bool {
    super::interaction_gate::pan_inertia_should_tick(&canvas.interaction)
}

pub(super) fn maybe_start_pan_inertia_timer<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    snapshot: &ViewSnapshot,
) -> bool {
    stop_pan_inertia_timer(canvas, host);

    let tuning = &snapshot.interaction.pan_inertia;
    if !tuning.enabled {
        return false;
    }

    let zoom = snapshot.zoom;
    if !zoom.is_finite() || zoom <= 0.0 {
        return false;
    }

    let mut velocity = canvas.interaction.pan_velocity;
    if !velocity.x.is_finite() || !velocity.y.is_finite() {
        return false;
    }

    let speed_screen = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt() * zoom;
    let min_speed = tuning.min_speed.max(0.0);
    if !speed_screen.is_finite() || speed_screen < min_speed {
        return false;
    }

    let max_speed = tuning.max_speed.max(min_speed);
    if max_speed.is_finite() && max_speed > 0.0 {
        let max_speed_canvas = max_speed / zoom;
        let speed_canvas = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
        if speed_canvas.is_finite() && speed_canvas > max_speed_canvas && speed_canvas > 0.0 {
            let scale = max_speed_canvas / speed_canvas;
            velocity.x *= scale;
            velocity.y *= scale;
        }
    }

    let timer = host.next_timer_token();
    host.push_effect(Effect::SetTimer {
        window,
        token: timer,
        after: NodeGraphCanvasWith::<M>::PAN_INERTIA_TICK_INTERVAL,
        repeat: Some(NodeGraphCanvasWith::<M>::PAN_INERTIA_TICK_INTERVAL),
    });
    canvas.interaction.pan_inertia = Some(PanInertiaState {
        timer,
        velocity,
        last_tick_at: Instant::now(),
    });
    true
}
