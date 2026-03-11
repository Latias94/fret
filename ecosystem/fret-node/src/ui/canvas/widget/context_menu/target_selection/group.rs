use crate::core::GroupId;
use crate::ui::canvas::widget::*;

pub(super) fn select_group_context_target_in_view_state(
    view_state: &mut NodeGraphViewState,
    group_id: GroupId,
) {
    view_state.selected_nodes.clear();
    view_state.selected_edges.clear();
    if !view_state.selected_groups.iter().any(|id| *id == group_id) {
        view_state.selected_groups.clear();
        view_state.selected_groups.push(group_id);
    }
}
