use super::*;
use crate::core::{CanvasPoint, EdgeId};

fn sample_snapshot(selected_edges: Vec<EdgeId>) -> ViewSnapshot {
    ViewSnapshot {
        pan: CanvasPoint::default(),
        zoom: 1.0,
        selected_nodes: Vec::new(),
        selected_edges,
        selected_groups: Vec::new(),
        draw_order: Vec::new(),
        group_draw_order: Vec::new(),
        interaction: crate::io::NodeGraphInteractionState::default(),
    }
}

#[test]
fn hover_anchor_target_edge_prefers_focused_edge() {
    let focused = EdgeId::from_u128(1);
    let selected = EdgeId::from_u128(2);
    let mut interaction = InteractionState::default();
    interaction.focused_edge = Some(focused);

    let target = hover_anchor_target_edge(&interaction, &sample_snapshot(vec![selected]));

    assert_eq!(target, Some(focused));
}

#[test]
fn hover_anchor_target_edge_falls_back_to_single_selected_edge() {
    let selected = EdgeId::from_u128(2);

    let target = hover_anchor_target_edge(
        &InteractionState::default(),
        &sample_snapshot(vec![selected]),
    );

    assert_eq!(target, Some(selected));
}

#[test]
fn hover_anchor_target_edge_ignores_multi_selection() {
    let a = EdgeId::from_u128(1);
    let b = EdgeId::from_u128(2);

    let target =
        hover_anchor_target_edge(&InteractionState::default(), &sample_snapshot(vec![a, b]));

    assert_eq!(target, None);
}
