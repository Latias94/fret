use fret_core::{Modifiers, MouseButton, Point, PointerEvent, Px, Rect, Size};
use fret_ui::retained_bridge::Widget as _;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::{
    CanvasPoint, Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey, Port,
    PortCapacity, PortDirection, PortId, PortKey, PortKind,
};

use crate::ui::canvas::state::{ContextMenuTarget, EdgeInsertDrag, PendingEdgeInsertDrag};
use crate::ui::presenter::{EdgeRenderHint, EdgeRouteKind, NodeGraphPresenter};

use super::prelude::{NodeGraphCanvas, edge_insert_drag};
use super::{
    NullServices, TestUiHostImpl, event_cx, insert_view,
    make_test_graph_two_nodes_with_ports_spaced_x,
};

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    )
}

#[test]
fn edge_insert_drag_threshold_is_zoom_invariant_in_screen_space() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(420.0);
    let edge_id = EdgeId(Uuid::from_u128(1));
    graph_value.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);

    let threshold_screen = 8.0;
    let eps_screen = 0.1;

    for zoom in [0.5, 2.0] {
        let _ = view.update(&mut host, |s, _cx| {
            s.zoom = zoom;
            s.interaction.connection_drag_threshold = threshold_screen;
        });

        let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
        let snapshot = canvas.sync_view_state(&mut host);
        assert!((snapshot.zoom - zoom).abs() <= 1.0e-6);

        canvas.interaction.pending_edge_insert_drag = Some(PendingEdgeInsertDrag {
            edge: edge_id,
            start_pos: Point::new(Px(0.0), Px(0.0)),
        });

        let mut services = NullServices::default();
        let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
        let mut cx = event_cx(
            &mut host,
            &mut services,
            bounds(),
            &mut prevented_default_actions,
        );

        let pos_small = Point::new(Px((threshold_screen - eps_screen) / zoom), Px(0.0));
        assert!(edge_insert_drag::handle_pending_edge_insert_drag_move(
            &mut canvas,
            &mut cx,
            &snapshot,
            pos_small,
        ));
        assert!(canvas.interaction.edge_insert_drag.is_none());
        assert!(canvas.interaction.pending_edge_insert_drag.is_some());

        let pos_big = Point::new(Px((threshold_screen + 1.0) / zoom), Px(0.0));
        assert!(edge_insert_drag::handle_pending_edge_insert_drag_move(
            &mut canvas,
            &mut cx,
            &snapshot,
            pos_big,
        ));
        assert!(canvas.interaction.edge_insert_drag.is_some());
        assert!(canvas.interaction.pending_edge_insert_drag.is_none());

        canvas.interaction.edge_insert_drag = None;
        canvas.interaction.pending_edge_insert_drag = None;
    }
}

#[test]
fn edge_insert_left_up_does_not_open_picker_when_searcher_is_open() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(420.0);

    let edge_a = EdgeId(Uuid::from_u128(1));
    let edge_b = EdgeId(Uuid::from_u128(2));
    for edge_id in [edge_a, edge_b] {
        graph_value.edges.insert(
            edge_id,
            Edge {
                kind: EdgeKind::Data,
                from: a_out,
                to: b_in,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
    }

    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);

    let mut canvas = NodeGraphCanvas::new(graph, view);
    canvas.open_edge_insert_node_picker(&mut host, None, edge_a, Point::new(Px(10.0), Px(10.0)));
    assert!(canvas.interaction.searcher.is_some());

    canvas.interaction.edge_insert_drag = Some(EdgeInsertDrag {
        edge: edge_b,
        pos: Point::new(Px(20.0), Px(20.0)),
    });

    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds(),
        &mut prevented_default_actions,
    );

    assert!(edge_insert_drag::handle_edge_insert_left_up(
        &mut canvas,
        &mut cx,
        Point::new(Px(20.0), Px(20.0)),
    ));
    assert!(canvas.interaction.edge_insert_drag.is_none());
    assert!(canvas.interaction.searcher.is_some());
    assert!(matches!(
        canvas.interaction.searcher.as_ref().unwrap().target,
        ContextMenuTarget::EdgeInsertNodePicker(e) if e == edge_a
    ));
}

