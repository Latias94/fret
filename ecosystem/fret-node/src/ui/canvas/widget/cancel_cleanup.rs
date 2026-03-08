use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::interaction::NodeGraphConnectionMode;
use crate::runtime::callbacks::ConnectEndOutcome;

pub(super) fn cancel_cleanup_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    mode: NodeGraphConnectionMode,
) -> bool {
    let mut canceled = false;

    if let Some(wire_drag) = canvas.interaction.suspended_wire_drag.take() {
        canvas.emit_connect_end(mode, &wire_drag.kind, None, ConnectEndOutcome::Canceled);
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

    canceled
}

pub(super) fn clear_hover_and_focus<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
) {
    canvas.interaction.hover_port = None;
    canvas.interaction.hover_port_valid = false;
    canvas.interaction.hover_port_convertible = false;
    canvas.interaction.hover_port_diagnostic = None;
    canvas.interaction.hover_edge = None;
    canvas.interaction.hover_edge_anchor = None;
    canvas.interaction.focused_edge = None;
}

pub(super) fn finish_cancel<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    consume: bool,
) {
    canvas.stop_auto_pan_timer(cx.app);
    cx.release_pointer_capture();
    if consume {
        cx.stop_propagation();
    }
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
}
