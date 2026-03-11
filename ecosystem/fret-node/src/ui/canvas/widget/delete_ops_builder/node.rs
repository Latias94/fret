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
mod tests {
    use super::*;
    use crate::core::{
        CanvasPoint, EdgeKind, Graph, GraphId, Node, NodeKindKey, Port, PortCapacity,
        PortDirection, PortKey, PortKind,
    };
    use serde_json::Value;

    fn test_node(kind: &str, pos: CanvasPoint) -> Node {
        Node {
            kind: NodeKindKey::new(kind),
            kind_version: 1,
            pos,
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
        }
    }

    fn test_port(node: GraphNodeId, key: &str, dir: PortDirection) -> Port {
        Port {
            node,
            key: PortKey::new(key),
            dir,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        }
    }

    #[test]
    fn collect_node_edges_deduplicates_edges_already_marked_removed() {
        let mut graph = Graph::new(GraphId::new());
        let node_id = GraphNodeId::new();
        let other_node = GraphNodeId::new();
        let out_port = PortId::new();
        let in_port = PortId::new();
        let edge_id = EdgeId::new();

        graph
            .nodes
            .insert(node_id, test_node("a", CanvasPoint { x: 0.0, y: 0.0 }));
        graph
            .nodes
            .insert(other_node, test_node("b", CanvasPoint { x: 1.0, y: 1.0 }));
        graph
            .ports
            .insert(out_port, test_port(node_id, "out", PortDirection::Out));
        graph
            .ports
            .insert(in_port, test_port(other_node, "in", PortDirection::In));
        graph.edges.insert(
            edge_id,
            Edge {
                kind: EdgeKind::Data,
                from: out_port,
                to: in_port,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );

        let ports = collect_node_ports(&graph, node_id);
        let mut removed_edges = BTreeSet::from([edge_id]);
        let edges = collect_node_edges(&graph, &mut removed_edges, &ports);
        assert!(edges.is_empty());
    }
}
