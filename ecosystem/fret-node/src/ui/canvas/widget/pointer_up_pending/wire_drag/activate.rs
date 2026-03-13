use fret_core::Point;

use super::super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::{PendingWireDrag, ViewSnapshot, WireDrag};

pub(super) fn promote_pending_wire_drag<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    pending: PendingWireDrag,
    position: Point,
) {
    let kind = pending.kind.clone();
    canvas.interaction.wire_drag = Some(WireDrag {
        kind: pending.kind,
        pos: position,
    });
    canvas.interaction.click_connect = true;
    canvas.emit_connect_start(snapshot, &kind);
}
