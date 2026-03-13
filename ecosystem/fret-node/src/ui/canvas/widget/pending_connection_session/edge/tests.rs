use super::*;
use crate::core::EdgeId;

#[test]
fn activate_pending_edge_insert_drag_moves_pending_into_active() {
    let pending = PendingEdgeInsertDrag {
        edge: EdgeId::from_u128(1),
        start_pos: Point::default(),
    };
    let mut interaction = InteractionState {
        pending_edge_insert_drag: Some(pending.clone()),
        ..Default::default()
    };

    activate_pending_edge_insert_drag(&mut interaction, pending.clone(), Point::default());

    assert!(interaction.pending_edge_insert_drag.is_none());
    let active = interaction.edge_insert_drag.expect("edge insert active");
    assert_eq!(active.edge, pending.edge);
}
