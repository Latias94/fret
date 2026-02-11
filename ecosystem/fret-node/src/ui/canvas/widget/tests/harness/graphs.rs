use crate::core::{
    CanvasPoint, CanvasSize, Graph, GraphId, Node, NodeId, NodeKindKey, Port, PortCapacity,
    PortDirection, PortId, PortKey, PortKind,
};
use crate::io::NodeGraphViewState;
use fret_runtime::Model;
use serde_json::Value;

use super::TestUiHostImpl;

fn test_node(
    kind: NodeKindKey,
    pos: CanvasPoint,
    size: Option<CanvasSize>,
    ports: Vec<PortId>,
) -> Node {
    Node {
        kind,
        kind_version: 1,
        pos,
        selectable: None,
        draggable: None,
        connectable: None,
        deletable: None,
        parent: None,
        extent: None,
        expand_parent: None,
        size,
        hidden: false,
        collapsed: false,
        ports,
        data: Value::Null,
    }
}

fn test_data_port(node: NodeId, key: &str, dir: PortDirection, capacity: PortCapacity) -> Port {
    Port {
        node,
        key: PortKey::new(key),
        dir,
        kind: PortKind::Data,
        capacity,
        connectable: None,
        connectable_start: None,
        connectable_end: None,
        ty: None,
        data: Value::Null,
    }
}

pub(crate) fn make_test_graph_two_nodes() -> (Graph, NodeId, NodeId) {
    let mut graph = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let a = NodeId::new();
    let b = NodeId::new();

    graph.nodes.insert(
        a,
        test_node(
            kind.clone(),
            CanvasPoint { x: 0.0, y: 0.0 },
            None,
            Vec::new(),
        ),
    );
    graph.nodes.insert(
        b,
        test_node(kind, CanvasPoint { x: 10.0, y: 0.0 }, None, Vec::new()),
    );

    (graph, a, b)
}

pub(crate) fn make_test_graph_two_nodes_with_size() -> (Graph, NodeId, NodeId) {
    let mut graph = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let a = NodeId::new();
    let b = NodeId::new();

    graph.nodes.insert(
        a,
        test_node(
            kind.clone(),
            CanvasPoint { x: 0.0, y: 0.0 },
            Some(CanvasSize {
                width: 40.0,
                height: 20.0,
            }),
            Vec::new(),
        ),
    );
    graph.nodes.insert(
        b,
        test_node(
            kind,
            CanvasPoint { x: 10.0, y: 5.0 },
            Some(CanvasSize {
                width: 40.0,
                height: 20.0,
            }),
            Vec::new(),
        ),
    );

    (graph, a, b)
}

pub(crate) fn make_test_graph_two_nodes_with_ports()
-> (Graph, NodeId, PortId, PortId, NodeId, PortId) {
    let mut graph = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let a = NodeId::new();
    let a_in = PortId::new();
    let a_out = PortId::new();
    graph.nodes.insert(
        a,
        test_node(
            kind.clone(),
            CanvasPoint { x: 0.0, y: 0.0 },
            None,
            vec![a_in, a_out],
        ),
    );
    graph.ports.insert(
        a_in,
        test_data_port(a, "in", PortDirection::In, PortCapacity::Single),
    );
    graph.ports.insert(
        a_out,
        test_data_port(a, "out", PortDirection::Out, PortCapacity::Multi),
    );

    let b = NodeId::new();
    let b_in = PortId::new();
    graph.nodes.insert(
        b,
        test_node(kind, CanvasPoint { x: 200.0, y: 0.0 }, None, vec![b_in]),
    );
    graph.ports.insert(
        b_in,
        test_data_port(b, "in", PortDirection::In, PortCapacity::Single),
    );

    (graph, a, a_in, a_out, b, b_in)
}

pub(crate) fn make_test_graph_two_nodes_with_ports_spaced_x(
    dx: f32,
) -> (Graph, NodeId, PortId, PortId, NodeId, PortId) {
    let (mut graph, a, a_in, a_out, b, b_in) = make_test_graph_two_nodes_with_ports();
    graph
        .nodes
        .entry(b)
        .and_modify(|n| n.pos = CanvasPoint { x: dx, y: 0.0 });
    (graph, a, a_in, a_out, b, b_in)
}

pub(crate) fn read_node_pos(
    host: &mut TestUiHostImpl,
    model: &Model<Graph>,
    id: NodeId,
) -> CanvasPoint {
    model
        .read_ref(host, |g| g.nodes.get(&id).map(|n| n.pos))
        .ok()
        .flatten()
        .unwrap_or_default()
}

pub(crate) fn insert_view(host: &mut TestUiHostImpl) -> Model<NodeGraphViewState> {
    host.models.insert(NodeGraphViewState::default())
}

pub(crate) fn insert_graph_view(
    host: &mut TestUiHostImpl,
    graph_value: Graph,
) -> (Model<Graph>, Model<NodeGraphViewState>) {
    let graph = host.models.insert(graph_value);
    let view = insert_view(host);
    (graph, view)
}
pub(crate) fn make_host_graph_view(
    graph_value: Graph,
) -> (TestUiHostImpl, Model<Graph>, Model<NodeGraphViewState>) {
    let mut host = TestUiHostImpl::default();
    let (graph, view) = insert_graph_view(&mut host, graph_value);
    (host, graph, view)
}
