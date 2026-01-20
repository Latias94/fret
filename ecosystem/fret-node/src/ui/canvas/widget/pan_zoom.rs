use std::time::Instant;

use fret_canvas::view::PanZoom2D;
use fret_core::{MouseButton, Point};
use fret_runtime::Effect;
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};
use crate::core::CanvasPoint;
use crate::runtime::callbacks::{ViewportMoveEndOutcome, ViewportMoveKind};

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn zoom_about_center_factor(&mut self, bounds: fret_core::Rect, factor: f32) {
        let zoom = self.cached_zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return;
        }
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }

        let new_zoom = (zoom * factor).clamp(self.style.min_zoom, self.style.max_zoom);
        if (new_zoom - zoom).abs() <= 1.0e-6 {
            return;
        }

        let mut view = PanZoom2D {
            pan: Point::new(
                fret_core::Px(self.cached_pan.x),
                fret_core::Px(self.cached_pan.y),
            ),
            zoom,
        };
        let center = Point::new(
            fret_core::Px(0.5 * bounds.size.width.0),
            fret_core::Px(0.5 * bounds.size.height.0),
        );
        view.zoom_about_screen_point(bounds, center, new_zoom);
        self.cached_pan = CanvasPoint {
            x: view.pan.x.0,
            y: view.pan.y.0,
        };
        self.cached_zoom = view.zoom;
    }

    pub(super) fn zoom_about_pointer_factor(&mut self, position: Point, factor: f32) {
        let zoom = self.cached_zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return;
        }
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }
        if !position.x.0.is_finite() || !position.y.0.is_finite() {
            return;
        }

        let new_zoom = (zoom * factor).clamp(self.style.min_zoom, self.style.max_zoom);
        if (new_zoom - zoom).abs() <= 1.0e-6 {
            return;
        }

        let pan_x = self.cached_pan.x;
        let pan_y = self.cached_pan.y;

        // `position` is in the widget's local (canvas) coordinates.
        // Compute the pivot in screen coordinates (relative to bounds origin) to keep the
        // graph point under the cursor stable.
        let pivot_screen_x = (position.x.0 + pan_x) * zoom;
        let pivot_screen_y = (position.y.0 + pan_y) * zoom;

        let g0_x = pivot_screen_x / zoom - pan_x;
        let g0_y = pivot_screen_y / zoom - pan_y;

        let new_pan_x = pivot_screen_x / new_zoom - g0_x;
        let new_pan_y = pivot_screen_y / new_zoom - g0_y;

        self.cached_pan = CanvasPoint {
            x: new_pan_x,
            y: new_pan_y,
        };
        self.cached_zoom = new_zoom;
    }
}

pub(super) fn begin_panning<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    start_pos: Point,
    button: MouseButton,
) -> bool {
    if canvas.interaction.pan_inertia.is_some() {
        canvas.stop_pan_inertia_timer(cx.app);
        canvas.emit_move_end(
            snapshot,
            ViewportMoveKind::PanInertia,
            ViewportMoveEndOutcome::Ended,
        );
    }
    if let Some(state) = canvas.interaction.viewport_move_debounce.take() {
        cx.app
            .push_effect(Effect::CancelTimer { token: state.timer });
        canvas.emit_move_end(snapshot, state.kind, ViewportMoveEndOutcome::Ended);
    }

    canvas.interaction.hover_edge = None;
    canvas.interaction.pending_group_drag = None;
    canvas.interaction.group_drag = None;
    canvas.interaction.pending_group_resize = None;
    canvas.interaction.group_resize = None;
    canvas.interaction.pending_node_drag = None;
    canvas.interaction.node_drag = None;
    canvas.interaction.pending_node_resize = None;
    canvas.interaction.node_resize = None;
    canvas.interaction.pending_wire_drag = None;
    canvas.interaction.wire_drag = None;
    canvas.interaction.click_connect = false;
    canvas.interaction.edge_drag = None;
    canvas.interaction.pending_marquee = None;
    canvas.interaction.marquee = None;
    canvas.interaction.focused_edge = None;
    canvas.interaction.hover_port = None;
    canvas.interaction.hover_port_valid = false;
    canvas.interaction.hover_port_convertible = false;
    canvas.interaction.hover_port_diagnostic = None;

    canvas.interaction.panning = true;
    canvas.interaction.panning_button = Some(button);

    let zoom = snapshot.zoom;
    let pan = snapshot.pan;
    let screen_pos = Point::new(
        fret_core::Px(cx.bounds.origin.x.0 + (start_pos.x.0 + pan.x) * zoom),
        fret_core::Px(cx.bounds.origin.y.0 + (start_pos.y.0 + pan.y) * zoom),
    );
    canvas.interaction.pan_last_screen_pos = Some(screen_pos);
    canvas.interaction.pan_last_sample_at = Some(Instant::now());
    canvas.interaction.pan_velocity = CanvasPoint::default();

    canvas.emit_move_start(snapshot, ViewportMoveKind::PanDrag);

    cx.capture_pointer(cx.node);
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}

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
