use crate::core::EdgeId;
use crate::ui::canvas::widget::*;

pub(super) fn select_edge_context_target_in_view_state(
    view_state: &mut NodeGraphViewState,
    edge_id: EdgeId,
) {
    view_state.selected_nodes.clear();
    view_state.selected_groups.clear();
    if !view_state.selected_edges.iter().any(|id| *id == edge_id) {
        view_state.selected_edges.clear();
        view_state.selected_edges.push(edge_id);
    }
}
