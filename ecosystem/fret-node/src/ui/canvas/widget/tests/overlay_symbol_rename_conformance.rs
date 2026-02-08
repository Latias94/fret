use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fret_core::{AppWindowId, Event, KeyCode, Modifiers, Point, Px, Rect, Size};
use fret_runtime::ModelsHost as _;
use fret_ui::UiTree;
use fret_ui::retained_bridge::UiTreeRetainedExt as _;

use crate::core::{Graph, GraphId, Symbol, SymbolId};
use crate::ops::{GraphOp, GraphTransaction};
use crate::ui::{
    NodeGraphEditQueue, NodeGraphEditor, NodeGraphOverlayHost, NodeGraphOverlayState,
    NodeGraphStyle, SymbolRenameOverlay,
};

use super::{NullServices, TestUiHostImpl, insert_graph_view};

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
        if let Event::Pointer(fret_core::PointerEvent::Down {
            button: fret_core::MouseButton::Left,
            ..
        }) = event
        {
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

fn open_symbol_rename_overlay<H: fret_ui::UiHost>(
    host: &mut H,
    overlays: &fret_runtime::Model<NodeGraphOverlayState>,
    symbol: SymbolId,
    invoked_at_window: Point,
) {
    let _ = overlays.update(host, |s, _cx| {
        s.symbol_rename = Some(SymbolRenameOverlay {
            symbol,
            invoked_at_window,
        });
    });
}

#[test]
fn symbol_rename_overlay_enter_commits_transaction_and_closes() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let symbol_id = SymbolId::new();
    let mut graph_value = Graph::new(GraphId::new());
    graph_value.symbols.insert(
        symbol_id,
        Symbol {
            name: "Old".to_string(),
            ty: None,
            default_value: None,
            meta: serde_json::Value::Null,
        },
    );
    let (graph, _view) = insert_graph_view(&mut host, graph_value);
    let edits = host.models.insert(NodeGraphEditQueue::default());
    let overlays = host.models.insert(NodeGraphOverlayState::default());
    let rename_text = host.models.insert(String::new());
    let style = NodeGraphStyle::default();

    let underlay = ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    let overlay_host = NodeGraphOverlayHost::new(
        graph.clone(),
        edits.clone(),
        overlays.clone(),
        rename_text.clone(),
        underlay,
        style,
    );
    let overlay_host_node = ui.create_node_retained(overlay_host);
    let overlay_child =
        ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    ui.set_children(overlay_host_node, vec![overlay_child]);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![underlay, overlay_host_node]);
    ui.set_root(editor);

    open_symbol_rename_overlay(
        &mut host,
        &overlays,
        symbol_id,
        Point::new(Px(400.0), Px(300.0)),
    );
    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    assert_eq!(
        ui.focus(),
        Some(overlay_child),
        "expected symbol rename overlay to focus its text input child when opened"
    );

    let _ = rename_text.update(&mut host, |t, _cx| {
        *t = "New".to_string();
    });

    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::Enter,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    let closed = overlays
        .read_ref(&host, |s| s.symbol_rename.is_none())
        .ok()
        .unwrap_or(false);
    assert!(closed, "expected Enter to close the symbol rename overlay");
    assert_eq!(
        ui.focus(),
        Some(underlay),
        "expected focus to return to the canvas node when overlay closes"
    );

    let pending = edits
        .read_ref(&host, |q| q.pending.clone())
        .ok()
        .unwrap_or_default();
    assert_eq!(pending.len(), 1, "expected exactly one queued transaction");

    let GraphTransaction { label, ops } = &pending[0];
    assert_eq!(label.as_deref(), Some("Rename Symbol"));
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        GraphOp::SetSymbolName { id, from, to } => {
            assert_eq!(*id, symbol_id);
            assert_eq!(from, "Old");
            assert_eq!(to, "New");
        }
        other => panic!("unexpected op: {other:?}"),
    }
}

