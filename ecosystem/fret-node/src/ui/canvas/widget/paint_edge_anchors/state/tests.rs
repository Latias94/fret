use super::{EdgeAnchorInteractionState, edge_anchor_interaction_state};
use crate::core::{EdgeId, PortId};
use crate::rules::EdgeEndpoint;
use crate::ui::canvas::state::{InteractionState, WireDrag, WireDragKind};
use fret_core::Point;

#[test]
fn edge_anchor_interaction_state_tracks_hover_and_active_reconnect() {
    let edge = EdgeId::new();
    let interaction = InteractionState {
        hover_edge_anchor: Some((edge, EdgeEndpoint::From)),
        wire_drag: Some(WireDrag {
            kind: WireDragKind::Reconnect {
                edge,
                endpoint: EdgeEndpoint::From,
                fixed: PortId::new(),
            },
            pos: Point::default(),
        }),
        ..Default::default()
    };

    let state = edge_anchor_interaction_state(&interaction, Some(edge), EdgeEndpoint::From);
    assert_eq!(
        state,
        EdgeAnchorInteractionState {
            hovered: true,
            active: true,
        }
    );
}

#[test]
fn edge_anchor_interaction_state_ignores_other_edges() {
    let interaction = InteractionState {
        hover_edge_anchor: Some((EdgeId::new(), EdgeEndpoint::From)),
        ..Default::default()
    };

    let state =
        edge_anchor_interaction_state(&interaction, Some(EdgeId::new()), EdgeEndpoint::From);
    assert_eq!(
        state,
        EdgeAnchorInteractionState {
            hovered: false,
            active: false,
        }
    );
}
