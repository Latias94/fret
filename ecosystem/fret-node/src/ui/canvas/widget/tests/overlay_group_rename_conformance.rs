use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fret_core::{
    AppWindowId, Event, KeyCode, Modifiers, MouseButton, Point, PointerEvent, PointerType, Px,
    Rect, Size,
};
use fret_runtime::ModelsHost as _;
use fret_ui::retained_bridge::UiTreeRetainedExt as _;
use fret_ui::{Invalidation, UiTree};

use crate::core::{CanvasPoint, CanvasRect, CanvasSize, Graph, GraphId, Group, GroupId};
use crate::ops::{GraphOp, GraphTransaction};
use crate::ui::{
    GroupRenameOverlay, NodeGraphEditQueue, NodeGraphEditor, NodeGraphOverlayHost,
    NodeGraphOverlayState, NodeGraphStyle,
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

fn open_rename_overlay<H: fret_ui::UiHost>(
    host: &mut H,
    overlays: &fret_runtime::Model<NodeGraphOverlayState>,
    group: GroupId,
    invoked_at_window: Point,
) {
    let _ = overlays.update(host, |s, _cx| {
        s.group_rename = Some(GroupRenameOverlay {
            group,
            invoked_at_window,
        });
    });
}

fn overlay_rect_for(style: &NodeGraphStyle, desired_origin: Point, bounds: Rect) -> Rect {
    let w = style.context_menu_width.max(40.0);
    let h = (style.context_menu_item_height.max(20.0) + 2.0 * style.context_menu_padding).max(24.0);

    let min_x = bounds.origin.x.0;
    let min_y = bounds.origin.y.0;
    let max_x = bounds.origin.x.0 + (bounds.size.width.0 - w).max(0.0);
    let max_y = bounds.origin.y.0 + (bounds.size.height.0 - h).max(0.0);

    let x = desired_origin.x.0.clamp(min_x, max_x);
    let y = desired_origin.y.0.clamp(min_y, max_y);
    Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
}

#[test]
fn group_rename_overlay_is_hit_test_transparent_when_inactive_and_blocks_within_bounds_when_active()
{
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let group_id = GroupId::new();
    let mut graph_value = Graph::new(GraphId::new());
    graph_value.groups.insert(
        group_id,
        Group {
            title: "Group A".to_string(),
            rect: CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: CanvasSize {
                    width: 100.0,
                    height: 60.0,
                },
            },
            color: None,
        },
    );
    let (graph, _view) = insert_graph_view(&mut host, graph_value);
    let edits = host.models.insert(NodeGraphEditQueue::default());
    let overlays = host.models.insert(NodeGraphOverlayState::default());
    let group_rename_text = host.models.insert(String::new());

    let style = NodeGraphStyle::default();
    let invoked_at = Point::new(Px(400.0), Px(300.0));
    let expected_rect = overlay_rect_for(&style, invoked_at, bounds());

    let underlay_downs = Arc::new(AtomicUsize::new(0));
    let underlay = ui.create_node_retained(PointerDownCounter::new(underlay_downs.clone()));

    let overlay_host = NodeGraphOverlayHost::new(
        graph,
        edits,
        overlays.clone(),
        group_rename_text,
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

    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    // Inactive overlay: clicks should reach the canvas.
    let inside = Point::new(
        Px(expected_rect.origin.x.0 + 5.0),
        Px(expected_rect.origin.y.0 + 5.0),
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    assert_eq!(underlay_downs.load(Ordering::Relaxed), 1);

    // Activate rename overlay and re-layout.
    open_rename_overlay(&mut host, &overlays, group_id, invoked_at);
    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);
    ui.invalidate(overlay_host_node, Invalidation::Layout);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    // Active overlay: clicks within bounds must NOT reach the canvas.
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: PointerType::Mouse,
        }),
    );
    assert_eq!(
        underlay_downs.load(Ordering::Relaxed),
        1,
        "expected overlay to intercept pointer input within its bounds"
    );

    // Outside bounds: events fall through.
    let outside = Point::new(
        Px(expected_rect.origin.x.0 + expected_rect.size.width.0 + 10.0),
        Px(expected_rect.origin.y.0 + expected_rect.size.height.0 + 10.0),
    );
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
    assert_eq!(underlay_downs.load(Ordering::Relaxed), 2);
}

#[test]
fn group_rename_overlay_escape_closes_and_restores_focus_to_canvas() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let group_id = GroupId::new();
    let mut graph_value = Graph::new(GraphId::new());
    graph_value.groups.insert(
        group_id,
        Group {
            title: "Group A".to_string(),
            rect: CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: CanvasSize {
                    width: 100.0,
                    height: 60.0,
                },
            },
            color: None,
        },
    );
    let (graph, _view) = insert_graph_view(&mut host, graph_value);
    let edits = host.models.insert(NodeGraphEditQueue::default());
    let overlays = host.models.insert(NodeGraphOverlayState::default());
    let group_rename_text = host.models.insert(String::new());
    let style = NodeGraphStyle::default();

    let underlay = ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    let overlay_host = NodeGraphOverlayHost::new(
        graph,
        edits,
        overlays.clone(),
        group_rename_text,
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

    open_rename_overlay(
        &mut host,
        &overlays,
        group_id,
        Point::new(Px(400.0), Px(300.0)),
    );
    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    assert_eq!(
        ui.focus(),
        Some(overlay_child),
        "expected rename overlay to focus its text input child when opened"
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
        .read_ref(&host, |s| s.group_rename.is_none())
        .ok()
        .unwrap_or(false);
    assert!(closed, "expected Escape to close the group rename overlay");
    assert_eq!(
        ui.focus(),
        Some(underlay),
        "expected focus to return to the canvas node when overlay closes"
    );
}

#[test]
fn group_rename_overlay_enter_commits_transaction_and_closes() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let group_id = GroupId::new();
    let mut graph_value = Graph::new(GraphId::new());
    graph_value.groups.insert(
        group_id,
        Group {
            title: "Old".to_string(),
            rect: CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: CanvasSize {
                    width: 100.0,
                    height: 60.0,
                },
            },
            color: None,
        },
    );
    let (graph, _view) = insert_graph_view(&mut host, graph_value);
    let edits = host.models.insert(NodeGraphEditQueue::default());
    let overlays = host.models.insert(NodeGraphOverlayState::default());
    let group_rename_text = host.models.insert(String::new());
    let style = NodeGraphStyle::default();

    let underlay = ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    let overlay_host = NodeGraphOverlayHost::new(
        graph.clone(),
        edits.clone(),
        overlays.clone(),
        group_rename_text.clone(),
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

    open_rename_overlay(
        &mut host,
        &overlays,
        group_id,
        Point::new(Px(400.0), Px(300.0)),
    );
    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    // Change text before committing.
    let _ = group_rename_text.update(&mut host, |t, _cx| {
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
        .read_ref(&host, |s| s.group_rename.is_none())
        .ok()
        .unwrap_or(false);
    assert!(closed, "expected Enter to close the group rename overlay");

    let pending = edits
        .read_ref(&host, |q| q.pending.clone())
        .ok()
        .unwrap_or_default();
    assert_eq!(pending.len(), 1, "expected exactly one queued transaction");

    let GraphTransaction { label, ops } = &pending[0];
    assert_eq!(label.as_deref(), Some("Rename Group"));
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        GraphOp::SetGroupTitle { id, from, to } => {
            assert_eq!(*id, group_id);
            assert_eq!(from, "Old");
            assert_eq!(to, "New");
        }
        other => panic!("unexpected op: {other:?}"),
    }
}
