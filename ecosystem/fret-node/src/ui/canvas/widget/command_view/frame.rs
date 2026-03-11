use super::super::*;

fn graph_node_ids(graph: &Graph) -> Vec<GraphNodeId> {
    graph.nodes.keys().copied().collect()
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn cmd_frame_selection<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let did = self.frame_nodes_in_view(cx.app, cx.window, bounds, &snapshot.selected_nodes);
        super::super::command_ui::finish_command_paint_if(cx, did)
    }

    pub(in super::super) fn cmd_frame_all<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let nodes = self
            .graph
            .read_ref(cx.app, graph_node_ids)
            .ok()
            .unwrap_or_default();
        let did = self.frame_nodes_in_view(cx.app, cx.window, bounds, &nodes);
        super::super::command_ui::finish_command_paint_if(cx, did)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{CanvasPoint, GraphId, Node, NodeKindKey};
    use serde_json::Value;

    #[test]
    fn graph_node_ids_follow_graph_key_order() {
        let mut graph = Graph::new(GraphId::from_u128(1));
        let a = GraphNodeId::from_u128(30);
        let b = GraphNodeId::from_u128(10);
        let c = GraphNodeId::from_u128(20);

        for id in [a, b, c] {
            graph.nodes.insert(
                id,
                Node {
                    kind: NodeKindKey::new("test.node"),
                    kind_version: 1,
                    pos: CanvasPoint::default(),
                    selectable: None,
                    draggable: None,
                    connectable: None,
                    deletable: None,
                    parent: None,
                    extent: None,
                    expand_parent: None,
                    size: None,
                    hidden: false,
                    collapsed: false,
                    ports: Vec::new(),
                    data: Value::Null,
                },
            );
        }

        assert_eq!(graph_node_ids(&graph), vec![b, c, a]);
    }
}
