use crate::core::{CanvasPoint, NodeId as GraphNodeId};
use crate::ui::canvas::state::{GroupDrag, InteractionState, PendingGroupDrag};

pub(in super::super) fn activate_pending_group_drag(
    interaction: &mut InteractionState,
    pending: PendingGroupDrag,
    nodes: Vec<(GraphNodeId, CanvasPoint)>,
) {
    interaction.pending_group_drag = None;
    interaction.group_drag = Some(GroupDrag {
        group: pending.group,
        start_pos: pending.start_pos,
        start_rect: pending.start_rect,
        nodes: nodes.clone(),
        current_rect: pending.start_rect,
        current_nodes: nodes,
        preview_rev: 0,
    });
}

#[cfg(test)]
mod tests;
