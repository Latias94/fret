use crate::ui::canvas::state::{EdgeInsertDrag, InteractionState, PendingEdgeInsertDrag};
use fret_core::Point;

pub(super) fn activate_pending_edge_insert_drag(
    interaction: &mut InteractionState,
    pending: PendingEdgeInsertDrag,
    position: Point,
) {
    interaction.pending_edge_insert_drag = None;
    interaction.edge_insert_drag = Some(EdgeInsertDrag {
        edge: pending.edge,
        pos: position,
    });
}

#[cfg(test)]
mod tests;
