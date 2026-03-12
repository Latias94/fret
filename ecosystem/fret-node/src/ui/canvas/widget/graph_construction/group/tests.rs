use super::*;

#[test]
fn centered_group_origin_offsets_half_size() {
    let size = default_group_size();
    let origin = centered_group_origin(CanvasPoint { x: 100.0, y: 200.0 }, size);

    assert_eq!(
        origin,
        CanvasPoint {
            x: 100.0 - 0.5 * GROUP_WIDTH,
            y: 200.0 - 0.5 * GROUP_HEIGHT,
        }
    );
}

#[test]
fn create_group_ops_builds_single_add_group_op() {
    let (group_id, ops) = create_group_ops(CanvasPoint { x: 100.0, y: 200.0 });

    let [GraphOp::AddGroup { id, group }] = ops.as_slice() else {
        panic!("expected one add group op");
    };
    assert_eq!(*id, group_id);
    assert_eq!(group.title, "Group");
    assert_eq!(group.rect.size, default_group_size());
    assert_eq!(
        group.rect.origin,
        CanvasPoint {
            x: 100.0 - 0.5 * GROUP_WIDTH,
            y: 200.0 - 0.5 * GROUP_HEIGHT,
        }
    );
}

#[test]
fn select_created_group_in_view_state_clears_other_selection_kinds() {
    let group_id = crate::core::GroupId::from_u128(9);
    let mut view_state = NodeGraphViewState::default();
    view_state.selected_nodes.push(GraphNodeId::from_u128(1));
    view_state.selected_edges.push(EdgeId::from_u128(2));
    view_state
        .selected_groups
        .push(crate::core::GroupId::from_u128(3));
    view_state.group_draw_order.extend([
        crate::core::GroupId::from_u128(1),
        group_id,
        crate::core::GroupId::from_u128(2),
    ]);

    select_created_group_in_view_state(&mut view_state, group_id);

    assert!(view_state.selected_nodes.is_empty());
    assert!(view_state.selected_edges.is_empty());
    assert_eq!(view_state.selected_groups, vec![group_id]);
    assert_eq!(
        view_state.group_draw_order,
        vec![
            crate::core::GroupId::from_u128(1),
            crate::core::GroupId::from_u128(2),
            group_id,
        ]
    );
}
