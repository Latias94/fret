use fret_core::{Point, Px, Rect, Size};

use crate::core::{
    CanvasPoint, CanvasSize, Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey,
    Port, PortCapacity, PortDirection, PortId, PortKey, PortKind,
};

use crate::rules::EdgeEndpoint;

use super::prelude::{NodeGraphCanvas, edge_drag};
use super::{NullServices, TestUiHostImpl, event_cx, insert_view};
use crate::ui::canvas::state::{EdgeDrag, WireDragKind};

fn make_test_graph_edge_reconnect() -> (Graph, EdgeId, PortId, PortId) {
    let mut graph = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let a = NodeId::new();
    let out = PortId::new();
    graph.nodes.insert(
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
            size: Some(CanvasSize {
                width: 220.0,
                height: 80.0,
            }),
            hidden: false,
            collapsed: false,
            ports: vec![out],
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        out,
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

    let b = NodeId::new();
    let inn = PortId::new();
    graph.nodes.insert(
        b,
        Node {
            kind,
            kind_version: 1,
            pos: CanvasPoint { x: 320.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 220.0,
                height: 80.0,
            }),
            hidden: false,
            collapsed: false,
            ports: vec![inn],
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        inn,
        Port {
            node: b,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        },
    );

    let edge = EdgeId::new();
    graph.edges.insert(
        edge,
        Edge {
            kind: EdgeKind::Data,
            from: out,
            to: inn,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    (graph, edge, out, inn)
}

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    )
}

#[test]
fn edge_drag_prefers_from_endpoint_when_port_centers_are_equidistant() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, edge, out, inn) = make_test_graph_edge_reconnect();
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);

    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 1.0;
        s.interaction.edges_reconnectable = true;
        s.interaction.connection_drag_threshold = 1.0;
        s.interaction.reconnect_radius = 0.0;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let snapshot = canvas.sync_view_state(&mut host);

    let geom = canvas.canvas_geometry(&host, &snapshot);
    let from_center = geom.port_center(out).expect("from port center");
    let to_center = geom.port_center(inn).expect("to port center");

    let start_pos = Point::new(
        Px(0.5 * (from_center.x.0 + to_center.x.0)),
        Px(0.5 * (from_center.y.0 + to_center.y.0)),
    );
    canvas.interaction.edge_drag = Some(EdgeDrag { edge, start_pos });

    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds(),
        &mut prevented_default_actions,
    );

    let pos_big = Point::new(Px(start_pos.x.0 + 16.0), start_pos.y);
    assert!(edge_drag::handle_edge_drag_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos_big,
        snapshot.zoom,
    ));

    let Some(w) = canvas.interaction.wire_drag.as_ref() else {
        panic!("expected wire_drag to start");
    };
    match &w.kind {
        WireDragKind::Reconnect {
            edge: e,
            endpoint,
            fixed,
        } => {
            assert_eq!(*e, edge);
            assert_eq!(*endpoint, EdgeEndpoint::From);
            assert_eq!(*fixed, inn);
        }
        other => panic!("unexpected wire drag kind: {other:?}"),
    }
}

#[test]
fn edge_reconnect_radius_is_zoom_invariant_in_screen_space() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, edge, out, inn) = make_test_graph_edge_reconnect();
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);

    let radius_screen = 16.0;

    for zoom in [0.5, 2.0] {
        let _ = view.update(&mut host, |s, _cx| {
            s.zoom = zoom;
            s.interaction.edges_reconnectable = true;
            s.interaction.connection_drag_threshold = 1.0;
            s.interaction.reconnect_radius = radius_screen;
        });

        // Inside radius: should start wire-drag.
        {
            let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
            let snapshot = canvas.sync_view_state(&mut host);
            let geom = canvas.canvas_geometry(&host, &snapshot);
            let from_center = geom.port_center(out).expect("from port center");
            let to_center = geom.port_center(inn).expect("to port center");
            let dir = if to_center.x.0 >= from_center.x.0 {
                -1.0
            } else {
                1.0
            };
            let start_pos = Point::new(
                Px(from_center.x.0 + dir * (radius_screen - 0.1) / zoom),
                from_center.y,
            );
            canvas.interaction.edge_drag = Some(EdgeDrag { edge, start_pos });

            let mut services = NullServices::default();
            let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
            let mut cx = event_cx(
                &mut host,
                &mut services,
                bounds(),
                &mut prevented_default_actions,
            );

            let pos_big = Point::new(Px(start_pos.x.0 + 16.0 / zoom), start_pos.y);
            assert!(edge_drag::handle_edge_drag_move(
                &mut canvas,
                &mut cx,
                &snapshot,
                pos_big,
                snapshot.zoom,
            ));
            assert!(canvas.interaction.wire_drag.is_some());
            assert!(canvas.interaction.edge_drag.is_none());
        }

        // Outside radius: should not start wire-drag, edge_drag stays active.
        {
            let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
            let snapshot = canvas.sync_view_state(&mut host);
            let geom = canvas.canvas_geometry(&host, &snapshot);
            let from_center = geom.port_center(out).expect("from port center");
            let to_center = geom.port_center(inn).expect("to port center");
            let dir = if to_center.x.0 >= from_center.x.0 {
                -1.0
            } else {
                1.0
            };
            let start_pos = Point::new(
                Px(from_center.x.0 + dir * (radius_screen + 1.0) / zoom),
                from_center.y,
            );
            canvas.interaction.edge_drag = Some(EdgeDrag { edge, start_pos });

            let mut services = NullServices::default();
            let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
            let mut cx = event_cx(
                &mut host,
                &mut services,
                bounds(),
                &mut prevented_default_actions,
            );

            let pos_big = Point::new(Px(start_pos.x.0 + 16.0 / zoom), start_pos.y);
            assert!(!edge_drag::handle_edge_drag_move(
                &mut canvas,
                &mut cx,
                &snapshot,
                pos_big,
                snapshot.zoom,
            ));
            assert!(canvas.interaction.wire_drag.is_none());
            assert!(canvas.interaction.edge_drag.is_some());
        }
    }
}
