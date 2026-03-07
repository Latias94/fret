use crate::core::GroupId;
use crate::io::NodeGraphViewState;

fn selected_groups_in_draw_order(
    group_draw_order: &[GroupId],
    selected_groups: &[GroupId],
) -> Vec<GroupId> {
    let mut selected_in_order = Vec::new();
    for group_id in group_draw_order {
        if selected_groups.contains(group_id) {
            selected_in_order.push(*group_id);
        }
    }
    for group_id in selected_groups {
        if !selected_in_order.contains(group_id) {
            selected_in_order.push(*group_id);
        }
    }
    selected_in_order
}

pub(super) fn bring_selected_groups_to_front_in_view_state(
    view_state: &mut NodeGraphViewState,
    selected_groups: &[GroupId],
) {
    let selected_in_order =
        selected_groups_in_draw_order(&view_state.group_draw_order, selected_groups);
    view_state
        .group_draw_order
        .retain(|group_id| !selected_groups.contains(group_id));
    view_state.group_draw_order.extend(selected_in_order);
}

pub(super) fn send_selected_groups_to_back_in_view_state(
    view_state: &mut NodeGraphViewState,
    selected_groups: &[GroupId],
) {
    let selected_in_order =
        selected_groups_in_draw_order(&view_state.group_draw_order, selected_groups);
    view_state
        .group_draw_order
        .retain(|group_id| !selected_groups.contains(group_id));
    let mut next = selected_in_order;
    next.extend_from_slice(&view_state.group_draw_order);
    view_state.group_draw_order = next;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn group_ids(count: usize) -> Vec<GroupId> {
        (0..count).map(|_| GroupId::new()).collect()
    }

    #[test]
    fn bring_to_front_preserves_existing_draw_order_for_selected_groups() {
        let ids = group_ids(4);
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
        let ids = group_ids(3);
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
        let ids = group_ids(4);
        let mut view_state = NodeGraphViewState::default();
        view_state.group_draw_order = ids.clone();

        send_selected_groups_to_back_in_view_state(&mut view_state, &[ids[2], ids[0]]);

        assert_eq!(
            view_state.group_draw_order,
            vec![ids[0], ids[2], ids[1], ids[3]]
        );
    }
}
