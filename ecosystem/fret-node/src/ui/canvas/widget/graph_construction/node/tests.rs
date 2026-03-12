use super::*;

#[test]
fn first_added_node_id_returns_first_add_node_op() {
    let a = GraphNodeId::from_u128(1);
    let b = GraphNodeId::from_u128(2);
    let ops = vec![
        GraphOp::SetNodePorts {
            id: a,
            from: Vec::new(),
            to: Vec::new(),
        },
        GraphOp::AddNode {
            id: a,
            node: reroute_node(CanvasPoint { x: 1.0, y: 2.0 }),
        },
        GraphOp::AddNode {
            id: b,
            node: reroute_node(CanvasPoint { x: 3.0, y: 4.0 }),
        },
    ];

    assert_eq!(
        NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::first_added_node_id(&ops),
        Some(a)
    );
}

#[test]
fn build_reroute_create_ops_emits_node_ports_and_ordering() {
    let ops = NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::build_reroute_create_ops(
        CanvasPoint { x: 10.0, y: 20.0 },
    );

    assert!(matches!(ops[0], GraphOp::AddNode { .. }));
    assert!(matches!(ops[1], GraphOp::AddPort { .. }));
    assert!(matches!(ops[2], GraphOp::AddPort { .. }));
    assert!(matches!(ops[3], GraphOp::SetNodePorts { .. }));

    let GraphOp::AddNode {
        id: node_id,
        ref node,
    } = ops[0]
    else {
        panic!("expected add node");
    };
    assert_eq!(node.kind, NodeKindKey::new(REROUTE_KIND));
    assert_eq!(node.pos, CanvasPoint { x: 10.0, y: 20.0 });

    let GraphOp::AddPort {
        id: in_port_id,
        ref port,
    } = ops[1]
    else {
        panic!("expected input add port");
    };
    assert_eq!(port.node, node_id);
    assert_eq!(port.dir, PortDirection::In);

    let GraphOp::AddPort {
        id: out_port_id,
        ref port,
    } = ops[2]
    else {
        panic!("expected output add port");
    };
    assert_eq!(port.node, node_id);
    assert_eq!(port.dir, PortDirection::Out);

    let GraphOp::SetNodePorts {
        id,
        ref from,
        ref to,
    } = ops[3]
    else {
        panic!("expected set node ports");
    };
    assert_eq!(id, node_id);
    assert!(from.is_empty());
    assert_eq!(to, &vec![in_port_id, out_port_id]);
}
