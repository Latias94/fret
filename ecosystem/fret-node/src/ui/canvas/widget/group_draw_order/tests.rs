use super::*;

#[test]
fn bring_to_front_preserves_existing_draw_order_for_selected_groups() {
    let ids = super::test_support::group_ids(4);
    let mut view_state = NodeGraphViewState::default();
    view_state.group_draw_order = ids.clone();

    bring_selected_groups_to_front_in_view_state(&mut view_state, &[ids[2], ids[0]]);

    assert_eq!(
        view_state.group_draw_order,
        vec![ids[1], ids[3], ids[0], ids[2]]
    );
}

#[test]
fn bring_to_front_appends_missing_selected_groups() {
    let ids = super::test_support::group_ids(3);
    let extra = GroupId::new();
    let mut view_state = NodeGraphViewState::default();
    view_state.group_draw_order = ids.clone();

    bring_selected_groups_to_front_in_view_state(&mut view_state, &[ids[1], extra]);

    assert_eq!(
        view_state.group_draw_order,
        vec![ids[0], ids[2], ids[1], extra]
    );
}

#[test]
fn send_to_back_preserves_existing_draw_order_for_selected_groups() {
    let ids = super::test_support::group_ids(4);
    let mut view_state = NodeGraphViewState::default();
    view_state.group_draw_order = ids.clone();

    send_selected_groups_to_back_in_view_state(&mut view_state, &[ids[2], ids[0]]);

    assert_eq!(
        view_state.group_draw_order,
        vec![ids[0], ids[2], ids[1], ids[3]]
    );
}
