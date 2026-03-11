mod edge;
mod group;

use crate::core::{EdgeId, GroupId};
use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in crate::ui::canvas::widget) fn select_group_context_target<H: UiHost>(
        &mut self,
        host: &mut H,
        group_id: GroupId,
    ) {
        self.update_view_state(host, |view_state| {
            group::select_group_context_target_in_view_state(view_state, group_id);
        });
    }

    pub(in crate::ui::canvas::widget) fn select_edge_context_target<H: UiHost>(
        &mut self,
        host: &mut H,
        edge_id: EdgeId,
    ) {
        self.update_view_state(host, |view_state| {
            edge::select_edge_context_target_in_view_state(view_state, edge_id);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{GroupId, NodeId as GraphNodeId};

    #[test]
    fn select_group_context_target_clears_node_and_edge_selection() {
        let group_id = GroupId::new();
        let mut view_state = NodeGraphViewState::default();
        view_state.selected_nodes.push(GraphNodeId::new());
        view_state.selected_edges.push(EdgeId::new());

        group::select_group_context_target_in_view_state(&mut view_state, group_id);

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

        group::select_group_context_target_in_view_state(&mut view_state, group_id);

        assert_eq!(view_state.selected_groups, vec![group_id, other_group]);
    }

    #[test]
    fn select_edge_context_target_clears_node_and_group_selection() {
        let edge_id = EdgeId::new();
        let mut view_state = NodeGraphViewState::default();
        view_state.selected_nodes.push(GraphNodeId::new());
        view_state.selected_groups.push(GroupId::new());

        edge::select_edge_context_target_in_view_state(&mut view_state, edge_id);

        assert!(view_state.selected_nodes.is_empty());
        assert!(view_state.selected_groups.is_empty());
        assert_eq!(view_state.selected_edges, vec![edge_id]);
    }

    #[test]
    fn select_edge_context_target_preserves_existing_edge_if_already_selected() {
        let edge_id = EdgeId::new();
        let other_edge = EdgeId::new();
        let mut view_state = NodeGraphViewState::default();
        view_state.selected_edges.extend([edge_id, other_edge]);

        edge::select_edge_context_target_in_view_state(&mut view_state, edge_id);

        assert_eq!(view_state.selected_edges, vec![edge_id, other_edge]);
    }
}
