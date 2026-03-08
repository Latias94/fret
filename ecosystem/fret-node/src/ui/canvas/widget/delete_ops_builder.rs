use super::*;

pub(super) fn delete_selection_ops(
    graph: &Graph,
    interaction: &NodeGraphInteractionState,
    selected_nodes: &[GraphNodeId],
    selected_edges: &[EdgeId],
    selected_groups: &[crate::core::GroupId],
) -> Vec<GraphOp> {
    let mut ops: Vec<GraphOp> = Vec::new();
    let mut removed_edges: std::collections::BTreeSet<EdgeId> = std::collections::BTreeSet::new();

    push_group_remove_ops(&mut ops, graph, selected_groups);
    push_node_remove_ops(
        &mut ops,
        &mut removed_edges,
        graph,
        interaction,
        selected_nodes,
    );
    push_edge_remove_ops(&mut ops, &removed_edges, graph, interaction, selected_edges);
    ops
}

fn push_group_remove_ops(
    ops: &mut Vec<GraphOp>,
    graph: &Graph,
    selected_groups: &[crate::core::GroupId],
) {
    let mut groups: Vec<crate::core::GroupId> = selected_groups.to_vec();
    groups.sort();
    for group_id in groups {
        if let Some(op) = graph.build_remove_group_op(group_id) {
            ops.push(op);
        }
    }
}

fn push_node_remove_ops(
    ops: &mut Vec<GraphOp>,
    removed_edges: &mut std::collections::BTreeSet<EdgeId>,
    graph: &Graph,
    interaction: &NodeGraphInteractionState,
    selected_nodes: &[GraphNodeId],
) {
    let mut nodes: Vec<GraphNodeId> = selected_nodes.to_vec();
    nodes.sort();

    for node_id in nodes {
        if !super::delete_predicates::node_is_deletable(graph, interaction, node_id) {
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
    removed_edges: &mut std::collections::BTreeSet<EdgeId>,
    ports: &[(PortId, crate::core::Port)],
) -> Vec<(EdgeId, Edge)> {
    let port_ids: std::collections::BTreeSet<PortId> = ports.iter().map(|(id, _)| *id).collect();
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

fn push_edge_remove_ops(
    ops: &mut Vec<GraphOp>,
    removed_edges: &std::collections::BTreeSet<EdgeId>,
    graph: &Graph,
    interaction: &NodeGraphInteractionState,
    selected_edges: &[EdgeId],
) {
    let mut edges_sel: Vec<EdgeId> = selected_edges.to_vec();
    edges_sel.sort();
    for edge_id in edges_sel {
        if removed_edges.contains(&edge_id) {
            continue;
        }
        if !super::delete_predicates::edge_is_deletable(graph, interaction, edge_id) {
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
}
