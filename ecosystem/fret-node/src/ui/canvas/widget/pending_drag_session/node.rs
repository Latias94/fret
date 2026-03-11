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
mod tests {
    use super::*;
    use crate::ui::canvas::state::PendingNodeSelectAction;
    use fret_core::Point;

    #[test]
    fn activate_pending_node_drag_moves_pending_into_active() {
        let pending = PendingNodeDrag {
            primary: GraphNodeId::from_u128(1),
            nodes: vec![GraphNodeId::from_u128(1)],
            grab_offset: Point::default(),
            start_pos: Point::default(),
            select_action: PendingNodeSelectAction::None,
            drag_enabled: true,
        };
        let nodes = vec![(GraphNodeId::from_u128(1), CanvasPoint::default())];
        let drag_nodes = vec![GraphNodeId::from_u128(1)];
        let mut interaction = InteractionState {
            pending_node_drag: Some(pending.clone()),
            ..Default::default()
        };

        activate_pending_node_drag(
            &mut interaction,
            pending.clone(),
            drag_nodes.clone(),
            nodes.clone(),
        );

        assert!(interaction.pending_node_drag.is_none());
        let active = interaction.node_drag.expect("node drag active");
        assert_eq!(active.primary, pending.primary);
        assert_eq!(active.node_ids, drag_nodes);
        assert_eq!(active.nodes, nodes);
    }
}
