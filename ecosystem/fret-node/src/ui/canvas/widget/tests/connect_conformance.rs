use std::sync::Arc;

use fret_core::{Point, Px, Rect, Size};
use serde_json::Value;

use crate::core::{
    CanvasPoint, Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey, Port,
    PortCapacity, PortDirection, PortId, PortKey, PortKind,
};
use crate::interaction::NodeGraphConnectionMode;
use crate::io::NodeGraphViewState;
use crate::rules::{ConnectPlan, EdgeEndpoint, InsertNodeTemplate, PortTemplate};
use crate::ui::presenter::NodeGraphPresenter;

use super::super::super::state::{ContextMenuTarget, WireDrag, WireDragKind};
use super::super::NodeGraphCanvas;
use super::{
    NullServices, TestUiHostImpl, event_cx, make_test_graph_two_nodes_with_ports_spaced_x,
};

#[derive(Clone)]
struct RejectingConversionPresenter {
    conversions: Vec<InsertNodeTemplate>,
}

impl NodeGraphPresenter for RejectingConversionPresenter {
    fn node_title(&self, _graph: &Graph, _node: NodeId) -> Arc<str> {
        Arc::<str>::from("Node")
    }

    fn port_label(&self, _graph: &Graph, _port: PortId) -> Arc<str> {
        Arc::<str>::from("Port")
    }

    fn plan_connect(
        &mut self,
        _graph: &Graph,
        _a: PortId,
        _b: PortId,
        _mode: crate::interaction::NodeGraphConnectionMode,
    ) -> ConnectPlan {
        ConnectPlan::reject("connect rejected")
    }

    fn list_conversions(
        &mut self,
        _graph: &Graph,
        _from: PortId,
        _to: PortId,
    ) -> Vec<InsertNodeTemplate> {
        self.conversions.clone()
    }
}

fn make_conversion_template(kind: &str) -> InsertNodeTemplate {
    InsertNodeTemplate {
        kind: NodeKindKey::new(kind),
        kind_version: 1,
        collapsed: false,
        data: Value::Null,
        ports: vec![
            PortTemplate {
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                ty: None,
                data: Value::Null,
            },
            PortTemplate {
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                ty: None,
                data: Value::Null,
            },
        ],
        input: PortKey::new("in"),
        output: PortKey::new("out"),
    }
}

