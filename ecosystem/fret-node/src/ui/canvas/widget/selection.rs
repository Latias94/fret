use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn box_select_edges_for_nodes<H: UiHost>(
        &self,
        host: &mut H,
        interaction: &NodeGraphInteractionState,
        nodes: &std::collections::BTreeSet<GraphNodeId>,
    ) -> Vec<EdgeId> {
        if !interaction.elements_selectable || !interaction.edges_selectable {
            return Vec::new();
        }

        match interaction.box_select_edges {
            crate::io::NodeGraphBoxSelectEdges::None => return Vec::new(),
            crate::io::NodeGraphBoxSelectEdges::Connected
            | crate::io::NodeGraphBoxSelectEdges::BothEndpoints => {}
        }

        if let Some(store) = self.store.as_ref() {
            if let Ok(out) = store.read_ref(host, |s| {
                let graph = s.graph();
                let mut out: std::collections::BTreeSet<EdgeId> = std::collections::BTreeSet::new();
                for &node in nodes {
                    let Some(conns) = s.lookups().connections_for_node(node) else {
                        continue;
                    };
                    for (edge_id, conn) in conns {
                        let Some(edge) = graph.edges.get(edge_id) else {
                            continue;
                        };
                        if !edge.selectable.unwrap_or(true) {
                            continue;
                        }
                        match interaction.box_select_edges {
                            crate::io::NodeGraphBoxSelectEdges::None => {}
                            crate::io::NodeGraphBoxSelectEdges::Connected => {
                                out.insert(*edge_id);
                            }
                            crate::io::NodeGraphBoxSelectEdges::BothEndpoints => {
                                if nodes.contains(&conn.source_node)
                                    && nodes.contains(&conn.target_node)
                                {
                                    out.insert(*edge_id);
                                }
                            }
                        }
                    }
                }
                out.into_iter().collect::<Vec<_>>()
            }) {
                return out;
            }
        }

        self.graph
            .read_ref(host, |graph| {
                graph
                    .edges
                    .iter()
                    .filter_map(|(edge_id, edge)| {
                        if !edge.selectable.unwrap_or(true) {
                            return None;
                        }
                        let from_node = graph.ports.get(&edge.from).map(|p| p.node)?;
                        let to_node = graph.ports.get(&edge.to).map(|p| p.node)?;
                        match interaction.box_select_edges {
                            crate::io::NodeGraphBoxSelectEdges::None => None,
                            crate::io::NodeGraphBoxSelectEdges::Connected => {
                                (nodes.contains(&from_node) || nodes.contains(&to_node))
                                    .then_some(*edge_id)
                            }
                            crate::io::NodeGraphBoxSelectEdges::BothEndpoints => {
                                (nodes.contains(&from_node) && nodes.contains(&to_node))
                                    .then_some(*edge_id)
                            }
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .ok()
            .unwrap_or_default()
    }

    pub(super) fn edge_is_selectable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        edge: EdgeId,
    ) -> bool {
        if !interaction.elements_selectable || !interaction.edges_selectable {
            return false;
        }
        let Some(edge) = graph.edges.get(&edge) else {
            return false;
        };
        edge.selectable.unwrap_or(true)
    }

    pub(super) fn node_is_selectable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        node: GraphNodeId,
    ) -> bool {
        if !interaction.elements_selectable {
            return false;
        }
        let Some(node) = graph.nodes.get(&node) else {
            return false;
        };
        node.selectable.unwrap_or(true)
    }
}
