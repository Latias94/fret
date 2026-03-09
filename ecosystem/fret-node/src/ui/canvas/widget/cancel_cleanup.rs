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
    canceled |= super::cancel_session::clear_cancel_residuals(&mut canvas.interaction);

    canceled
}

pub(super) fn clear_hover_and_focus<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
) {
    super::cancel_session::clear_hover_edge_focus(&mut canvas.interaction);
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
    super::paint_invalidation::invalidate_paint(cx);
}
