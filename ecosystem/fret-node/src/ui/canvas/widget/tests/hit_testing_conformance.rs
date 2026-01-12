use fret_core::{Point, Px};

use crate::interaction::NodeGraphConnectionMode;
use crate::io::NodeGraphViewState;

use super::super::NodeGraphCanvas;
use super::{TestUiHostImpl, make_test_graph_two_nodes_with_ports_spaced_x};

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
            let mut scratch_ports: Vec<crate::core::PortId> = Vec::new();
            this.pick_target_port(
                graph,
                snapshot,
                geom.as_ref(),
                index.as_ref(),
                from,
                true,
                pos,
                snapshot.zoom,
                &mut scratch_ports,
            )
        })
        .ok()
        .flatten()
}

#[test]
fn strict_requires_pointer_inside_pin_bounds_while_loose_accepts_radius() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());

    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, _index) = canvas.canvas_derived(&host, &snapshot);
    let handle = geom
        .ports
        .get(&b_in)
        .expect("target port handle should exist");

    let inside = handle.center;
    let r = canvas.style.pin_radius / snapshot.zoom.max(1.0e-6);
    let outside_but_near = Point::new(Px(handle.bounds.origin.x.0 - 0.5 * r), inside.y);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.connection_mode = NodeGraphConnectionMode::Strict;
        s.interaction.connection_radius = 24.0;
    });
    let snapshot_strict = canvas.sync_view_state(&mut host);
    assert_eq!(
        pick_target_port_at(
            &mut canvas,
            &mut host,
            &snapshot_strict,
            a_out,
            outside_but_near
        ),
        None
    );
    assert_eq!(
        pick_target_port_at(&mut canvas, &mut host, &snapshot_strict, a_out, inside),
        Some(b_in)
    );

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.connection_mode = NodeGraphConnectionMode::Loose;
        s.interaction.connection_radius = 24.0;
    });
    let snapshot_loose = canvas.sync_view_state(&mut host);
    assert_eq!(
        pick_target_port_at(
            &mut canvas,
            &mut host,
            &snapshot_loose,
            a_out,
            outside_but_near
        ),
        Some(b_in)
    );
}

#[test]
fn loose_mode_prefers_opposite_side_when_handles_overlap() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, a, a_in, a_out, b, b_in) =
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
        s.interaction.connection_radius = 48.0;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, _index) = canvas.canvas_derived(&host, &snapshot);

    let b_handle = geom
        .ports
        .get(&b_in)
        .expect("target input port handle should exist");
    let c_handle = geom
        .ports
        .get(&c_out)
        .expect("same-side port handle should exist");
    let delta_x = b_handle.center.x.0 - c_handle.center.x.0;
    let delta_y = b_handle.center.y.0 - c_handle.center.y.0;

    let _ = canvas.graph.update(&mut host, |g, _cx| {
        if let Some(node) = g.nodes.get_mut(&c) {
            node.pos.x += delta_x;
            node.pos.y += delta_y;
        }
    });

    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, _index) = canvas.canvas_derived(&host, &snapshot);
    let c_handle = geom
        .ports
        .get(&c_out)
        .expect("same-side port handle should exist after move");
    let pos = c_handle.center;

    let picked = pick_target_port_at(&mut canvas, &mut host, &snapshot, a_out, pos);
    assert_eq!(
        picked,
        Some(b_in),
        "expected loose pick to prefer opposite-side handle when handles overlap"
    );

    let _ = (a, a_in, b);
}
