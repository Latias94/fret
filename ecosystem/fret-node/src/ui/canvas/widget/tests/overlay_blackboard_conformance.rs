use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fret_core::{
    AppWindowId, Event, Modifiers, MouseButton, Point, PointerEvent, PointerType, Px, Rect, Size,
};
use fret_runtime::ModelsHost as _;
use fret_ui::UiTree;
use fret_ui::retained_bridge::UiTreeRetainedExt as _;

use crate::core::{
    CanvasPoint, CanvasSize, Graph, GraphId, Node, NodeId, NodeKindKey, Symbol, SymbolId,
};
use crate::core::{SYMBOL_REF_NODE_KIND, symbol_ref_node_data};
use crate::io::NodeGraphViewState;
use crate::ops::{GraphOp, GraphTransaction};
use crate::ui::{
    NodeGraphBlackboardOverlay, NodeGraphEditQueue, NodeGraphEditor, NodeGraphOverlayHost,
    NodeGraphOverlayState, NodeGraphStyle,
};

use super::{NullServices, TestUiHostImpl};

#[derive(Clone)]
struct PointerDownCounter {
    count: Arc<AtomicUsize>,
}

impl PointerDownCounter {
    fn new(count: Arc<AtomicUsize>) -> Self {
        Self { count }
    }
}

impl<H: fret_ui::UiHost> fret_ui::retained_bridge::Widget<H> for PointerDownCounter {
    fn hit_test(&self, bounds: Rect, position: Point) -> bool {
        bounds.contains(position)
    }

    fn event(&mut self, cx: &mut fret_ui::retained_bridge::EventCx<'_, H>, event: &Event) {
        let Event::Pointer(PointerEvent::Down { button, .. }) = event else {
            return;
        };
        if *button == MouseButton::Left {
            self.count.fetch_add(1, Ordering::Relaxed);
            cx.stop_propagation();
        }
    }

    fn layout(&mut self, cx: &mut fret_ui::retained_bridge::LayoutCx<'_, H>) -> Size {
        cx.bounds.size
    }
}

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    )
}

#[test]
fn blackboard_overlay_is_hit_test_transparent_outside_panel() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let graph = host.models.insert(Graph::new(GraphId::new()));
    let view = host.models.insert(NodeGraphViewState::default());
    let edits = host.models.insert(NodeGraphEditQueue::default());
    let overlays = host.models.insert(NodeGraphOverlayState::default());
    let style = NodeGraphStyle::default();

    let underlay_downs = Arc::new(AtomicUsize::new(0));
    let underlay = ui.create_node_retained(PointerDownCounter::new(underlay_downs.clone()));

    let overlay = NodeGraphBlackboardOverlay::new(graph, view, edits, overlays, underlay, style);
    let overlay_node = ui.create_node_retained(overlay);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![underlay, overlay_node]);
    ui.set_root(editor);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    // Outside the overlay panel.
    let outside = Point::new(Px(780.0), Px(580.0));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: outside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    assert_eq!(underlay_downs.load(Ordering::Relaxed), 1);
}

#[test]
fn blackboard_overlay_enter_defaults_to_add_symbol_when_focused() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let graph = host.models.insert(Graph::new(GraphId::new()));
    let view = host.models.insert(NodeGraphViewState::default());
    let edits = host.models.insert(NodeGraphEditQueue::default());
    let overlays = host.models.insert(NodeGraphOverlayState::default());
    let style = NodeGraphStyle::default();

    let underlay = ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    let overlay =
        NodeGraphBlackboardOverlay::new(graph, view, edits.clone(), overlays, underlay, style);
    let overlay_node = ui.create_node_retained(overlay);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![underlay, overlay_node]);
    ui.set_root(editor);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    ui.set_focus(Some(overlay_node));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let pending = edits
        .read_ref(&host, |q| q.pending.clone())
        .ok()
        .unwrap_or_default();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].label.as_deref(), Some("Add Symbol"));
    assert_eq!(pending[0].ops.len(), 1);
    assert!(
        matches!(&pending[0].ops[0], GraphOp::AddSymbol { .. }),
        "expected AddSymbol op"
    );
}

