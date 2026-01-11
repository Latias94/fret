use std::time::Instant;

use fret_core::Point;
use fret_ui::UiHost;

use super::{NodeGraphCanvas, ViewSnapshot};

pub(super) fn handle_panning_move<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
) -> bool {
    if !canvas.interaction.panning {
        return false;
    }

    canvas.stop_pan_inertia_timer(cx.app);

    let zoom = snapshot.zoom;
    if !zoom.is_finite() || zoom <= 0.0 {
        return false;
    }

    // `position` is in the node's local coordinate space (canvas coords) because `NodeGraphCanvas`
    // provides a `render_transform` for pan/zoom. Convert back to screen space so the delta is
    // stable even while pan changes (otherwise panning feeds back into the next pointer sample).
    let screen_pos = Point::new(
        fret_core::Px(cx.bounds.origin.x.0 + (position.x.0 + snapshot.pan.x) * zoom),
        fret_core::Px(cx.bounds.origin.y.0 + (position.y.0 + snapshot.pan.y) * zoom),
    );

    let last = canvas
        .interaction
        .pan_last_screen_pos
        .get_or_insert(screen_pos);
    let delta_screen = Point::new(screen_pos.x - last.x, screen_pos.y - last.y);
    *last = screen_pos;

    let inv_zoom = 1.0 / zoom;
    let delta_canvas = crate::core::CanvasPoint {
        x: delta_screen.x.0 * inv_zoom,
        y: delta_screen.y.0 * inv_zoom,
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
