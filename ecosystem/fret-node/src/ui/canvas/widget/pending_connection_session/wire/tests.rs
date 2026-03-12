use super::*;
use crate::core::{EdgeId, PortId};
use crate::rules::EdgeEndpoint;

#[test]
fn activate_pending_wire_drag_moves_pending_into_active_and_returns_kind() {
    let pending = PendingWireDrag {
        kind: WireDragKind::Reconnect {
            edge: EdgeId::from_u128(1),
            endpoint: EdgeEndpoint::To,
            fixed: PortId::from_u128(2),
        },
        start_pos: Default::default(),
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
