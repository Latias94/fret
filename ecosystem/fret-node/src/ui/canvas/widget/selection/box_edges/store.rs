use std::collections::BTreeSet;

use super::super::super::*;

fn collect_box_select_edges_from_store_with_mode(
    store: &Model<NodeGraphStore>,
    host: &mut impl UiHost,
    mode: crate::io::NodeGraphBoxSelectEdges,
    nodes: &BTreeSet<GraphNodeId>,
) -> Option<Vec<EdgeId>> {
    store
        .read_ref(host, |state| {
            let graph = state.graph();
            let mut out = BTreeSet::new();
            for &node in nodes {
                let Some(connections) = state.lookups().connections_for_node(node) else {
                    continue;
                };
                for (edge_id, connection) in connections {
                    let Some(edge) = graph.edges.get(edge_id) else {
                        continue;
                    };
                    if !edge.selectable.unwrap_or(true) {
                        continue;
                    }
                    if super::mode::edge_matches_box_select_mode(
                        mode,
                        nodes,
                        connection.source_node,
                        connection.target_node,
                    ) {
                        out.insert(*edge_id);
                    }
                }
            }
            out.into_iter().collect::<Vec<_>>()
        })
        .ok()
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super::super) fn box_select_edges_for_nodes<H: UiHost>(
        &self,
        host: &mut H,
        interaction: &NodeGraphInteractionState,
        nodes: &BTreeSet<GraphNodeId>,
    ) -> Vec<EdgeId> {
        let Some(mode) = super::mode::box_select_edge_mode(interaction) else {
            return Vec::new();
        };

        if let Some(store) = self.store.as_ref() {
            if let Some(out) =
                collect_box_select_edges_from_store_with_mode(store, host, mode, nodes)
            {
                return out;
            }
        }

        self.graph
            .read_ref(host, |graph| {
                super::graph::collect_box_select_edges_from_graph_with_mode(graph, mode, nodes)
            })
            .ok()
            .unwrap_or_default()
    }
}
