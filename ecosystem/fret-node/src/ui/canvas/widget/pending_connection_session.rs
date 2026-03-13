mod edge;
mod wire;

use crate::ui::canvas::state::{
    InteractionState, PendingEdgeInsertDrag, PendingWireDrag, WireDragKind,
};
use fret_core::Point;

pub(super) fn activate_pending_edge_insert_drag(
    interaction: &mut InteractionState,
    pending: PendingEdgeInsertDrag,
    position: Point,
) {
    edge::activate_pending_edge_insert_drag(interaction, pending, position)
}

pub(super) fn activate_pending_wire_drag(
    interaction: &mut InteractionState,
    pending: PendingWireDrag,
) -> WireDragKind {
    wire::activate_pending_wire_drag(interaction, pending)
}
