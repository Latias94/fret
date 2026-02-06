use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fret_core::{
    AppWindowId, Event, Modifiers, MouseButton, Point, PointerEvent, PointerType, Px, Rect, Size,
};
use fret_ui::UiTree;
use fret_ui::retained_bridge::UiTreeRetainedExt as _;

use crate::core::{
    CanvasPoint, CanvasSize, Graph, GraphId, Node, NodeId, NodeKindKey, Symbol, SymbolId,
};
use crate::core::{SYMBOL_REF_NODE_KIND, symbol_ref_node_data};
use crate::io::NodeGraphViewState;
use crate::ops::GraphOp;
use crate::ui::{
    NodeGraphBlackboardOverlay, NodeGraphEditQueue, NodeGraphEditor, NodeGraphOverlayState,
    NodeGraphStyle,
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
