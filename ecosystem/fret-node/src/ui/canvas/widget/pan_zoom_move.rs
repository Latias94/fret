use fret_core::time::Instant;

use fret_core::Point;
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};

pub(super) fn handle_panning_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
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

    let viewport = NodeGraphCanvasWith::<M>::viewport_from_pan_zoom(cx.bounds, snapshot.pan, zoom);
    let screen_pos = viewport.canvas_to_screen(position);

    let last = canvas
        .interaction
        .pan_last_screen_pos
        .get_or_insert(screen_pos);
    let delta_screen = Point::new(screen_pos.x - last.x, screen_pos.y - last.y);
    *last = screen_pos;

    let delta_canvas = crate::core::CanvasPoint {
        x: fret_canvas::scale::canvas_units_from_screen_px(delta_screen.x.0, zoom),
        y: fret_canvas::scale::canvas_units_from_screen_px(delta_screen.y.0, zoom),
    };

    update_pan_velocity(canvas, delta_canvas);
    canvas.update_view_state(cx.app, |state| {
        state.pan.x += delta_canvas.x;
        state.pan.y += delta_canvas.y;
    });
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}

fn update_pan_velocity<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    delta_canvas: crate::core::CanvasPoint,
) {
    let now = Instant::now();
    let dt = canvas
        .interaction
        .pan_last_sample_at
        .map(|prev| (now - prev).as_secs_f32())
        .unwrap_or_default();
    canvas.interaction.pan_last_sample_at = Some(now);

    if dt.is_finite() && dt > 0.0 && dt < 0.5 {
        let instant_velocity = crate::core::CanvasPoint {
            x: delta_canvas.x / dt,
            y: delta_canvas.y / dt,
        };
        let alpha = (dt * 24.0).clamp(0.0, 1.0);
        canvas.interaction.pan_velocity = crate::core::CanvasPoint {
            x: canvas.interaction.pan_velocity.x * (1.0 - alpha) + instant_velocity.x * alpha,
            y: canvas.interaction.pan_velocity.y * (1.0 - alpha) + instant_velocity.y * alpha,
        };
    } else if !dt.is_finite() {
        canvas.interaction.pan_velocity = crate::core::CanvasPoint::default();
    }
}
