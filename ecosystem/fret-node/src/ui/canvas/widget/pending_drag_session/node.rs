use fret_ui::UiHost;

use crate::core::{CanvasPoint, NodeId as GraphNodeId};
use crate::ui::canvas::state::{InteractionState, NodeDrag, PendingNodeDrag};

pub(in super::super) fn abort_pending_node_drag<H: UiHost>(
    interaction: &mut InteractionState,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    if interaction.pending_node_drag.take().is_none() {
        return false;
    }

    super::super::pointer_up_finish::finish_pointer_up(cx);
    true
}

pub(in super::super) fn activate_pending_node_drag(
    interaction: &mut InteractionState,
    pending: PendingNodeDrag,
    drag_nodes: Vec<GraphNodeId>,
    start_nodes: Vec<(GraphNodeId, CanvasPoint)>,
) {
    interaction.pending_node_drag = None;
    interaction.node_drag = Some(NodeDrag {
        primary: pending.primary,
        node_ids: drag_nodes,
        nodes: start_nodes.clone(),
        current_nodes: start_nodes,
        current_groups: Vec::new(),
        preview_rev: 0,
        grab_offset: pending.grab_offset,
        start_pos: pending.start_pos,
    });
}

#[cfg(test)]
mod tests;
