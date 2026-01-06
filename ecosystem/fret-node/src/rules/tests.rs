use crate::core::{
    CanvasPoint, Graph, Node, NodeId, NodeKindKey, Port, PortCapacity, PortDirection, PortId,
    PortKey, PortKind,
};
use crate::ops::{GraphTransaction, apply_transaction};
use crate::rules::plan_connect;

fn make_node(kind: &str) -> Node {
    Node {
        kind: NodeKindKey::new(kind),
        kind_version: 0,
        pos: CanvasPoint { x: 0.0, y: 0.0 },
        collapsed: false,
        ports: Vec::new(),
        data: serde_json::Value::Null,
    }
}

fn make_port(
    node: NodeId,
    key: &str,
    dir: PortDirection,
    kind: PortKind,
    capacity: PortCapacity,
) -> Port {
    Port {
        node,
        key: PortKey::new(key),
        dir,
        kind,
        capacity,
        ty: None,
        data: serde_json::Value::Null,
    }
}

#[test]
fn plan_connect_swaps_in_out() {
    let mut graph = Graph::default();

    let a = NodeId::new();
    let b = NodeId::new();
    graph.nodes.insert(a, make_node("core.a"));
    graph.nodes.insert(b, make_node("core.b"));

    let out = PortId::new();
    let inn = PortId::new();
    graph.ports.insert(
        out,
        make_port(
            a,
            "out",
            PortDirection::Out,
            PortKind::Data,
            PortCapacity::Multi,
        ),
    );
    graph.ports.insert(
        inn,
        make_port(
            b,
            "in",
            PortDirection::In,
            PortKind::Data,
            PortCapacity::Single,
        ),
    );

    let plan = plan_connect(&graph, inn, out);
    assert_eq!(plan.ops.len(), 1);
}

#[test]
fn plan_connect_single_input_disconnects_existing() {
    let mut graph = Graph::default();

    let a = NodeId::new();
    let b = NodeId::new();
    let c = NodeId::new();
    graph.nodes.insert(a, make_node("core.a"));
    graph.nodes.insert(b, make_node("core.b"));
    graph.nodes.insert(c, make_node("core.c"));

    let out1 = PortId::new();
    let out2 = PortId::new();
    let inn = PortId::new();
    graph.ports.insert(
        out1,
        make_port(
            a,
            "out",
            PortDirection::Out,
            PortKind::Data,
            PortCapacity::Multi,
        ),
    );
    graph.ports.insert(
        out2,
        make_port(
            c,
            "out",
            PortDirection::Out,
            PortKind::Data,
            PortCapacity::Multi,
        ),
    );
    graph.ports.insert(
        inn,
        make_port(
            b,
            "in",
            PortDirection::In,
            PortKind::Data,
            PortCapacity::Single,
        ),
    );

    // First connect out1 -> inn and apply.
    let plan1 = plan_connect(&graph, out1, inn);
    let tx1 = GraphTransaction {
        label: None,
        ops: plan1.ops,
    };
    apply_transaction(&mut graph, &tx1).unwrap();
    assert_eq!(graph.edges.len(), 1);

    // Now connect out2 -> inn; should remove old edge and add a new one.
    let plan2 = plan_connect(&graph, out2, inn);
    assert_eq!(plan2.ops.len(), 2);
}