#[test]
fn double_click_edge_splits_lowest_edge_id_when_overlapping() {
    #[derive(Debug, Clone, Copy)]
    struct StraightPresenter;

    impl NodeGraphPresenter for StraightPresenter {
        fn node_title(&self, _graph: &Graph, _node: NodeId) -> Arc<str> {
            Arc::<str>::from("Node")
        }

        fn port_label(&self, _graph: &Graph, _port: PortId) -> Arc<str> {
            Arc::<str>::from("Port")
        }

        fn edge_render_hint(
            &self,
            _graph: &Graph,
            _edge: EdgeId,
            _style: &crate::ui::style::NodeGraphStyle,
        ) -> EdgeRenderHint {
            EdgeRenderHint {
                route: EdgeRouteKind::Straight,
                ..EdgeRenderHint::default()
            }
        }
    }

    fn segment_intersection(a0: Point, a1: Point, b0: Point, b1: Point) -> Option<Point> {
        let ax0 = a0.x.0;
        let ay0 = a0.y.0;
        let ax1 = a1.x.0;
        let ay1 = a1.y.0;

        let bx0 = b0.x.0;
        let by0 = b0.y.0;
        let bx1 = b1.x.0;
        let by1 = b1.y.0;

        let r_x = ax1 - ax0;
        let r_y = ay1 - ay0;
        let s_x = bx1 - bx0;
        let s_y = by1 - by0;

        let denom = r_x * s_y - r_y * s_x;
        if denom.abs() <= 1.0e-6 {
            return None;
        }

        let q_p_x = bx0 - ax0;
        let q_p_y = by0 - ay0;
        let t = (q_p_x * s_y - q_p_y * s_x) / denom;
        let u = (q_p_x * r_y - q_p_y * r_x) / denom;

        if !(0.0..=1.0).contains(&t) || !(0.0..=1.0).contains(&u) {
            return None;
        }

        Some(Point::new(Px(ax0 + t * r_x), Px(ay0 + t * r_y)))
    }

    let mut host = TestUiHostImpl::default();

    let mut graph_value = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let n1 = NodeId::new();
    let n2 = NodeId::new();
    let n3 = NodeId::new();
    let n4 = NodeId::new();

    let p1_out = PortId::new();
    let p2_in = PortId::new();
    let p3_out = PortId::new();
    let p4_in = PortId::new();

    graph_value.nodes.insert(
        n1,
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
            ports: vec![p1_out],
            data: Value::Null,
        },
    );
    graph_value.nodes.insert(
        n2,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 420.0, y: 260.0 },
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
            ports: vec![p2_in],
            data: Value::Null,
        },
    );
    graph_value.nodes.insert(
        n3,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 260.0 },
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
            ports: vec![p3_out],
            data: Value::Null,
        },
    );
    graph_value.nodes.insert(
        n4,
        Node {
            kind,
            kind_version: 1,
            pos: CanvasPoint { x: 420.0, y: 0.0 },
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
            ports: vec![p4_in],
            data: Value::Null,
        },
    );

    graph_value.ports.insert(
        p1_out,
        Port {
            node: n1,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    graph_value.ports.insert(
        p2_in,
        Port {
            node: n2,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    graph_value.ports.insert(
        p3_out,
        Port {
            node: n3,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );
    graph_value.ports.insert(
        p4_in,
        Port {
            node: n4,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: Value::Null,
        },
    );

    let edge_a = EdgeId(Uuid::from_u128(1));
    let edge_b = EdgeId(Uuid::from_u128(2));
    graph_value.edges.insert(
        edge_a,
        Edge {
            kind: EdgeKind::Data,
            from: p1_out,
            to: p2_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    graph_value.edges.insert(
        edge_b,
        Edge {
            kind: EdgeKind::Data,
            from: p3_out,
            to: p4_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);
    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.zoom_on_double_click = true;
        s.interaction.reroute_on_edge_double_click = true;
    });

    let mut canvas =
        NodeGraphCanvas::new(graph.clone(), view.clone()).with_presenter(StraightPresenter);
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds(),
        &mut prevented_default_actions,
    );

    let snap = canvas.sync_view_state(cx.app);
    let (geom, _index) = canvas.canvas_derived(&*cx.app, &snap);
    let a0 = geom.port_center(p1_out).expect("p1_out center");
    let a1 = geom.port_center(p2_in).expect("p2_in center");
    let b0 = geom.port_center(p3_out).expect("p3_out center");
    let b1 = geom.port_center(p4_in).expect("p4_in center");
    let pos = segment_intersection(a0, a1, b0, b1).expect("edges should intersect");

    canvas.event(
        &mut cx,
        &fret_core::Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 2,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let edges_len = graph.read_ref(cx.app, |g| g.edges.len()).unwrap_or(0);
    assert_eq!(edges_len, 3);

    let split_a = graph
        .read_ref(cx.app, |g| g.edges.get(&edge_a).map(|e| e.to))
        .unwrap_or(None)
        .expect("edge_a");
    let split_b = graph
        .read_ref(cx.app, |g| g.edges.get(&edge_b).map(|e| e.to))
        .unwrap_or(None)
        .expect("edge_b");

    assert_ne!(split_a, p2_in);
    assert_eq!(split_b, p4_in);
}
