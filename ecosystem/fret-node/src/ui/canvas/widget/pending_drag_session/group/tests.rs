use super::*;
use crate::core::{CanvasRect, GroupId};
use fret_core::Point;

#[test]
fn activate_pending_group_drag_moves_pending_into_active() {
    let pending = PendingGroupDrag {
        group: GroupId::from_u128(1),
        start_pos: Point::default(),
        start_rect: CanvasRect::default(),
    };
    let nodes = vec![(GraphNodeId::from_u128(2), CanvasPoint::default())];
    let mut interaction = InteractionState {
        pending_group_drag: Some(pending.clone()),
        ..Default::default()
    };

    activate_pending_group_drag(&mut interaction, pending.clone(), nodes.clone());

    assert!(interaction.pending_group_drag.is_none());
    let active = interaction.group_drag.expect("group drag active");
    assert_eq!(active.group, pending.group);
    assert_eq!(active.nodes, nodes);
    assert_eq!(active.current_nodes, nodes);
}
