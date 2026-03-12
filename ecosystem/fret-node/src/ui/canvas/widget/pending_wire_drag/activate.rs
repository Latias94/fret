use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::{PendingWireDrag, ViewSnapshot};

pub(super) fn activate_pending_wire_drag<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    pending: PendingWireDrag,
) {
    let kind = super::super::pending_connection_session::activate_pending_wire_drag(
        &mut canvas.interaction,
        pending,
    );
    canvas.emit_connect_start(snapshot, &kind);
}
