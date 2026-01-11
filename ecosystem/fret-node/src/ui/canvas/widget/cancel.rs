use fret_ui::UiHost;

use fret_runtime::Effect;

use super::NodeGraphCanvas;
use crate::runtime::callbacks::{NodeDragEndOutcome, ViewportMoveEndOutcome, ViewportMoveKind};

fn cancel_active_gestures_inner<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    consume: bool,
) {
    let snapshot = canvas.sync_view_state(cx.app);
    let mode = snapshot.interaction.connection_mode;
    let mut canceled = false;
    if let Some(w) = canvas.interaction.wire_drag.take() {
        canvas.interaction.click_connect = false;
        canvas.emit_connect_end(
            mode,
            &w.kind,
            None,
            crate::runtime::callbacks::ConnectEndOutcome::Canceled,
        );
        canceled = true;
    }
    if canvas.interaction.edge_drag.take().is_some() {
        canceled = true;
    }
    if let Some(drag) = canvas.interaction.node_drag.take() {
        canvas.emit_node_drag_end(drag.primary, &drag.node_ids, NodeDragEndOutcome::Canceled);
        canceled = true;
    }
    if canvas.interaction.pending_node_drag.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.group_drag.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.pending_group_drag.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.group_resize.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.pending_group_resize.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.node_resize.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.pending_node_resize.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.pending_wire_drag.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.marquee.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.pending_marquee.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.panning {
        canvas.interaction.panning = false;
        canvas.interaction.panning_button = None;
        canvas.interaction.pan_last_screen_pos = None;
        canvas.interaction.pan_last_sample_at = None;
        canvas.emit_move_end(
            &snapshot,
            ViewportMoveKind::PanDrag,
            ViewportMoveEndOutcome::Canceled,
        );
        canceled = true;
    }
    if canvas.interaction.pan_inertia.is_some() {
        canvas.stop_pan_inertia_timer(cx.app);
        canvas.emit_move_end(
            &snapshot,
            ViewportMoveKind::PanInertia,
            ViewportMoveEndOutcome::Canceled,
        );
        canceled = true;
    }
    if let Some(state) = canvas.interaction.viewport_move_debounce.take() {
        cx.app
            .push_effect(Effect::CancelTimer { token: state.timer });
        canvas.emit_move_end(&snapshot, state.kind, ViewportMoveEndOutcome::Canceled);
        canceled = true;
    }
    if let Some(w) = canvas.interaction.suspended_wire_drag.take() {
        canvas.emit_connect_end(
            mode,
            &w.kind,
            None,
            crate::runtime::callbacks::ConnectEndOutcome::Canceled,
        );
        canceled = true;
    }
    if canvas.interaction.pending_right_click.take().is_some() {
        canceled = true;
    }
    if canvas.interaction.sticky_wire || canvas.interaction.sticky_wire_ignore_next_up {
        canvas.interaction.sticky_wire = false;
        canvas.interaction.sticky_wire_ignore_next_up = false;
        canceled = true;
    }
    if canvas.interaction.snap_guides.take().is_some() {
        canceled = true;
    }
    canvas.interaction.hover_port = None;
    canvas.interaction.hover_port_valid = false;
    canvas.interaction.hover_port_convertible = false;
    canvas.interaction.hover_edge = None;
    canvas.interaction.hover_edge_anchor = None;
    canvas.interaction.focused_edge = None;

    if canceled {
        canvas.stop_auto_pan_timer(cx.app);
        cx.release_pointer_capture();
        if consume {
            cx.stop_propagation();
        }
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    }
}

pub(super) fn cancel_active_gestures<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) {
    cancel_active_gestures_inner(canvas, cx, false);
}

pub(super) fn handle_escape_cancel<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) {
    cancel_active_gestures_inner(canvas, cx, true);
}
