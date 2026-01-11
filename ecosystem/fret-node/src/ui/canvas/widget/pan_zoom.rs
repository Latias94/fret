use std::time::Instant;

use fret_core::Point;
use fret_ui::UiHost;

use super::NodeGraphCanvas;

pub(super) fn handle_panning_move<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    delta: Point,
) -> bool {
    if !canvas.interaction.panning {
        return false;
    }

    canvas.stop_pan_inertia_timer(cx.app);

    let zoom = canvas.cached_zoom;
    let inv_zoom = if zoom.is_finite() && zoom > 0.0 {
        1.0 / zoom
    } else {
        1.0
    };
    let delta_canvas = crate::core::CanvasPoint {
        x: delta.x.0 * inv_zoom,
        y: delta.y.0 * inv_zoom,
    };

    let now = Instant::now();
    let dt = canvas
        .interaction
        .pan_last_sample_at
        .map(|prev| (now - prev).as_secs_f32())
        .unwrap_or_default();
    canvas.interaction.pan_last_sample_at = Some(now);

    if dt.is_finite() && dt > 0.0 && dt < 0.5 {
        let inst = crate::core::CanvasPoint {
            x: delta_canvas.x / dt,
            y: delta_canvas.y / dt,
        };
        let alpha = (dt * 24.0).clamp(0.0, 1.0);
        canvas.interaction.pan_velocity = crate::core::CanvasPoint {
            x: canvas.interaction.pan_velocity.x * (1.0 - alpha) + inst.x * alpha,
            y: canvas.interaction.pan_velocity.y * (1.0 - alpha) + inst.y * alpha,
        };
    } else if !dt.is_finite() {
        canvas.interaction.pan_velocity = crate::core::CanvasPoint::default();
    }

    canvas.update_view_state(cx.app, |s| {
        s.pan.x += delta_canvas.x;
        s.pan.y += delta_canvas.y;
    });
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}
