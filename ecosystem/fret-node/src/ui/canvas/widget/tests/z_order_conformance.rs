use crate::core::{
    CanvasPoint, Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey, Port,
    PortCapacity, PortDirection, PortId, PortKey, PortKind,
};

use super::prelude::NodeGraphCanvas;
use super::{TestUiHostImpl, insert_view};

#[test]
fn edges_are_sorted_by_endpoint_z_order() {
    let mut host = TestUiHostImpl::default();

    let mut graph_value = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let a = NodeId::new();
    let b = NodeId::new();
    let c = NodeId::new();

    let a_out = PortId::new();
    let b_in = PortId::new();
    let c_out = PortId::new();

    graph_value.nodes.insert(
        a,
        Node {
            kind: kind.clone(),
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
            ports: vec![a_out],
            data: serde_json::Value::Null,
        },
    );
    graph_value.nodes.insert(
        b,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 260.0, y: 0.0 },
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
            ports: vec![b_in],
            data: serde_json::Value::Null,
        },
    );
    graph_value.nodes.insert(
        c,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 520.0, y: 0.0 },
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
            ports: vec![c_out],
            data: serde_json::Value::Null,
        },
    );

    graph_value.ports.insert(
        a_out,
        Port {
            node: a,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        },
    );
    graph_value.ports.insert(
        b_in,
        Port {
            node: b,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        },
    );
    graph_value.ports.insert(
        c_out,
        Port {
            node: c,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        },
    );

    let edge_low = EdgeId::new();
    let edge_high = EdgeId::new();
    graph_value.edges.insert(
        edge_low,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    graph_value.edges.insert(
        edge_high,
        Edge {
            kind: EdgeKind::Data,
            from: c_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);

    let _ = view.update(&mut host, |s, _cx| {
        // Put node C on top (highest z). Edge C->B should therefore be drawn above A->B.
        s.draw_order = vec![a, b, c];
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, index) = canvas.canvas_derived(&host, &snapshot);
    let render = canvas.collect_render_data(
        &host,
        &snapshot,
        geom,
        index,
        None,
        snapshot.zoom,
        None,
        false,
        false,
        true,
    );

    assert_eq!(render.edges.len(), 2);
    assert_eq!(render.edges[0].id, edge_low);
    assert_eq!(render.edges[1].id, edge_high);

    // Extra sanity: both edges should exist and have finite endpoints.
    for e in &render.edges {
        assert!(e.from.x.0.is_finite() && e.from.y.0.is_finite());
        assert!(e.to.x.0.is_finite() && e.to.y.0.is_finite());
    }
}
