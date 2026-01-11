use crate::core::{
    CanvasPoint, Edge, EdgeId, EdgeKind, Graph, Node, NodeId, NodeKindKey, Port, PortCapacity,
    PortDirection, PortId, PortKey, PortKind,
};
use crate::ops::{GraphTransaction, apply_transaction};
use crate::rules::{
    EdgeEndpoint, InsertNodeSpec, plan_connect, plan_connect_by_inserting_node, plan_connect_typed,
    plan_reconnect_edge, plan_split_edge_by_inserting_node,
};
use crate::types::{DefaultTypeCompatibility, TypeDesc};

fn make_node(kind: &str) -> Node {
    Node {
        kind: NodeKindKey::new(kind),
        kind_version: 0,
        pos: CanvasPoint { x: 0.0, y: 0.0 },
        selectable: None,
        parent: None,
        size: None,
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

#[test]
fn plan_connect_typed_rejects_incompatible_data_types() {
    let mut graph = Graph::default();

    let a = NodeId::new();
    let b = NodeId::new();
    graph.nodes.insert(a, make_node("core.a"));
    graph.nodes.insert(b, make_node("core.b"));

    let out = PortId::new();
    let inn = PortId::new();
    let mut out_port = make_port(
        a,
        "out",
        PortDirection::Out,
        PortKind::Data,
        PortCapacity::Multi,
    );
    out_port.ty = Some(TypeDesc::Int);
    graph.ports.insert(out, out_port);

    let mut in_port = make_port(
        b,
        "in",
        PortDirection::In,
        PortKind::Data,
        PortCapacity::Single,
    );
    in_port.ty = Some(TypeDesc::String);
    graph.ports.insert(inn, in_port);

    let mut compat = DefaultTypeCompatibility::default();
    let plan = plan_connect_typed(
        &graph,
        out,
        inn,
        |g, p| g.ports.get(&p).and_then(|p| p.ty.clone()),
        &mut compat,
    );
    assert_eq!(plan.decision, crate::rules::ConnectDecision::Reject);
    assert!(plan.ops.is_empty());
}

#[test]
fn plan_connect_typed_accepts_int_to_float() {
    let mut graph = Graph::default();

    let a = NodeId::new();
    let b = NodeId::new();
    graph.nodes.insert(a, make_node("core.a"));
    graph.nodes.insert(b, make_node("core.b"));

    let out = PortId::new();
    let inn = PortId::new();
    let mut out_port = make_port(
        a,
        "out",
        PortDirection::Out,
        PortKind::Data,
        PortCapacity::Multi,
    );
    out_port.ty = Some(TypeDesc::Int);
    graph.ports.insert(out, out_port);

    let mut in_port = make_port(
        b,
        "in",
        PortDirection::In,
        PortKind::Data,
        PortCapacity::Single,
    );
    in_port.ty = Some(TypeDesc::Float);
    graph.ports.insert(inn, in_port);

    let mut compat = DefaultTypeCompatibility::default();
    let plan = plan_connect_typed(
        &graph,
        out,
        inn,
        |g, p| g.ports.get(&p).and_then(|p| p.ty.clone()),
        &mut compat,
    );
    assert_eq!(plan.decision, crate::rules::ConnectDecision::Accept);
    assert!(!plan.ops.is_empty());
}

#[test]
fn plan_reconnect_preserves_edge_id() {
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
            "out1",
            PortDirection::Out,
            PortKind::Data,
            PortCapacity::Multi,
        ),
    );
    graph.ports.insert(
        out2,
        make_port(
            c,
            "out2",
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
            PortCapacity::Multi,
        ),
    );

    let edge_id = EdgeId::new();
    graph.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: out1,
            to: inn,
            selectable: None,
        },
    );

    let plan = plan_reconnect_edge(&graph, edge_id, EdgeEndpoint::From, out2);
    assert_eq!(plan.decision, crate::rules::ConnectDecision::Accept);
    assert_eq!(plan.ops.len(), 1);

    let tx = GraphTransaction {
        label: None,
        ops: plan.ops,
    };
    apply_transaction(&mut graph, &tx).unwrap();

    let edge = graph.edges.get(&edge_id).unwrap();
    assert_eq!(edge.from, out2);
    assert_eq!(edge.to, inn);
}