#[test]
fn blackboard_overlay_can_insert_symbol_ref_for_selected_symbol() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let graph = host.models.insert(Graph::new(GraphId::new()));
    let view = host.models.insert(NodeGraphViewState::default());
    let edits = host.models.insert(NodeGraphEditQueue::default());
    let overlays = host.models.insert(NodeGraphOverlayState::default());
    let style = NodeGraphStyle::default();

    let symbol_id = SymbolId::new();
    let _ = host.models.update(&graph, |g| {
        g.symbols.insert(
            symbol_id,
            Symbol {
                name: "foo".to_string(),
                ty: None,
                default_value: None,
                meta: serde_json::Value::Null,
            },
        );
    });

    let pan = CanvasPoint { x: 50.0, y: -20.0 };
    let zoom = 2.0;
    let _ = host.models.update(&view, |s| {
        s.pan = pan;
        s.zoom = zoom;
    });

    let canvas = ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    let overlay =
        NodeGraphBlackboardOverlay::new(graph, view, edits.clone(), overlays, canvas, style);
    let overlay_node = ui.create_node_retained(overlay);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![canvas, overlay_node]);
    ui.set_root(editor);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    ui.set_focus(Some(overlay_node));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let pending = edits
        .read_ref(&host, |q| q.pending.clone())
        .ok()
        .unwrap_or_default();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].label.as_deref(), Some("Insert Symbol Ref"));
    assert_eq!(pending[0].ops.len(), 1);

    let GraphOp::AddNode { node, .. } = &pending[0].ops[0] else {
        panic!("expected AddNode op");
    };
    assert_eq!(node.kind, NodeKindKey::new(SYMBOL_REF_NODE_KIND));
    assert_eq!(
        node.data.get("symbol_id"),
        Some(&serde_json::json!(symbol_id))
    );

    let expected_center = CanvasPoint {
        x: 400.0 / zoom - pan.x,
        y: 300.0 / zoom - pan.y,
    };
    assert!(
        (node.pos.x - expected_center.x).abs() <= 1.0e-3,
        "expected node.pos.x ~= {}, got {}",
        expected_center.x,
        node.pos.x
    );
    assert!(
        (node.pos.y - expected_center.y).abs() <= 1.0e-3,
        "expected node.pos.y ~= {}, got {}",
        expected_center.y,
        node.pos.y
    );
}

#[test]
fn blackboard_overlay_delete_removes_symbol_ref_nodes_before_removing_symbol() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let graph = host.models.insert(Graph::new(GraphId::new()));
    let view = host.models.insert(NodeGraphViewState::default());
    let edits = host.models.insert(NodeGraphEditQueue::default());
    let overlays = host.models.insert(NodeGraphOverlayState::default());
    let style = NodeGraphStyle::default();

    let symbol_id = SymbolId::new();
    let ref_node_id = NodeId::new();
    let _ = host.models.update(&graph, |g| {
        g.symbols.insert(
            symbol_id,
            Symbol {
                name: "foo".to_string(),
                ty: None,
                default_value: None,
                meta: serde_json::Value::Null,
            },
        );
        g.nodes.insert(
            ref_node_id,
            Node {
                kind: NodeKindKey::new(SYMBOL_REF_NODE_KIND),
                kind_version: 1,
                pos: CanvasPoint::default(),
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 140.0,
                    height: 40.0,
                }),
                hidden: false,
                collapsed: false,
                ports: Vec::new(),
                data: symbol_ref_node_data(symbol_id),
            },
        );
    });

    let canvas = ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    let overlay =
        NodeGraphBlackboardOverlay::new(graph, view, edits.clone(), overlays, canvas, style);
    let overlay_node = ui.create_node_retained(overlay);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![canvas, overlay_node]);
    ui.set_root(editor);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    ui.set_focus(Some(overlay_node));
    for _ in 0..3 {
        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: fret_core::KeyCode::ArrowDown,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
    }
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let pending = edits
        .read_ref(&host, |q| q.pending.clone())
        .ok()
        .unwrap_or_default();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].label.as_deref(), Some("Delete Symbol"));
    assert!(
        pending[0].ops.len() >= 2,
        "expected at least a RemoveNode and RemoveSymbol op"
    );

    assert!(
        matches!(
            &pending[0].ops[0],
            GraphOp::RemoveNode { id, .. } if *id == ref_node_id
        ),
        "expected first op to remove symbol ref node"
    );
    assert!(
        matches!(
            pending[0].ops.last(),
            Some(GraphOp::RemoveSymbol { id, .. }) if *id == symbol_id
        ),
        "expected last op to remove symbol"
    );
}