#[test]
fn symbol_rename_overlay_escape_closes_without_queueing_transaction() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let symbol_id = SymbolId::new();
    let mut graph_value = Graph::new(GraphId::new());
    graph_value.symbols.insert(
        symbol_id,
        Symbol {
            name: "Old".to_string(),
            ty: None,
            default_value: None,
            meta: serde_json::Value::Null,
        },
    );
    let (graph, _view) = insert_graph_view(&mut host, graph_value);
    let edits = host.models.insert(NodeGraphEditQueue::default());
    let overlays = host.models.insert(NodeGraphOverlayState::default());
    let rename_text = host.models.insert(String::new());
    let style = NodeGraphStyle::default();

    let underlay = ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    let overlay_host = NodeGraphOverlayHost::new(
        graph.clone(),
        edits.clone(),
        overlays.clone(),
        rename_text.clone(),
        underlay,
        style,
    );
    let overlay_host_node = ui.create_node_retained(overlay_host);
    let overlay_child =
        ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    ui.set_children(overlay_host_node, vec![overlay_child]);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![underlay, overlay_host_node]);
    ui.set_root(editor);

    open_symbol_rename_overlay(
        &mut host,
        &overlays,
        symbol_id,
        Point::new(Px(400.0), Px(300.0)),
    );
    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    assert_eq!(
        ui.focus(),
        Some(overlay_child),
        "expected symbol rename overlay to focus its text input child when opened"
    );

    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::Escape,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    let closed = overlays
        .read_ref(&host, |s| s.symbol_rename.is_none())
        .ok()
        .unwrap_or(false);
    assert!(closed, "expected Escape to close the symbol rename overlay");
    assert_eq!(
        ui.focus(),
        Some(underlay),
        "expected focus to return to the canvas node when overlay closes"
    );

    let pending = edits
        .read_ref(&host, |q| q.pending.clone())
        .ok()
        .unwrap_or_default();
    assert!(
        pending.is_empty(),
        "expected Escape to close without queueing a rename transaction"
    );
}

#[test]
fn symbol_rename_overlay_enter_with_unchanged_text_closes_without_queueing_transaction() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let symbol_id = SymbolId::new();
    let mut graph_value = Graph::new(GraphId::new());
    graph_value.symbols.insert(
        symbol_id,
        Symbol {
            name: "Old".to_string(),
            ty: None,
            default_value: None,
            meta: serde_json::Value::Null,
        },
    );
    let (graph, _view) = insert_graph_view(&mut host, graph_value);
    let edits = host.models.insert(NodeGraphEditQueue::default());
    let overlays = host.models.insert(NodeGraphOverlayState::default());
    let rename_text = host.models.insert(String::new());
    let style = NodeGraphStyle::default();

    let underlay = ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    let overlay_host = NodeGraphOverlayHost::new(
        graph.clone(),
        edits.clone(),
        overlays.clone(),
        rename_text.clone(),
        underlay,
        style,
    );
    let overlay_host_node = ui.create_node_retained(overlay_host);
    let overlay_child =
        ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    ui.set_children(overlay_host_node, vec![overlay_child]);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![underlay, overlay_host_node]);
    ui.set_root(editor);

    open_symbol_rename_overlay(
        &mut host,
        &overlays,
        symbol_id,
        Point::new(Px(400.0), Px(300.0)),
    );
    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    assert_eq!(
        ui.focus(),
        Some(overlay_child),
        "expected symbol rename overlay to focus its text input child when opened"
    );

    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::Enter,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    let closed = overlays
        .read_ref(&host, |s| s.symbol_rename.is_none())
        .ok()
        .unwrap_or(false);
    assert!(closed, "expected Enter to close the symbol rename overlay");
    assert_eq!(
        ui.focus(),
        Some(underlay),
        "expected focus to return to the canvas node when overlay closes"
    );

    let pending = edits
        .read_ref(&host, |q| q.pending.clone())
        .ok()
        .unwrap_or_default();
    assert!(
        pending.is_empty(),
        "expected Enter with unchanged text to close without queueing a rename transaction"
    );
}
