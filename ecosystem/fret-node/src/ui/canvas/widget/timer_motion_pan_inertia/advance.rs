use fret_core::time::Instant;
use fret_ui::UiHost;

use crate::core::CanvasPoint;
use crate::io::NodeGraphPanInertiaTuning;
use crate::ui::canvas::state::PanInertiaState;
use crate::ui::canvas::widget::*;

pub(super) fn advance_pan_inertia_frame<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    before: CanvasPoint,
    zoom: f32,
    tuning: &NodeGraphPanInertiaTuning,
    inertia: &mut PanInertiaState,
) -> bool {
    let now = Instant::now();
    let dt = (now - inertia.last_tick_at).as_secs_f32().clamp(0.0, 0.2);
    inertia.last_tick_at = now;

    if dt > 0.0 {
        let dx = inertia.velocity.x * dt;
        let dy = inertia.velocity.y * dt;
        canvas.update_view_state(host, |state| {
            state.pan.x += dx;
            state.pan.y += dy;
        });
    }

    let moved = moved_distance(before, canvas.sync_view_state(host).pan);
    let decay = (-tuning.decay_per_s * dt).exp();
    inertia.velocity.x *= decay;
    inertia.velocity.y *= decay;

    let speed_screen =
        (inertia.velocity.x * inertia.velocity.x + inertia.velocity.y * inertia.velocity.y).sqrt()
            * zoom;
    let min_speed = tuning.min_speed.max(0.0);

    moved <= 1.0e-6
        || !speed_screen.is_finite()
        || speed_screen <= min_speed
        || !inertia.velocity.x.is_finite()
        || !inertia.velocity.y.is_finite()
}

fn moved_distance(before: CanvasPoint, after: CanvasPoint) -> f32 {
    let moved_x = after.x - before.x;
    let moved_y = after.y - before.y;
    (moved_x * moved_x + moved_y * moved_y).sqrt()
}
