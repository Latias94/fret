use fret_core::time::Instant;

use fret_core::{MouseButton, Point};
use fret_runtime::Effect;
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};
use crate::core::CanvasPoint;
use crate::runtime::callbacks::{ViewportMoveEndOutcome, ViewportMoveKind};

pub(super) fn begin_panning<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    start_pos: Point,
    button: MouseButton,
) -> bool {
    cancel_previous_motion(canvas, cx, snapshot);
    clear_competing_interactions(canvas);

    canvas.interaction.panning = true;
    canvas.interaction.panning_button = Some(button);

    let viewport =
        NodeGraphCanvasWith::<M>::viewport_from_pan_zoom(cx.bounds, snapshot.pan, snapshot.zoom);
    let screen_pos = viewport.canvas_to_screen(start_pos);
    canvas.interaction.pan_last_screen_pos = Some(screen_pos);
    canvas.interaction.pan_last_sample_at = Some(Instant::now());
    canvas.interaction.pan_velocity = CanvasPoint::default();

    canvas.emit_move_start(snapshot, ViewportMoveKind::PanDrag);
    cx.capture_pointer(cx.node);
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}

fn cancel_previous_motion<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
) {
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
}

fn clear_competing_interactions<M: NodeGraphCanvasMiddleware>(canvas: &mut NodeGraphCanvasWith<M>) {
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
}
