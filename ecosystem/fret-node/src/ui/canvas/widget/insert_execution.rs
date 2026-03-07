use super::*;

pub(super) fn is_reroute_insert_candidate(candidate: &InsertNodeCandidate) -> bool {
    candidate.kind.0 == REROUTE_KIND
}

pub(super) fn insert_candidate_canvas_point<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    candidate: &InsertNodeCandidate,
    invoked_at: Point,
) -> CanvasPoint {
    if is_reroute_insert_candidate(candidate) {
        canvas.reroute_pos_for_invoked_at(invoked_at)
    } else {
        CanvasPoint {
            x: invoked_at.x.0,
            y: invoked_at.y.0,
        }
    }
}

fn select_inserted_node_in_view_state(view_state: &mut NodeGraphViewState, node_id: GraphNodeId) {
    view_state.selected_edges.clear();
    view_state.selected_groups.clear();
    view_state.selected_nodes.clear();
    view_state.selected_nodes.push(node_id);
    view_state.draw_order.retain(|id| *id != node_id);
    view_state.draw_order.push(node_id);
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn plan_insert_candidate_ops_with_graph(
        presenter: &mut dyn NodeGraphPresenter,
        graph: &Graph,
        candidate: &InsertNodeCandidate,
        at: CanvasPoint,
    ) -> Result<Vec<GraphOp>, Arc<str>> {
        if is_reroute_insert_candidate(candidate) {
            Ok(Self::build_reroute_create_ops(at))
        } else {
            presenter.plan_create_node(graph, candidate, at)
        }
    }

    pub(super) fn plan_canvas_insert_candidate_ops<H: UiHost>(
        &mut self,
        host: &mut H,
        candidate: &InsertNodeCandidate,
        at: CanvasPoint,
    ) -> Option<Result<Vec<GraphOp>, Arc<str>>> {
        let presenter = &mut *self.presenter;
        self.graph
            .read_ref(host, |graph| {
                Self::plan_insert_candidate_ops_with_graph(presenter, graph, candidate, at)
            })
            .ok()
    }

    pub(super) fn select_inserted_node<H: UiHost>(
        &mut self,
        host: &mut H,
        node_id: Option<GraphNodeId>,
    ) {
        if let Some(node_id) = node_id {
            self.update_view_state(host, |view_state| {
                select_inserted_node_in_view_state(view_state, node_id);
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::insert_candidates::reroute_insert_candidate;
    use super::*;
    use crate::core::{EdgeId, GroupId, NodeKindKey};
    use serde_json::Value;

    fn regular_candidate() -> InsertNodeCandidate {
        InsertNodeCandidate {
            kind: NodeKindKey::new("regular"),
            label: Arc::<str>::from("Regular"),
            enabled: true,
            template: None,
            payload: Value::Null,
        }
    }

    #[test]
    fn reroute_insert_candidate_detection_is_kind_based() {
        assert!(is_reroute_insert_candidate(&reroute_insert_candidate()));
        assert!(!is_reroute_insert_candidate(&regular_candidate()));
    }

    #[test]
    fn select_inserted_node_clears_other_selection_kinds() {
        let node_id = GraphNodeId::new();
        let existing_node = GraphNodeId::new();
        let edge_id = EdgeId::new();
        let group_id = GroupId::new();
        let mut view_state = NodeGraphViewState::default();
        view_state.selected_nodes.push(existing_node);
        view_state.selected_edges.push(edge_id);
        view_state.selected_groups.push(group_id);

        select_inserted_node_in_view_state(&mut view_state, node_id);

        assert_eq!(view_state.selected_nodes, vec![node_id]);
        assert!(view_state.selected_edges.is_empty());
        assert!(view_state.selected_groups.is_empty());
    }

    #[test]
    fn select_inserted_node_moves_node_to_draw_order_tail() {
        let node_id = GraphNodeId::new();
        let other = GraphNodeId::new();
        let mut view_state = NodeGraphViewState::default();
        view_state.draw_order.extend([node_id, other]);

        select_inserted_node_in_view_state(&mut view_state, node_id);

        assert_eq!(view_state.draw_order, vec![other, node_id]);
    }
}
