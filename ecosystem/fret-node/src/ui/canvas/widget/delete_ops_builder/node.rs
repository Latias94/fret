use std::collections::BTreeSet;

use super::super::*;

pub(in super::super) fn push_node_remove_ops(
    ops: &mut Vec<GraphOp>,
    removed_edges: &mut BTreeSet<EdgeId>,
    graph: &Graph,
    interaction: &NodeGraphInteractionState,
    selected_nodes: &[GraphNodeId],
) {
    let mut nodes: Vec<GraphNodeId> = selected_nodes.to_vec();
    nodes.sort();

    for node_id in nodes {
        if !super::super::delete_predicates::node_is_deletable(graph, interaction, node_id) {
            continue;
        }
        let Some(node) = graph.nodes.get(&node_id) else {
            continue;
        };

        let ports = collect_node_ports(graph, node_id);
        let edges = collect_node_edges(graph, removed_edges, &ports);
        ops.push(GraphOp::RemoveNode {
            id: node_id,
            node: node.clone(),
            ports,
            edges,
        });
    }
}

fn collect_node_ports(graph: &Graph, node_id: GraphNodeId) -> Vec<(PortId, crate::core::Port)> {
    let mut ports: Vec<(PortId, crate::core::Port)> = graph
        .ports
        .iter()
        .filter_map(|(port_id, port)| (port.node == node_id).then_some((*port_id, port.clone())))
        .collect();
    ports.sort_by_key(|(id, _)| *id);
    ports
}

fn collect_node_edges(
    graph: &Graph,
    removed_edges: &mut BTreeSet<EdgeId>,
    ports: &[(PortId, crate::core::Port)],
) -> Vec<(EdgeId, Edge)> {
    let port_ids: BTreeSet<PortId> = ports.iter().map(|(id, _)| *id).collect();
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
    edges
}

#[cfg(test)]
mod tests;