#[test]
fn plan_reconnect_single_target_disconnects_other_edges() {
    let mut graph = Graph::default();

    let a = NodeId::new();
    let b = NodeId::new();
    let c = NodeId::new();
    let d = NodeId::new();
    graph.nodes.insert(a, make_node("core.a"));
    graph.nodes.insert(b, make_node("core.b"));
    graph.nodes.insert(c, make_node("core.c"));
    graph.nodes.insert(d, make_node("core.d"));

    let out1 = PortId::new();
    let out2 = PortId::new();
    let out3 = PortId::new();
    let inn = PortId::new();
    graph.ports.insert(
        out1,
        make_port(
            a,
            "out1",
            PortDirection::Out,
            PortKind::Data,
            PortCapacity::Multi,
        ),
    );
    graph.ports.insert(
        out2,
        make_port(
            c,
            "out2",
            PortDirection::Out,
            PortKind::Data,
            PortCapacity::Multi,
        ),
    );
    graph.ports.insert(
        out3,
        make_port(
            d,
            "out3",
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

    let edge_keep = EdgeId::new();
    let edge_drop = EdgeId::new();
    graph.edges.insert(
        edge_keep,
        Edge {
            kind: EdgeKind::Data,
            from: out1,
            to: inn,
            selectable: None,
        },
    );
    graph.edges.insert(
        edge_drop,
        Edge {
            kind: EdgeKind::Data,
            from: out2,
            to: inn,
            selectable: None,
        },
    );

    let plan = plan_reconnect_edge(&graph, edge_keep, EdgeEndpoint::From, out3);
    assert_eq!(plan.decision, crate::rules::ConnectDecision::Accept);
    assert_eq!(plan.ops.len(), 2, "expected remove + set_endpoints");

    let tx = GraphTransaction {
        label: None,
        ops: plan.ops,
    };
    apply_transaction(&mut graph, &tx).unwrap();

    assert!(
        !graph.edges.contains_key(&edge_drop),
        "expected other edge removed"
    );
    let edge = graph.edges.get(&edge_keep).unwrap();
    assert_eq!(edge.from, out3);
    assert_eq!(edge.to, inn);
}

#[test]
fn plan_connect_by_inserting_node_disconnects_single_target() {
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
            "out1",
            PortDirection::Out,
            PortKind::Data,
            PortCapacity::Multi,
        ),
    );
    graph.ports.insert(
        out2,
        make_port(
            c,
            "out2",
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

    // Now connect out2 -> inn via an inserted node; should remove old edge and add two new edges.
    let inserted_node_id = NodeId::new();
    let inserted_in = PortId::new();
    let inserted_out = PortId::new();

    let spec = InsertNodeSpec {
        node_id: inserted_node_id,
        node: make_node("demo.convert"),
        ports: vec![
            (
                inserted_in,
                make_port(
                    inserted_node_id,
                    "in",
                    PortDirection::In,
                    PortKind::Data,
                    PortCapacity::Single,
                ),
            ),
            (
                inserted_out,
                make_port(
                    inserted_node_id,
                    "out",
                    PortDirection::Out,
                    PortKind::Data,
                    PortCapacity::Multi,
                ),
            ),
        ],
        input: inserted_in,
        output: inserted_out,
    };

    let edge_a = EdgeId::new();
    let edge_b = EdgeId::new();
    let plan2 = plan_connect_by_inserting_node(&graph, out2, inn, edge_a, edge_b, spec);
    assert_eq!(plan2.decision, crate::rules::ConnectDecision::Accept);

    let tx2 = GraphTransaction {
        label: None,
        ops: plan2.ops,
    };
    apply_transaction(&mut graph, &tx2).unwrap();

    assert_eq!(graph.nodes.len(), 4);
    assert_eq!(graph.edges.len(), 2);
}

#[test]
fn plan_split_edge_by_inserting_node_preserves_edge_id() {
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

    let edge_id = EdgeId::new();
    graph.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: out,
            to: inn,
            selectable: None,
        },
    );

    let inserted_node_id = NodeId::new();
    let inserted_in = PortId::new();
    let inserted_out = PortId::new();
    let spec = InsertNodeSpec {
        node_id: inserted_node_id,
        node: make_node("demo.reroute"),
        ports: vec![
            (
                inserted_in,
                make_port(
                    inserted_node_id,
                    "in",
                    PortDirection::In,
                    PortKind::Data,
                    PortCapacity::Single,
                ),
            ),
            (
                inserted_out,
                make_port(
                    inserted_node_id,
                    "out",
                    PortDirection::Out,
                    PortKind::Data,
                    PortCapacity::Multi,
                ),
            ),
        ],
        input: inserted_in,
        output: inserted_out,
    };

    let new_edge_id = EdgeId::new();
    let plan = plan_split_edge_by_inserting_node(&graph, edge_id, new_edge_id, spec);
    assert_eq!(plan.decision, crate::rules::ConnectDecision::Accept);

    let tx = GraphTransaction {
        label: None,
        ops: plan.ops,
    };
    apply_transaction(&mut graph, &tx).unwrap();

    assert_eq!(graph.edges.len(), 2);
    assert!(graph.edges.contains_key(&edge_id));
    assert!(graph.edges.contains_key(&new_edge_id));
    assert_eq!(graph.edges.get(&edge_id).unwrap().to, inserted_in);
    assert_eq!(graph.edges.get(&new_edge_id).unwrap().from, inserted_out);
}

#[test]
fn plan_reconnect_rejects_duplicate_connection() {
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
            "out1",
            PortDirection::Out,
            PortKind::Data,
            PortCapacity::Multi,
        ),
    );
    graph.ports.insert(
        out2,
        make_port(
            c,
            "out2",
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
            PortCapacity::Multi,
        ),
    );

    let edge_a = EdgeId::new();
    let edge_b = EdgeId::new();
    graph.edges.insert(
        edge_a,
        Edge {
            kind: EdgeKind::Data,
            from: out1,
            to: inn,
            selectable: None,
        },
    );
    graph.edges.insert(
        edge_b,
        Edge {
            kind: EdgeKind::Data,
            from: out2,
            to: inn,
            selectable: None,
        },
    );

    let plan = plan_reconnect_edge(&graph, edge_a, EdgeEndpoint::From, out2);
    assert_eq!(plan.decision, crate::rules::ConnectDecision::Reject);
}
