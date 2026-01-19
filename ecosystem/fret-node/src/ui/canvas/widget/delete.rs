use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn delete_selection_ops(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        selected_nodes: &[GraphNodeId],
        selected_edges: &[EdgeId],
        selected_groups: &[crate::core::GroupId],
    ) -> Vec<GraphOp> {
        let mut ops: Vec<GraphOp> = Vec::new();
        let mut removed_edges: std::collections::BTreeSet<EdgeId> =
            std::collections::BTreeSet::new();

        let mut groups: Vec<crate::core::GroupId> = selected_groups.to_vec();
        groups.sort();
        for group_id in groups {
            if let Some(op) = graph.build_remove_group_op(group_id) {
                ops.push(op);
            }
        }

        let mut nodes: Vec<GraphNodeId> = selected_nodes.to_vec();
        nodes.sort();

        for node_id in nodes {
            if !Self::node_is_deletable(graph, interaction, node_id) {
                continue;
            }
            let Some(node) = graph.nodes.get(&node_id) else {
                continue;
            };

            let mut ports: Vec<(PortId, crate::core::Port)> = graph
                .ports
                .iter()
                .filter_map(|(port_id, port)| {
                    (port.node == node_id).then_some((*port_id, port.clone()))
                })
                .collect();
            ports.sort_by_key(|(id, _)| *id);

            let port_ids: std::collections::BTreeSet<PortId> =
                ports.iter().map(|(id, _)| *id).collect();

            let mut edges: Vec<(EdgeId, Edge)> = graph
                .edges
                .iter()
                .filter_map(|(edge_id, edge)| {
                    if port_ids.contains(&edge.from) || port_ids.contains(&edge.to) {
                        Some((*edge_id, edge.clone()))
                    } else {
                        None
                    }
                })
                .collect();
            edges.sort_by_key(|(id, _)| *id);
            edges.retain(|(id, _)| removed_edges.insert(*id));

            ops.push(GraphOp::RemoveNode {
                id: node_id,
                node: node.clone(),
                ports,
                edges,
            });
        }

        let mut edges_sel: Vec<EdgeId> = selected_edges.to_vec();
        edges_sel.sort();
        for edge_id in edges_sel {
            if removed_edges.contains(&edge_id) {
                continue;
            }
            if !Self::edge_is_deletable(graph, interaction, edge_id) {
                continue;
            }
            let Some(edge) = graph.edges.get(&edge_id) else {
                continue;
            };
            ops.push(GraphOp::RemoveEdge {
                id: edge_id,
                edge: edge.clone(),
            });
        }

        ops
    }

    pub(super) fn removed_ids_from_ops(
        ops: &[GraphOp],
    ) -> (
        HashSet<GraphNodeId>,
        HashSet<EdgeId>,
        HashSet<crate::core::GroupId>,
    ) {
        let mut removed_nodes: HashSet<GraphNodeId> = HashSet::new();
        let mut removed_edges: HashSet<EdgeId> = HashSet::new();
        let mut removed_groups: HashSet<crate::core::GroupId> = HashSet::new();

        for op in ops {
            match op {
                GraphOp::RemoveNode { id, edges, .. } => {
                    removed_nodes.insert(*id);
                    for (edge_id, _) in edges {
                        removed_edges.insert(*edge_id);
                    }
                }
                GraphOp::RemoveEdge { id, .. } => {
                    removed_edges.insert(*id);
                }
                GraphOp::RemoveGroup { id, .. } => {
                    removed_groups.insert(*id);
                }
                _ => {}
            }
        }

        (removed_nodes, removed_edges, removed_groups)
    }

    pub(super) fn edge_is_deletable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        edge: EdgeId,
    ) -> bool {
        if !interaction.edges_deletable {
            return false;
        }
        let Some(edge) = graph.edges.get(&edge) else {
            return false;
        };
        edge.deletable.unwrap_or(true)
    }

    pub(super) fn node_is_deletable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        node: GraphNodeId,
    ) -> bool {
        if !interaction.nodes_deletable {
            return false;
        }
        let Some(node) = graph.nodes.get(&node) else {
            return false;
        };
        node.deletable.unwrap_or(true)
    }
}
