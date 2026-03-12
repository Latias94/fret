use super::*;
use crate::core::{CanvasPoint, EdgeKind, Graph, GraphId, PortDirection};

#[test]
fn collect_node_edges_deduplicates_edges_already_marked_removed() {
    let mut graph = Graph::new(GraphId::new());
    let node_id = GraphNodeId::new();
    let other_node = GraphNodeId::new();
    let out_port = PortId::new();
    let in_port = PortId::new();
    let edge_id = EdgeId::new();

    graph.nodes.insert(
        node_id,
        super::super::test_support::test_node("a", CanvasPoint { x: 0.0, y: 0.0 }),
    );
    graph.nodes.insert(
        other_node,
        super::super::test_support::test_node("b", CanvasPoint { x: 1.0, y: 1.0 }),
    );
    graph.ports.insert(
        out_port,
        super::super::test_support::test_port(node_id, "out", PortDirection::Out),
    );
    graph.ports.insert(
        in_port,
        super::super::test_support::test_port(other_node, "in", PortDirection::In),
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

    let ports = collect_node_ports(&graph, node_id);
    let mut removed_edges = BTreeSet::from([edge_id]);
    let edges = collect_node_edges(&graph, &mut removed_edges, &ports);
    assert!(edges.is_empty());
}
