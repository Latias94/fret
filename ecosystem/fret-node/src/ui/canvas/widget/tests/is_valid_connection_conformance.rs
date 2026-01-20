use fret_core::{Modifiers, Point, Px, Rect, Size};

use crate::core::{CanvasPoint, Port, PortCapacity, PortDirection, PortId, PortKey, PortKind};
use crate::io::NodeGraphViewState;
use crate::ui::NodeGraphCanvas;

use super::super::super::state::{WireDrag, WireDragKind};
use super::{
    NullServices, TestUiHostImpl, event_cx, make_test_graph_two_nodes_with_ports_spaced_x,
};

#[test]
fn wire_drag_hover_tracks_invalid_port_in_strict_mode() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (mut graph_value, _a, _a_in, a_out, b, _b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(200.0);
    let b_out = PortId::new();
    graph_value
        .nodes
        .get_mut(&b)
        .expect("node b exists")
        .ports
        .push(b_out);
    graph_value.ports.insert(
        b_out,
        Port {
            node: b,
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

    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());
    let _ = view.update(&mut host, |s, _cx| {
        s.pan = CanvasPoint::default();
        s.zoom = 1.0;
        s.interaction.connection_mode = crate::interaction::NodeGraphConnectionMode::Strict;
        s.interaction.frame_view_duration_ms = 0;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, _index) = canvas.canvas_derived(&host, &snapshot);
    let pos = geom.ports.get(&b_out).expect("b_out handle exists").center;

    canvas.interaction.wire_drag = Some(WireDrag {
        kind: WireDragKind::New {
            from: a_out,
            bundle: Vec::new(),
        },
        pos,
    });

    let mut services = NullServices::default();
    let mut cx = event_cx(&mut host, &mut services, bounds);
    assert!(super::super::wire_drag::handle_wire_drag_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        Modifiers::default(),
        1.0
    ));

    assert_eq!(canvas.interaction.hover_port, Some(b_out));
    assert!(!canvas.interaction.hover_port_valid);
    assert!(!canvas.interaction.hover_port_convertible);
}

#[test]
fn wire_drag_hover_tracks_non_connectable_end_port_as_invalid() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (mut graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(200.0);
    graph_value
        .ports
        .entry(b_in)
        .and_modify(|p| p.connectable_end = Some(false));

    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());
    let _ = view.update(&mut host, |s, _cx| {
        s.pan = CanvasPoint::default();
        s.zoom = 1.0;
        s.interaction.connection_mode = crate::interaction::NodeGraphConnectionMode::Strict;
        s.interaction.frame_view_duration_ms = 0;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, _index) = canvas.canvas_derived(&host, &snapshot);
    let pos = geom.ports.get(&b_in).expect("b_in handle exists").center;

    canvas.interaction.wire_drag = Some(WireDrag {
        kind: WireDragKind::New {
            from: a_out,
            bundle: Vec::new(),
        },
        pos,
    });

    let mut services = NullServices::default();
    let mut cx = event_cx(&mut host, &mut services, bounds);
    assert!(super::super::wire_drag::handle_wire_drag_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        Modifiers::default(),
        1.0
    ));

    assert_eq!(canvas.interaction.hover_port, Some(b_in));
    assert!(!canvas.interaction.hover_port_valid);
    assert!(!canvas.interaction.hover_port_convertible);
}

#[test]
fn wire_drag_hover_marks_valid_target_port_as_valid() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(200.0);

    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());
    let _ = view.update(&mut host, |s, _cx| {
        s.pan = CanvasPoint::default();
        s.zoom = 1.0;
        s.interaction.connection_mode = crate::interaction::NodeGraphConnectionMode::Strict;
        s.interaction.frame_view_duration_ms = 0;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, _index) = canvas.canvas_derived(&host, &snapshot);
    let pos = geom.ports.get(&b_in).expect("b_in handle exists").center;

    canvas.interaction.wire_drag = Some(WireDrag {
        kind: WireDragKind::New {
            from: a_out,
            bundle: Vec::new(),
        },
        pos,
    });

    let mut services = NullServices::default();
    let mut cx = event_cx(&mut host, &mut services, bounds);
    assert!(super::super::wire_drag::handle_wire_drag_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        Modifiers::default(),
        1.0
    ));

    assert_eq!(canvas.interaction.hover_port, Some(b_in));
    assert!(canvas.interaction.hover_port_valid);
}