fn make_graph_reconnect_to_new_target() -> (Graph, EdgeId, PortId, PortId, PortId) {
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
            size: None,
            hidden: false,
            collapsed: false,
            ports: vec![out],
            data: Value::Null,
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
            data: Value::Null,
        },
    );

    let b = NodeId::new();
    let old_in = PortId::new();
    graph.nodes.insert(
        b,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 280.0, y: 0.0 },
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
            ports: vec![old_in],
            data: Value::Null,
        },
    );
    graph.ports.insert(
        old_in,
        Port {
            node: b,
            key: PortKey::new("in_old"),
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

    let c = NodeId::new();
    let new_in = PortId::new();
    graph.nodes.insert(
        c,
        Node {
            kind,
            kind_version: 1,
            pos: CanvasPoint { x: 280.0, y: 160.0 },
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
            ports: vec![new_in],
            data: Value::Null,
        },
    );
    graph.ports.insert(
        new_in,
        Port {
            node: c,
            key: PortKey::new("in_new"),
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

    let edge = EdgeId::new();
    graph.edges.insert(
        edge,
        Edge {
            kind: EdgeKind::Data,
            from: out,
            to: old_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    (graph, edge, out, old_in, new_in)
}

#[test]
fn connect_drop_opens_conversion_picker_when_multiple_conversions() {
    for mode in [
        NodeGraphConnectionMode::Strict,
        NodeGraphConnectionMode::Loose,
    ] {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _a_in, a_out, _b, b_in) =
            make_test_graph_two_nodes_with_ports_spaced_x(260.0);
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.connection_mode = mode;
        });

        let presenter = RejectingConversionPresenter {
            conversions: vec![
                make_conversion_template("test.conv.a"),
                make_conversion_template("test.conv.b"),
            ],
        };

        let mut canvas =
            NodeGraphCanvas::new(graph.clone(), view.clone()).with_presenter(presenter);
        let snapshot = canvas.sync_view_state(&mut host);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
        let mut cx = event_cx(
            &mut host,
            &mut services,
            bounds,
            &mut prevented_default_actions,
        );
        let pos = Point::new(Px(400.0), Px(300.0));
        canvas.interaction.wire_drag = Some(WireDrag {
            kind: WireDragKind::New {
                from: a_out,
                bundle: Vec::new(),
            },
            pos,
        });

        assert!(
            super::super::wire_drag::handle_wire_left_up_with_forced_target(
                &mut canvas,
                &mut cx,
                &snapshot,
                snapshot.zoom,
                Some(b_in),
            )
        );

        assert!(canvas.interaction.suspended_wire_drag.is_some());

        let searcher = canvas
            .interaction
            .searcher
            .as_ref()
            .expect("conversion picker should open");
        match &searcher.target {
            ContextMenuTarget::ConnectionConvertPicker { from, to, at } => {
                assert_eq!(*from, a_out);
                assert_eq!(*to, b_in);
                assert!((at.x - pos.x.0).abs() <= 1.0e-3);
                assert!((at.y - pos.y.0).abs() <= 1.0e-3);
            }
            other => panic!("unexpected searcher target: {other:?}"),
        }
        assert_eq!(searcher.candidates.len(), 2);

        let edges = graph.read_ref(&host, |g| g.edges.len()).unwrap_or_default();
        assert_eq!(edges, 0);
    }
}

#[test]
fn connect_drop_auto_inserts_conversion_when_single_choice() {
    for mode in [
        NodeGraphConnectionMode::Strict,
        NodeGraphConnectionMode::Loose,
    ] {
        let mut host = TestUiHostImpl::default();
        let (graph_value, _a, _a_in, a_out, _b, b_in) =
            make_test_graph_two_nodes_with_ports_spaced_x(260.0);
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(NodeGraphViewState::default());

        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.connection_mode = mode;
        });

        let presenter = RejectingConversionPresenter {
            conversions: vec![make_conversion_template("test.conv")],
        };

        let mut canvas =
            NodeGraphCanvas::new(graph.clone(), view.clone()).with_presenter(presenter);
        let snapshot = canvas.sync_view_state(&mut host);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = NullServices::default();
        let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
        let mut cx = event_cx(
            &mut host,
            &mut services,
            bounds,
            &mut prevented_default_actions,
        );
        let pos = Point::new(Px(400.0), Px(300.0));
        canvas.interaction.wire_drag = Some(WireDrag {
            kind: WireDragKind::New {
                from: a_out,
                bundle: Vec::new(),
            },
            pos,
        });

        assert!(
            super::super::wire_drag::handle_wire_left_up_with_forced_target(
                &mut canvas,
                &mut cx,
                &snapshot,
                snapshot.zoom,
                Some(b_in),
            )
        );

        assert!(canvas.interaction.searcher.is_none());
        assert!(canvas.interaction.suspended_wire_drag.is_none());

        let (nodes, edges, conv_node, conv_in, conv_out) = graph
            .read_ref(&host, |g| {
                let conv_kind = NodeKindKey::new("test.conv");
                let conv_node = g
                    .nodes
                    .iter()
                    .find_map(|(id, n)| (n.kind == conv_kind).then_some(*id));
                let Some(conv_node) = conv_node else {
                    return (g.nodes.len(), g.edges.len(), None, None, None);
                };

                let mut conv_in: Option<PortId> = None;
                let mut conv_out: Option<PortId> = None;
                if let Some(node) = g.nodes.get(&conv_node) {
                    for port_id in node.ports.iter().copied() {
                        let Some(p) = g.ports.get(&port_id) else {
                            continue;
                        };
                        if p.key.0 == "in" {
                            conv_in = Some(port_id);
                        }
                        if p.key.0 == "out" {
                            conv_out = Some(port_id);
                        }
                    }
                }

                (
                    g.nodes.len(),
                    g.edges.len(),
                    Some(conv_node),
                    conv_in,
                    conv_out,
                )
            })
            .unwrap_or_default();

        assert_eq!(nodes, 3);
        assert_eq!(edges, 2);

        let conv_node = conv_node.expect("conversion node should be inserted");
        let conv_in = conv_in.expect("conversion input port should exist");
        let conv_out = conv_out.expect("conversion output port should exist");

        let (has_upstream, has_downstream) = graph
            .read_ref(&host, |g| {
                let mut has_upstream = false;
                let mut has_downstream = false;
                for e in g.edges.values() {
                    if e.from == a_out && e.to == conv_in {
                        has_upstream = true;
                    }
                    if e.from == conv_out && e.to == b_in {
                        has_downstream = true;
                    }
                }
                (has_upstream, has_downstream)
            })
            .unwrap_or_default();

        assert!(has_upstream, "expected upstream edge a_out -> conv_in");
        assert!(has_downstream, "expected downstream edge conv_out -> b_in");

        let _ = conv_node;
    }
}

#[test]
fn reconnect_preserves_edge_identity_and_updates_endpoint() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, edge, from, old_to, new_to) = make_graph_reconnect_to_new_target();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.edges_reconnectable = true;
    });

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let pos = Point::new(Px(420.0), Px(280.0));
    canvas.interaction.wire_drag = Some(WireDrag {
        kind: WireDragKind::Reconnect {
            edge,
            endpoint: EdgeEndpoint::To,
            fixed: from,
        },
        pos,
    });

    assert!(
        super::super::wire_drag::handle_wire_left_up_with_forced_target(
            &mut canvas,
            &mut cx,
            &snapshot,
            snapshot.zoom,
            Some(new_to),
        )
    );

    let (edge_count, from_after, to_after) = graph
        .read_ref(&host, |g| {
            let Some(e) = g.edges.get(&edge) else {
                return (g.edges.len(), None, None);
            };
            (g.edges.len(), Some(e.from), Some(e.to))
        })
        .unwrap_or_default();

    assert_eq!(edge_count, 1);
    assert_eq!(from_after, Some(from));
    assert_eq!(to_after, Some(new_to));

    let _ = old_to;
}
