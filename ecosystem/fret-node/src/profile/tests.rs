use crate::core::{
    CanvasPoint, Edge, EdgeId, EdgeKind, Graph, Node, NodeId, NodeKindKey, Port, PortCapacity,
    PortDirection, PortId, PortKey, PortKind,
};
use crate::ops::GraphTransaction;
use crate::profile::{DataflowProfile, apply_transaction_with_profile};

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

fn find_port_by_key(graph: &Graph, node: NodeId, key: &str) -> Option<PortId> {
    graph
        .ports
        .iter()
        .find_map(|(id, p)| (p.node == node && p.key.0 == key).then_some(*id))
}

fn count_inputs(graph: &Graph, node: NodeId) -> usize {
    graph
        .ports
        .values()
        .filter(|p| p.node == node && p.dir == PortDirection::In && p.kind == PortKind::Data)
        .count()
}

#[test]
fn variadic_merge_adds_new_input_when_last_is_connected() {
    let mut graph = Graph::default();
    let mut profile = DataflowProfile::new();

    let src = NodeId::new();
    let merge = NodeId::new();
    graph.nodes.insert(src, make_node("core.src"));
    graph.nodes.insert(merge, make_node("fret.variadic_merge"));

    let out = PortId::new();
    graph.ports.insert(
        out,
        make_port(
            src,
            "out",
            PortDirection::Out,
            PortKind::Data,
            PortCapacity::Multi,
        ),
    );
    graph.nodes.get_mut(&src).unwrap().ports.push(out);

    let in0 = PortId::new();
    let out_merge = PortId::new();
    graph.ports.insert(
        in0,
        make_port(
            merge,
            "in0",
            PortDirection::In,
            PortKind::Data,
            PortCapacity::Single,
        ),
    );
    graph.ports.insert(
        out_merge,
        make_port(
            merge,
            "out",
            PortDirection::Out,
            PortKind::Data,
            PortCapacity::Multi,
        ),
    );
    graph.nodes.get_mut(&merge).unwrap().ports = vec![in0, out_merge];

    let edge_id = EdgeId::new();
    let tx = GraphTransaction {
        label: None,
        ops: vec![crate::ops::GraphOp::AddEdge {
            id: edge_id,
            edge: Edge {
                kind: EdgeKind::Data,
                from: out,
                to: in0,
            },
        }],
    };

    apply_transaction_with_profile(&mut graph, &mut profile, &tx).unwrap();

    assert_eq!(count_inputs(&graph, merge), 2);
    assert!(find_port_by_key(&graph, merge, "in1").is_some());
}

#[test]
fn variadic_merge_trims_trailing_empty_inputs() {
    let mut graph = Graph::default();
    let mut profile = DataflowProfile::new();

    let src = NodeId::new();
    let merge = NodeId::new();
    graph.nodes.insert(src, make_node("core.src"));
    graph.nodes.insert(merge, make_node("fret.variadic_merge"));

    let out = PortId::new();
    graph.ports.insert(
        out,
        make_port(
            src,
            "out",
            PortDirection::Out,
            PortKind::Data,
            PortCapacity::Multi,
        ),
    );
    graph.nodes.get_mut(&src).unwrap().ports.push(out);

    let in0 = PortId::new();
    let out_merge = PortId::new();
    graph.ports.insert(
        in0,
        make_port(
            merge,
            "in0",
            PortDirection::In,
            PortKind::Data,
            PortCapacity::Single,
        ),
    );
    graph.ports.insert(
        out_merge,
        make_port(
            merge,
            "out",
            PortDirection::Out,
            PortKind::Data,
            PortCapacity::Multi,
        ),
    );
    graph.nodes.get_mut(&merge).unwrap().ports = vec![in0, out_merge];

    // Connect to in0 => concretize adds in1.
    let edge_id = EdgeId::new();
    let connect_tx = GraphTransaction {
        label: None,
        ops: vec![crate::ops::GraphOp::AddEdge {
            id: edge_id,
            edge: Edge {
                kind: EdgeKind::Data,
                from: out,
                to: in0,
            },
        }],
    };
    apply_transaction_with_profile(&mut graph, &mut profile, &connect_tx).unwrap();
    assert_eq!(count_inputs(&graph, merge), 2);

    // Remove the only edge; concretize trims to one trailing empty input.
    let remove_tx = GraphTransaction {
        label: None,
        ops: vec![crate::ops::GraphOp::RemoveEdge {
            id: edge_id,
            edge: Edge {
                kind: EdgeKind::Data,
                from: out,
                to: in0,
            },
        }],
    };
    apply_transaction_with_profile(&mut graph, &mut profile, &remove_tx).unwrap();
    assert_eq!(count_inputs(&graph, merge), 1);
    assert!(find_port_by_key(&graph, merge, "in1").is_none());
}