#[test]
fn blackboard_overlay_rename_action_opens_symbol_rename_overlay() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let graph = host.models.insert(Graph::new(GraphId::new()));
    let view = host.models.insert(NodeGraphViewState::default());
    let edits = host.models.insert(NodeGraphEditQueue::default());
    let overlays = host.models.insert(NodeGraphOverlayState::default());
    let rename_text = host.models.insert(String::new());
    let style = NodeGraphStyle::default();

    let symbol_id = SymbolId::new();
    let _ = host.models.update(&graph, |g| {
        g.symbols.insert(
            symbol_id,
            Symbol {
                name: "foo".to_string(),
                ty: None,
                default_value: None,
                meta: serde_json::Value::Null,
            },
        );
    });

    let canvas = ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    let blackboard = NodeGraphBlackboardOverlay::new(
        graph.clone(),
        view,
        edits.clone(),
        overlays.clone(),
        canvas,
        style.clone(),
    );
    let blackboard_node = ui.create_node_retained(blackboard);

    let overlay_host = NodeGraphOverlayHost::new(
        graph,
        edits.clone(),
        overlays.clone(),
        rename_text,
        canvas,
        style,
    );
    let overlay_host_node = ui.create_node_retained(overlay_host);
    let overlay_child =
        ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    ui.set_children(overlay_host_node, vec![overlay_child]);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![canvas, blackboard_node, overlay_host_node]);
    ui.set_root(editor);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    ui.set_focus(Some(blackboard_node));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    let opened_symbol = overlays
        .read_ref(&host, |s| s.symbol_rename.as_ref().map(|o| o.symbol))
        .ok()
        .flatten();
    assert_eq!(opened_symbol, Some(symbol_id));
    assert_eq!(
        ui.focus(),
        Some(overlay_child),
        "expected symbol rename overlay host to grab focus after blackboard rename action"
    );

    let pending = edits
        .read_ref(&host, |q| q.pending.clone())
        .ok()
        .unwrap_or_default();
    assert!(
        pending.is_empty(),
        "expected rename action to open overlay only, without queueing graph transactions"
    );
}

#[test]
fn blackboard_overlay_rename_action_then_enter_commits_symbol_rename() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let graph = host.models.insert(Graph::new(GraphId::new()));
    let view = host.models.insert(NodeGraphViewState::default());
    let edits = host.models.insert(NodeGraphEditQueue::default());
    let overlays = host.models.insert(NodeGraphOverlayState::default());
    let rename_text = host.models.insert(String::new());
    let style = NodeGraphStyle::default();

    let symbol_id = SymbolId::new();
    let _ = host.models.update(&graph, |g| {
        g.symbols.insert(
            symbol_id,
            Symbol {
                name: "foo".to_string(),
                ty: None,
                default_value: None,
                meta: serde_json::Value::Null,
            },
        );
    });

    let canvas = ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    let blackboard = NodeGraphBlackboardOverlay::new(
        graph.clone(),
        view,
        edits.clone(),
        overlays.clone(),
        canvas,
        style.clone(),
    );
    let blackboard_node = ui.create_node_retained(blackboard);

    let overlay_host = NodeGraphOverlayHost::new(
        graph,
        edits.clone(),
        overlays.clone(),
        rename_text.clone(),
        canvas,
        style,
    );
    let overlay_host_node = ui.create_node_retained(overlay_host);
    let overlay_child =
        ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    ui.set_children(overlay_host_node, vec![overlay_child]);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![canvas, blackboard_node, overlay_host_node]);
    ui.set_root(editor);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    ui.set_focus(Some(blackboard_node));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    assert_eq!(
        ui.focus(),
        Some(overlay_child),
        "expected symbol rename overlay host to grab focus after blackboard rename action"
    );

    let _ = rename_text.update(&mut host, |t, _cx| {
        *t = "bar".to_string();
    });
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    let is_closed = overlays
        .read_ref(&host, |s| s.symbol_rename.is_none())
        .ok()
        .unwrap_or(false);
    assert!(is_closed, "expected Enter to close symbol rename overlay");
    assert_eq!(
        ui.focus(),
        Some(canvas),
        "expected focus to return to canvas after symbol rename commit"
    );

    let pending = edits
        .read_ref(&host, |q| q.pending.clone())
        .ok()
        .unwrap_or_default();
    assert_eq!(pending.len(), 1, "expected one queued transaction");

    let GraphTransaction { label, ops } = &pending[0];
    assert_eq!(label.as_deref(), Some("Rename Symbol"));
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        GraphOp::SetSymbolName { id, from, to } => {
            assert_eq!(*id, symbol_id);
            assert_eq!(from, "foo");
            assert_eq!(to, "bar");
        }
        other => panic!("unexpected op: {other:?}"),
    }
}

