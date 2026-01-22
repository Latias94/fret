use std::sync::Arc;

use fret_core::{AppWindowId, MouseButtons, Point, PointerId, Px, Rect, Size};
use serde_json::Value;

use crate::ui::presenter::InsertNodeCandidate;

use super::super::super::state::PendingInsertNodeDrag;
use super::super::NodeGraphCanvas;
use super::super::insert_node_drag::InsertNodeDragPayload;
use super::{NullServices, TestUiHostImpl, event_cx, make_test_graph_two_nodes};

#[test]
fn insert_node_drag_does_not_start_until_threshold() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(crate::io::NodeGraphViewState::default());
    let mut canvas = NodeGraphCanvas::new(graph, view);
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
    let window = AppWindowId::default();
    let pointer_id = PointerId(0);
    cx.window = Some(window);
    cx.pointer_id = Some(pointer_id);

    canvas.interaction.pending_insert_node_drag = Some(PendingInsertNodeDrag {
        candidate: InsertNodeCandidate {
            kind: crate::core::NodeKindKey::new("test.node"),
            label: Arc::<str>::from("Test"),
            enabled: true,
            template: None,
            payload: Value::Null,
        },
        start_pos: Point::new(Px(0.0), Px(0.0)),
        pointer_id,
        start_tick: fret_runtime::TickId(0),
    });

    let started = super::super::insert_node_drag::handle_pending_insert_node_drag_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(5.0), Px(0.0)),
        MouseButtons {
            left: true,
            ..Default::default()
        },
        1.0,
    );
    assert!(!started);
    assert!(host.drag.is_none());
    assert!(canvas.interaction.pending_insert_node_drag.is_some());
}

#[test]
fn insert_node_drag_starts_after_threshold() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(crate::io::NodeGraphViewState::default());
    let mut canvas = NodeGraphCanvas::new(graph, view);
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
    let window = AppWindowId::default();
    let pointer_id = PointerId(0);
    cx.window = Some(window);
    cx.pointer_id = Some(pointer_id);

    canvas.interaction.pending_insert_node_drag = Some(PendingInsertNodeDrag {
        candidate: InsertNodeCandidate {
            kind: crate::core::NodeKindKey::new("test.node"),
            label: Arc::<str>::from("Test"),
            enabled: true,
            template: None,
            payload: Value::Null,
        },
        start_pos: Point::new(Px(0.0), Px(0.0)),
        pointer_id,
        start_tick: fret_runtime::TickId(0),
    });

    let started = super::super::insert_node_drag::handle_pending_insert_node_drag_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(6.0), Px(0.0)),
        MouseButtons {
            left: true,
            ..Default::default()
        },
        1.0,
    );
    assert!(started);
    assert!(canvas.interaction.pending_insert_node_drag.is_none());

    let drag = host.drag.as_ref().expect("drag should start");
    assert!(drag.cross_window_hover);
    assert_eq!(drag.pointer_id, pointer_id);
    assert_eq!(
        drag.kind,
        super::super::insert_node_drag::DRAG_KIND_INSERT_NODE
    );
    assert_eq!(drag.start_position, Point::new(Px(0.0), Px(0.0)));
    assert!(drag.dragging);
    assert!(drag.payload::<InsertNodeDragPayload>().is_some());
    assert!(host.redraw.contains(&window));
}
