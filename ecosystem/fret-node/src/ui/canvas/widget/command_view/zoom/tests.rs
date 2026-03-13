use super::*;

#[test]
fn reset_view_state_only_resets_pan_and_zoom() {
    let node_id = GraphNodeId::from_u128(1);
    let edge_id = EdgeId::from_u128(2);
    let group_id = crate::core::GroupId::from_u128(3);

    let mut view_state = NodeGraphViewState::default();
    view_state.pan = CanvasPoint { x: 12.0, y: -7.0 };
    view_state.zoom = 2.5;
    view_state.selected_nodes = vec![node_id];
    view_state.selected_edges = vec![edge_id];
    view_state.selected_groups = vec![group_id];

    reset_view_state(&mut view_state);

    assert_eq!(view_state.pan, CanvasPoint::default());
    assert_eq!(view_state.zoom, 1.0);
    assert_eq!(view_state.selected_nodes, vec![node_id]);
    assert_eq!(view_state.selected_edges, vec![edge_id]);
    assert_eq!(view_state.selected_groups, vec![group_id]);
}

#[test]
fn zoom_command_factor_matches_expected_direction() {
    assert_eq!(zoom_command_factor(true), 1.2);
    assert_eq!(zoom_command_factor(false), 1.0 / 1.2);
}

#[test]
fn apply_cached_viewport_writes_pan_and_zoom() {
    let mut view_state = NodeGraphViewState::default();
    let pan = CanvasPoint { x: 3.0, y: 4.0 };

    apply_cached_viewport(&mut view_state, pan, 1.75);

    assert_eq!(view_state.pan, pan);
    assert_eq!(view_state.zoom, 1.75);
}
