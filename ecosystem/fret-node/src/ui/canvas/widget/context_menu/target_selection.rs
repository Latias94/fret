use crate::core::GroupId;
use crate::ui::canvas::widget::*;

fn select_group_context_target_in_view_state(
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

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in crate::ui::canvas::widget) fn select_group_context_target<H: UiHost>(
        &mut self,
        host: &mut H,
        group_id: GroupId,
    ) {
        self.update_view_state(host, |view_state| {
            select_group_context_target_in_view_state(view_state, group_id);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{EdgeId, GroupId, NodeId as GraphNodeId};

    #[test]
    fn select_group_context_target_clears_node_and_edge_selection() {
        let group_id = GroupId::new();
        let mut view_state = NodeGraphViewState::default();
        view_state.selected_nodes.push(GraphNodeId::new());
        view_state.selected_edges.push(EdgeId::new());

        select_group_context_target_in_view_state(&mut view_state, group_id);

        assert!(view_state.selected_nodes.is_empty());
        assert!(view_state.selected_edges.is_empty());
        assert_eq!(view_state.selected_groups, vec![group_id]);
    }

    #[test]
    fn select_group_context_target_preserves_existing_group_if_already_selected() {
        let group_id = GroupId::new();
        let other_group = GroupId::new();
        let mut view_state = NodeGraphViewState::default();
        view_state.selected_groups.extend([group_id, other_group]);

        select_group_context_target_in_view_state(&mut view_state, group_id);

        assert_eq!(view_state.selected_groups, vec![group_id, other_group]);
    }
}
