use super::*;
use crate::core::{CanvasPoint, GraphId, Node, NodeKindKey};
use serde_json::Value;

fn sample_graph() -> (Graph, GraphNodeId, GraphNodeId) {
    let graph_id = GraphId::from_u128(1);
    let default_node = GraphNodeId::from_u128(10);
    let locked_node = GraphNodeId::from_u128(11);

    let mut graph = Graph::new(graph_id);
    graph.nodes.insert(
        default_node,
        Node {
            kind: NodeKindKey::new("test.default"),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
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
    graph.nodes.insert(
        locked_node,
        Node {
            kind: NodeKindKey::new("test.locked"),
            kind_version: 1,
            pos: CanvasPoint { x: 100.0, y: 0.0 },
            selectable: None,
            draggable: Some(false),
            connectable: Some(false),
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

    (graph, default_node, locked_node)
}

#[test]
fn node_is_draggable_respects_global_and_node_override() {
    let (graph, default_node, locked_node) = sample_graph();
    let interaction = NodeGraphInteractionState::default();

    assert!(
        NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::node_is_draggable(
            &graph,
            &interaction,
            default_node,
        )
    );
    assert!(
        !NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::node_is_draggable(
            &graph,
            &interaction,
            locked_node,
        )
    );

    let mut blocked = NodeGraphInteractionState::default();
    blocked.nodes_draggable = false;
    assert!(
        !NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::node_is_draggable(
            &graph,
            &blocked,
            default_node,
        )
    );
}

#[test]
fn node_is_connectable_respects_global_default_and_override() {
    let (graph, default_node, locked_node) = sample_graph();
    let interaction = NodeGraphInteractionState::default();

    assert!(graph_node_is_connectable(
        &graph,
        &interaction,
        default_node
    ));
    assert!(!graph_node_is_connectable(
        &graph,
        &interaction,
        locked_node
    ));

    let mut blocked = NodeGraphInteractionState::default();
    blocked.nodes_connectable = false;
    assert!(!graph_node_is_connectable(&graph, &blocked, default_node));
}
