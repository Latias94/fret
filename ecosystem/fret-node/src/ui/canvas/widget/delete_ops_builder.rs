mod edge;
mod group;
mod node;

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

    group::push_group_remove_ops(&mut ops, graph, selected_groups);
    node::push_node_remove_ops(
        &mut ops,
        &mut removed_edges,
        graph,
        interaction,
        selected_nodes,
    );
    edge::push_edge_remove_ops(&mut ops, &removed_edges, graph, interaction, selected_edges);
    ops
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        CanvasPoint, Edge, EdgeKind, Graph, GraphId, Node, NodeKindKey, Port, PortCapacity,
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
    fn delete_selection_ops_does_not_double_remove_edges_already_owned_by_removed_nodes() {
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
            .insert(other_node, test_node("b", CanvasPoint { x: 100.0, y: 0.0 }));
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

        let ops = delete_selection_ops(
            &graph,
            &NodeGraphInteractionState::default(),
            &[node_id],
            &[edge_id],
            &[],
        );

        assert!(
            ops.iter()
                .any(|op| matches!(op, GraphOp::RemoveNode { id, .. } if *id == node_id))
        );
        assert!(
            !ops.iter()
                .any(|op| matches!(op, GraphOp::RemoveEdge { id, .. } if *id == edge_id))
        );
    }
}
