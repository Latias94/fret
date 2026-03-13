use super::super::*;
use crate::core::{CanvasPoint, Edge, EdgeKind, Graph, GraphId, PortDirection};

#[test]
fn delete_selection_ops_does_not_double_remove_edges_already_owned_by_removed_nodes() {
    let mut graph = Graph::new(GraphId::new());
    let node_id = GraphNodeId::new();
    let other_node = GraphNodeId::new();
    let out_port = PortId::new();
    let in_port = PortId::new();
    let edge_id = EdgeId::new();

    graph.nodes.insert(
        node_id,
        super::test_support::test_node("a", CanvasPoint { x: 0.0, y: 0.0 }),
    );
    graph.nodes.insert(
        other_node,
        super::test_support::test_node("b", CanvasPoint { x: 100.0, y: 0.0 }),
    );
    graph.ports.insert(
        out_port,
        super::test_support::test_port(node_id, "out", PortDirection::Out),
    );
    graph.ports.insert(
        in_port,
        super::test_support::test_port(other_node, "in", PortDirection::In),
    );
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

    let ops = super::assemble::delete_selection_ops(
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
