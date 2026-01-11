use std::cell::RefCell;
use std::rc::Rc;

use fret_core::{Modifiers, MouseButton, Point, Px, Rect, Size};

use crate::io::NodeGraphViewState;
use crate::runtime::callbacks::{ConnectStart, NodeGraphCallbacks};

use super::super::super::state::{PendingWireDrag, WireDrag, WireDragKind};
use super::super::NodeGraphCanvas;
use super::{
    NullServices, TestUiHostImpl, event_cx, make_test_graph_two_nodes_with_ports_spaced_x,
};

#[derive(Clone)]
struct Recorder {
    log: Rc<RefCell<Vec<String>>>,
}

impl NodeGraphCallbacks for Recorder {
    fn on_connect_start(&mut self, ev: ConnectStart) {
        self.log.borrow_mut().push(format!("start:{:?}", ev.kind));
    }

    fn on_connect_end(&mut self, ev: crate::runtime::callbacks::ConnectEnd) {
        self.log
            .borrow_mut()
            .push(format!("end:{:?}:{:?}", ev.outcome, ev.target));
    }
}

fn make_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    )
}

#[test]
fn click_connect_emits_connect_start_and_committed_end() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.connect_on_click = true;
    });

    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone()).with_callbacks(recorder);
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = make_bounds();
    let mut services = NullServices::default();
    let mut cx = event_cx(&mut host, &mut services, bounds);

    let pos = Point::new(Px(40.0), Px(40.0));
    canvas.interaction.pending_wire_drag = Some(PendingWireDrag {
        kind: WireDragKind::New {
            from: a_out,
            bundle: Vec::new(),
        },
        start_pos: pos,
    });

    assert!(super::super::pointer_up::handle_pointer_up(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        MouseButton::Left,
        1,
        Modifiers::default(),
        snapshot.zoom,
    ));

    assert!(
        super::super::wire_drag::handle_wire_left_up_with_forced_target(
            &mut canvas,
            &mut cx,
            &snapshot,
            snapshot.zoom,
            Some(b_in),
        )
    );

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s.starts_with("start:")));
    assert!(
        got.iter()
            .any(|s| s.contains("end:Committed") && s.contains("Some"))
    );
}

#[test]
fn escape_cancel_emits_connect_end_canceled() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, a_out, _b, _b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.connect_on_click = true;
    });

    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_callbacks(recorder);
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = make_bounds();
    let mut services = NullServices::default();
    let mut cx = event_cx(&mut host, &mut services, bounds);

    let pos = Point::new(Px(40.0), Px(40.0));
    canvas.interaction.wire_drag = Some(WireDrag {
        kind: WireDragKind::New {
            from: a_out,
            bundle: Vec::new(),
        },
        pos,
    });

    super::super::cancel::handle_escape_cancel(&mut canvas, &mut cx);

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s.contains("end:Canceled")));
    let _ = snapshot;
}

#[test]
fn rejected_drop_emits_connect_end_rejected() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);

    graph_value.ports.get_mut(&b_in).unwrap().kind = crate::core::PortKind::Exec;
    graph_value.nodes.get_mut(&b).unwrap().ports = vec![b_in];

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_callbacks(recorder);
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = make_bounds();
    let mut services = NullServices::default();
    let mut cx = event_cx(&mut host, &mut services, bounds);

    canvas.interaction.wire_drag = Some(WireDrag {
        kind: WireDragKind::New {
            from: a_out,
            bundle: Vec::new(),
        },
        pos: Point::new(Px(40.0), Px(40.0)),
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

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s.contains("end:Rejected")));
}
