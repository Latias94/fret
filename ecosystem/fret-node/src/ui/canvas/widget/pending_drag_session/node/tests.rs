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
