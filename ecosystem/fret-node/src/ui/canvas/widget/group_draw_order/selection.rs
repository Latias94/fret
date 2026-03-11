use crate::core::GroupId;

pub(super) fn selected_groups_in_draw_order(
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
