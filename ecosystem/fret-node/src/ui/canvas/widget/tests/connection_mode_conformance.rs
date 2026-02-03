use fret_core::{Point, Px, Rect, Size};

use crate::interaction::NodeGraphConnectionMode;
use crate::io::NodeGraphViewState;

use super::super::super::state::{WireDrag, WireDragKind};
use super::super::NodeGraphCanvas;
use super::super::wire_drag::handle_wire_left_up_with_forced_target;
use super::super::{HitTestCtx, HitTestScratch};
use super::{
    NullServices, TestUiHostImpl, event_cx, make_test_graph_two_nodes_with_ports_spaced_x,
};
use crate::ui::presenter::NodeGraphPresenter;
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
struct SimplePresenter;

impl NodeGraphPresenter for SimplePresenter {
    fn node_title(&self, _graph: &crate::core::Graph, _node: crate::core::NodeId) -> Arc<str> {
        Arc::<str>::from("Node")
    }

    fn port_label(&self, _graph: &crate::core::Graph, _port: crate::core::PortId) -> Arc<str> {
        Arc::<str>::from("Port")
    }
}

fn pick_target_port_at(
    canvas: &mut NodeGraphCanvas,
    host: &mut TestUiHostImpl,
    snapshot: &super::super::super::state::ViewSnapshot,
    from: crate::core::PortId,
    pos: Point,
) -> Option<crate::core::PortId> {
    let (geom, index) = canvas.canvas_derived(&*host, snapshot);
    let this = canvas;
    this.graph
        .read_ref(host, |graph| {
            let mut scratch = HitTestScratch::default();
            let mut ctx =
                HitTestCtx::new(geom.as_ref(), index.as_ref(), snapshot.zoom, &mut scratch);
            this.pick_target_port(graph, snapshot, &mut ctx, from, true, pos)
        })
        .ok()
        .flatten()
}

#[test]
fn pick_target_port_loose_can_select_same_side_when_closer() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);

    let c = crate::core::NodeId::new();
    let c_out = crate::core::PortId::new();
    graph_value.nodes.insert(
        c,
        crate::core::Node {
            kind: crate::core::NodeKindKey::new("test.node"),
            kind_version: 1,
            pos: crate::core::CanvasPoint { x: 260.0, y: 60.0 },
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
        c_out,
        crate::core::Port {
            node: c,
            key: crate::core::PortKey::new("out"),
            dir: crate::core::PortDirection::Out,
            kind: crate::core::PortKind::Data,
            capacity: crate::core::PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        },
    );

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.connection_mode = NodeGraphConnectionMode::Loose;
        s.interaction.connection_radius = 80.0;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, _index) = canvas.canvas_derived(&host, &snapshot);
    let c_handle = geom.ports.get(&c_out).expect("port handle");

    let picked = pick_target_port_at(&mut canvas, &mut host, &snapshot, a_out, c_handle.center);
    assert_eq!(
        picked,
        Some(c_out),
        "loose mode should allow selecting same-side ports when closest"
    );

    let _ = b_in;
}

#[test]
fn pick_target_port_strict_rejects_same_side_even_when_inside_bounds() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, _b, _b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);

    let c = crate::core::NodeId::new();
    let c_out = crate::core::PortId::new();
    graph_value.nodes.insert(
        c,
        crate::core::Node {
            kind: crate::core::NodeKindKey::new("test.node"),
            kind_version: 1,
            pos: crate::core::CanvasPoint { x: 260.0, y: 60.0 },
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
        c_out,
        crate::core::Port {
            node: c,
            key: crate::core::PortKey::new("out"),
            dir: crate::core::PortDirection::Out,
            kind: crate::core::PortKind::Data,
            capacity: crate::core::PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        },
    );

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.connection_mode = NodeGraphConnectionMode::Strict;
        s.interaction.connection_radius = 80.0;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, _index) = canvas.canvas_derived(&host, &snapshot);
    let c_handle = geom.ports.get(&c_out).expect("port handle");

    let picked = pick_target_port_at(&mut canvas, &mut host, &snapshot, a_out, c_handle.center);
    assert_eq!(
        picked, None,
        "strict mode must reject same-side target ports"
    );
}

#[test]
fn strict_rejects_out_to_out_but_loose_commits_out_to_out_when_forced() {
    let mut host = TestUiHostImpl::default();

    let (mut graph_value, _a, _a_in, a_out, b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);

    // Replace b's input with an output-only port, so the only possible target is same-side.
    let b_port = b_in;
    graph_value.nodes.get_mut(&b).unwrap().ports = vec![b_port];
    graph_value.ports.get_mut(&b_port).unwrap().dir = crate::core::PortDirection::Out;

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas =
        NodeGraphCanvas::new(graph.clone(), view.clone()).with_presenter(SimplePresenter);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    // Strict: reject.
    {
        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.connection_mode = NodeGraphConnectionMode::Strict;
        });
        let snapshot_strict = canvas.sync_view_state(&mut host);
        canvas.interaction.wire_drag = Some(WireDrag {
            kind: WireDragKind::New {
                from: a_out,
                bundle: Vec::new(),
            },
            pos: Point::new(Px(10.0), Px(10.0)),
        });
        let mut services = NullServices::default();
        let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
        let mut cx = event_cx(
            &mut host,
            &mut services,
            bounds,
            &mut prevented_default_actions,
        );
        assert!(handle_wire_left_up_with_forced_target(
            &mut canvas,
            &mut cx,
            &snapshot_strict,
            snapshot_strict.zoom,
            Some(b_port),
        ));
    }
    let edges_after_strict = graph
        .read_ref(&mut host, |g| g.edges.len())
        .ok()
        .unwrap_or_default();
    assert_eq!(edges_after_strict, 0);

    // Loose: accept and commit.
    {
        let _ = view.update(&mut host, |s, _cx| {
            s.interaction.connection_mode = NodeGraphConnectionMode::Loose;
        });
        let snapshot_loose = canvas.sync_view_state(&mut host);
        canvas.interaction.wire_drag = Some(WireDrag {
            kind: WireDragKind::New {
                from: a_out,
                bundle: Vec::new(),
            },
            pos: Point::new(Px(10.0), Px(10.0)),
        });
        let mut services = NullServices::default();
        let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
        let mut cx = event_cx(
            &mut host,
            &mut services,
            bounds,
            &mut prevented_default_actions,
        );
        assert!(handle_wire_left_up_with_forced_target(
            &mut canvas,
            &mut cx,
            &snapshot_loose,
            snapshot_loose.zoom,
            Some(b_port),
        ));
    }

    let edges = graph
        .read_ref(&mut host, |g| g.edges.clone())
        .ok()
        .unwrap_or_default();
    assert_eq!(edges.len(), 1);
    let edge = edges.values().next().unwrap();
    assert_eq!(edge.from, a_out);
    assert_eq!(edge.to, b_port);
}
