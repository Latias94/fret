use super::*;
use crate::core::{CanvasPoint, Edge, EdgeKind, GraphId, Node, NodeKindKey, PortId};
use serde_json::Value;

fn sample_graph() -> (Graph, GraphNodeId, GraphNodeId, EdgeId, EdgeId) {
    let graph_id = GraphId::from_u128(1);
    let selectable_node = GraphNodeId::from_u128(10);
    let locked_node = GraphNodeId::from_u128(11);
    let default_edge = EdgeId::from_u128(20);
    let locked_edge = EdgeId::from_u128(21);
    let from = PortId::from_u128(30);
    let to = PortId::from_u128(31);

    let mut graph = Graph::new(graph_id);
    graph.nodes.insert(
        selectable_node,
        Node {
            kind: NodeKindKey::new("test.selectable"),
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
            ports: vec![from],
            data: Value::Null,
        },
    );
    graph.nodes.insert(
        locked_node,
        Node {
            kind: NodeKindKey::new("test.locked"),
            kind_version: 1,
            pos: CanvasPoint { x: 100.0, y: 0.0 },
            selectable: Some(false),
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: vec![to],
            data: Value::Null,
        },
    );
    graph.edges.insert(
        default_edge,
        Edge {
            kind: EdgeKind::Data,
            from,
            to,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    graph.edges.insert(
        locked_edge,
        Edge {
            kind: EdgeKind::Data,
            from,
            to,
            selectable: Some(false),
            deletable: None,
            reconnectable: None,
        },
    );

    (
        graph,
        selectable_node,
        locked_node,
        default_edge,
        locked_edge,
    )
}

#[test]
fn edge_is_selectable_respects_global_and_edge_overrides() {
    let (graph, _selectable_node, _locked_node, default_edge, locked_edge) = sample_graph();
    let interaction = NodeGraphInteractionState::default();

    assert!(
        NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::edge_is_selectable(
            &graph,
            &interaction,
            default_edge,
        )
    );
    assert!(
        !NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::edge_is_selectable(
            &graph,
            &interaction,
            locked_edge,
        )
    );

    let mut blocked = NodeGraphInteractionState::default();
    blocked.elements_selectable = false;
    assert!(
        !NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::edge_is_selectable(
            &graph,
            &blocked,
            default_edge,
        )
    );

    blocked = NodeGraphInteractionState::default();
    blocked.edges_selectable = false;
    assert!(
        !NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::edge_is_selectable(
            &graph,
            &blocked,
            default_edge,
        )
    );
}

#[test]
fn node_is_selectable_respects_global_and_node_overrides() {
    let (graph, selectable_node, locked_node, _default_edge, _locked_edge) = sample_graph();
    let interaction = NodeGraphInteractionState::default();

    assert!(
        NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::node_is_selectable(
            &graph,
            &interaction,
            selectable_node,
        )
    );
    assert!(
        !NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::node_is_selectable(
            &graph,
            &interaction,
            locked_node,
        )
    );

    let mut blocked = NodeGraphInteractionState::default();
    blocked.elements_selectable = false;
    assert!(
        !NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::node_is_selectable(
            &graph,
            &blocked,
            selectable_node,
        )
    );
}

#[test]
fn selectable_queries_return_false_for_missing_ids() {
    let (graph, _selectable_node, _locked_node, _default_edge, _locked_edge) = sample_graph();
    let interaction = NodeGraphInteractionState::default();

    assert!(
        !NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::node_is_selectable(
            &graph,
            &interaction,
            GraphNodeId::from_u128(999),
        )
    );
    assert!(
        !NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::edge_is_selectable(
            &graph,
            &interaction,
            EdgeId::from_u128(999),
        )
    );
}