#[test]
fn blackboard_overlay_rename_action_then_escape_cancels_without_transaction() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let graph = host.models.insert(Graph::new(GraphId::new()));
    let view = host.models.insert(NodeGraphViewState::default());
    let edits = host.models.insert(NodeGraphEditQueue::default());
    let overlays = host.models.insert(NodeGraphOverlayState::default());
    let rename_text = host.models.insert(String::new());
    let style = NodeGraphStyle::default();

    let symbol_id = SymbolId::new();
    let _ = host.models.update(&graph, |g| {
        g.symbols.insert(
            symbol_id,
            Symbol {
                name: "foo".to_string(),
                ty: None,
                default_value: None,
                meta: serde_json::Value::Null,
            },
        );
    });

    let canvas = ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    let blackboard = NodeGraphBlackboardOverlay::new(
        graph.clone(),
        view,
        edits.clone(),
        overlays.clone(),
        canvas,
        style.clone(),
    );
    let blackboard_node = ui.create_node_retained(blackboard);

    let overlay_host = NodeGraphOverlayHost::new(
        graph,
        edits.clone(),
        overlays.clone(),
        rename_text.clone(),
        canvas,
        style,
    );
    let overlay_host_node = ui.create_node_retained(overlay_host);
    let overlay_child =
        ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    ui.set_children(overlay_host_node, vec![overlay_child]);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![canvas, blackboard_node, overlay_host_node]);
    ui.set_root(editor);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    ui.set_focus(Some(blackboard_node));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    assert_eq!(
        ui.focus(),
        Some(overlay_child),
        "expected symbol rename overlay host to grab focus after blackboard rename action"
    );

    let _ = rename_text.update(&mut host, |t, _cx| {
        *t = "bar".to_string();
    });
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    let is_closed = overlays
        .read_ref(&host, |s| s.symbol_rename.is_none())
        .ok()
        .unwrap_or(false);
    assert!(
        is_closed,
        "expected Escape to close symbol rename overlay after handoff"
    );
    assert_eq!(
        ui.focus(),
        Some(canvas),
        "expected focus to return to canvas after symbol rename cancel"
    );

    let pending = edits
        .read_ref(&host, |q| q.pending.clone())
        .ok()
        .unwrap_or_default();
    assert!(
        pending.is_empty(),
        "expected Escape after handoff to cancel without queueing graph transactions"
    );
}

#[test]
fn blackboard_overlay_rename_action_then_enter_unchanged_closes_without_transaction() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let graph = host.models.insert(Graph::new(GraphId::new()));
    let view = host.models.insert(NodeGraphViewState::default());
    let edits = host.models.insert(NodeGraphEditQueue::default());
    let overlays = host.models.insert(NodeGraphOverlayState::default());
    let rename_text = host.models.insert(String::new());
    let style = NodeGraphStyle::default();

    let symbol_id = SymbolId::new();
    let _ = host.models.update(&graph, |g| {
        g.symbols.insert(
            symbol_id,
            Symbol {
                name: "foo".to_string(),
                ty: None,
                default_value: None,
                meta: serde_json::Value::Null,
            },
        );
    });

    let canvas = ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    let blackboard = NodeGraphBlackboardOverlay::new(
        graph.clone(),
        view,
        edits.clone(),
        overlays.clone(),
        canvas,
        style.clone(),
    );
    let blackboard_node = ui.create_node_retained(blackboard);

    let overlay_host = NodeGraphOverlayHost::new(
        graph,
        edits.clone(),
        overlays.clone(),
        rename_text,
        canvas,
        style,
    );
    let overlay_host_node = ui.create_node_retained(overlay_host);
    let overlay_child =
        ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    ui.set_children(overlay_host_node, vec![overlay_child]);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![canvas, blackboard_node, overlay_host_node]);
    ui.set_root(editor);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    ui.set_focus(Some(blackboard_node));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    assert_eq!(
        ui.focus(),
        Some(overlay_child),
        "expected symbol rename overlay host to grab focus after blackboard rename action"
    );

    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    let is_closed = overlays
        .read_ref(&host, |s| s.symbol_rename.is_none())
        .ok()
        .unwrap_or(false);
    assert!(
        is_closed,
        "expected Enter to close symbol rename overlay after handoff"
    );
    assert_eq!(
        ui.focus(),
        Some(canvas),
        "expected focus to return to canvas after symbol rename no-op"
    );

    let pending = edits
        .read_ref(&host, |q| q.pending.clone())
        .ok()
        .unwrap_or_default();
    assert!(
        pending.is_empty(),
        "expected Enter with unchanged text after handoff to close without queueing graph transactions"
    );
}
