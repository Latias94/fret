use crate::core::GroupId;
use crate::io::NodeGraphViewState;

pub(super) fn bring_selected_groups_to_front_in_view_state(
    view_state: &mut NodeGraphViewState,
    selected_groups: &[GroupId],
    selected_in_order: Vec<GroupId>,
) {
    view_state
        .group_draw_order
        .retain(|group_id| !selected_groups.contains(group_id));
    view_state.group_draw_order.extend(selected_in_order);
}

pub(super) fn send_selected_groups_to_back_in_view_state(
    view_state: &mut NodeGraphViewState,
    selected_groups: &[GroupId],
    selected_in_order: Vec<GroupId>,
) {
    view_state
        .group_draw_order
        .retain(|group_id| !selected_groups.contains(group_id));
    let mut next = selected_in_order;
    next.extend_from_slice(&view_state.group_draw_order);
    view_state.group_draw_order = next;
}
