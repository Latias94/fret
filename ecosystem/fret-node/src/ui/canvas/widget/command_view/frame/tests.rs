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
