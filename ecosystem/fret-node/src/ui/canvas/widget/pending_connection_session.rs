use crate::ui::canvas::state::{
    EdgeInsertDrag, InteractionState, PendingEdgeInsertDrag, PendingWireDrag, WireDrag,
    WireDragKind,
};
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

pub(super) fn activate_pending_wire_drag(
    interaction: &mut InteractionState,
    pending: PendingWireDrag,
) -> WireDragKind {
    interaction.pending_wire_drag = None;
    let kind = pending.kind.clone();
    interaction.wire_drag = Some(WireDrag {
        kind: pending.kind,
        pos: pending.start_pos,
    });
    kind
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{EdgeId, PortId};
    use crate::rules::EdgeEndpoint;

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

    #[test]
    fn activate_pending_wire_drag_moves_pending_into_active_and_returns_kind() {
        let pending = PendingWireDrag {
            kind: WireDragKind::Reconnect {
                edge: EdgeId::from_u128(1),
                endpoint: EdgeEndpoint::To,
                fixed: PortId::from_u128(2),
            },
            start_pos: Point::default(),
        };
        let mut interaction = InteractionState {
            pending_wire_drag: Some(pending.clone()),
            ..Default::default()
        };

        let kind = activate_pending_wire_drag(&mut interaction, pending.clone());

        assert!(interaction.pending_wire_drag.is_none());
        assert!(matches!(kind, WireDragKind::Reconnect { .. }));
        assert!(matches!(
            interaction.wire_drag,
            Some(WireDrag {
                kind: WireDragKind::Reconnect { .. },
                ..
            })
        ));
    }
}
