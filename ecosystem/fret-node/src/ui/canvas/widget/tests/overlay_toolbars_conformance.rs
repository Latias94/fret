use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fret_core::{AppWindowId, Event, Modifiers, MouseButton, Point, PointerEvent, Px, Rect, Size};
use fret_runtime::ModelsHost as _;
use fret_ui::retained_bridge::UiTreeRetainedExt as _;
use fret_ui::{Invalidation, UiTree};

use crate::core::{EdgeId, Graph, GraphId, Node, NodeId, NodeKindKey};
use crate::ui::internals::{NodeGraphInternalsSnapshot, NodeGraphInternalsStore};
use crate::ui::{
    NodeGraphEdgeToolbar, NodeGraphEditor, NodeGraphNodeToolbar, NodeGraphToolbarAlign,
    NodeGraphToolbarPosition, NodeGraphToolbarSize,
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

#[test]
fn node_toolbar_pointer_events_fall_through_outside_bounds() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let node_id = NodeId::new();
    let mut graph_value = Graph::new(GraphId::new());
    graph_value.nodes.insert(
        node_id,
        Node {
            kind: NodeKindKey::new("test.node"),
            kind_version: 1,
            pos: crate::core::CanvasPoint { x: 0.0, y: 0.0 },
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
            ports: Vec::new(),
            data: serde_json::Value::Null,
        },
    );
    let (graph, view) = insert_graph_view(&mut host, graph_value);
    let _ = view.update(&mut host, |s, _cx| {
        s.selected_nodes = vec![node_id];
    });

    let internals = Arc::new(NodeGraphInternalsStore::new());
    let mut snap = NodeGraphInternalsSnapshot::default();
    snap.nodes_window.insert(
        node_id,
        Rect::new(
            Point::new(Px(200.0), Px(200.0)),
            Size::new(Px(100.0), Px(100.0)),
        ),
    );
    internals.update(snap);

    let underlay_downs = Arc::new(AtomicUsize::new(0));
    let toolbar_downs = Arc::new(AtomicUsize::new(0));

    let underlay = ui.create_node_retained(PointerDownCounter::new(underlay_downs.clone()));

    let toolbar = NodeGraphNodeToolbar::new(underlay, graph, view, internals)
        .for_node(node_id)
        .with_position(NodeGraphToolbarPosition::Top)
        .with_align(NodeGraphToolbarAlign::Center)
        .with_gap_px(0.0)
        .with_size(NodeGraphToolbarSize::Fixed(Size::new(Px(80.0), Px(40.0))));
    let toolbar_node = ui.create_node_retained(toolbar);

    let toolbar_button = ui.create_node_retained(PointerDownCounter::new(toolbar_downs.clone()));
    ui.set_children(toolbar_node, vec![toolbar_button]);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![underlay, toolbar_node]);
    ui.set_root(editor);

    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    let outside = Point::new(Px(10.0), Px(10.0));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: outside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(underlay_downs.load(Ordering::Relaxed), 1);
    assert_eq!(toolbar_downs.load(Ordering::Relaxed), 0);

    // Node rect: (200,200)-(300,300), toolbar fixed size: 80x40, Top+Center, gap=0 =>
    // origin = (210,160). Pick a point inside.
    let inside = Point::new(Px(220.0), Px(170.0));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        underlay_downs.load(Ordering::Relaxed),
        1,
        "expected toolbar to intercept pointer input within its bounds"
    );
    assert_eq!(toolbar_downs.load(Ordering::Relaxed), 1);
}

#[test]
fn edge_toolbar_pointer_events_fall_through_outside_bounds() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let edge_id = EdgeId::new();
    let (graph, view) = insert_graph_view(&mut host, Graph::new(GraphId::new()));
    let _ = view.update(&mut host, |s, _cx| {
        s.selected_edges = vec![edge_id];
    });

    let internals = Arc::new(NodeGraphInternalsStore::new());
    let mut snap = NodeGraphInternalsSnapshot::default();
    snap.edge_centers_window
        .insert(edge_id, Point::new(Px(300.0), Px(300.0)));
    internals.update(snap);

    let underlay_downs = Arc::new(AtomicUsize::new(0));
    let toolbar_downs = Arc::new(AtomicUsize::new(0));

    let underlay = ui.create_node_retained(PointerDownCounter::new(underlay_downs.clone()));

    let toolbar = NodeGraphEdgeToolbar::new(underlay, graph, view, internals)
        .for_edge(edge_id)
        .with_align_x(NodeGraphToolbarAlign::Center)
        .with_align_y(NodeGraphToolbarAlign::Center)
        .with_size(NodeGraphToolbarSize::Fixed(Size::new(Px(80.0), Px(40.0))));
    let toolbar_node = ui.create_node_retained(toolbar);

    let toolbar_button = ui.create_node_retained(PointerDownCounter::new(toolbar_downs.clone()));
    ui.set_children(toolbar_node, vec![toolbar_button]);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![underlay, toolbar_node]);
    ui.set_root(editor);

    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    let outside = Point::new(Px(10.0), Px(10.0));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: outside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(underlay_downs.load(Ordering::Relaxed), 1);
    assert_eq!(toolbar_downs.load(Ordering::Relaxed), 0);

    // Edge center: (300,300), fixed size: 80x40, Center+Center =>
    // origin = (260,280). Pick a point inside.
    let inside = Point::new(Px(270.0), Px(290.0));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(underlay_downs.load(Ordering::Relaxed), 1);
    assert_eq!(toolbar_downs.load(Ordering::Relaxed), 1);
}

#[test]
fn toolbars_release_focus_to_canvas_when_hidden() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let node_id = NodeId::new();
    let edge_id = EdgeId::new();

    let (graph, view) = insert_graph_view(&mut host, Graph::new(GraphId::new()));
    let _ = view.update(&mut host, |s, _cx| {
        s.selected_nodes = vec![node_id];
        s.selected_edges = vec![edge_id];
    });

    let internals = Arc::new(NodeGraphInternalsStore::new());
    let mut snap = NodeGraphInternalsSnapshot::default();
    snap.nodes_window.insert(
        node_id,
        Rect::new(
            Point::new(Px(200.0), Px(200.0)),
            Size::new(Px(100.0), Px(100.0)),
        ),
    );
    snap.edge_centers_window
        .insert(edge_id, Point::new(Px(300.0), Px(300.0)));
    internals.update(snap);

    let underlay = ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));

    let node_toolbar =
        NodeGraphNodeToolbar::new(underlay, graph.clone(), view.clone(), internals.clone())
            .for_node(node_id)
            .with_gap_px(0.0)
            .with_size(NodeGraphToolbarSize::Fixed(Size::new(Px(80.0), Px(40.0))));
    let node_toolbar_node = ui.create_node_retained(node_toolbar);
    let node_toolbar_child =
        ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    ui.set_children(node_toolbar_node, vec![node_toolbar_child]);

    let edge_toolbar = NodeGraphEdgeToolbar::new(underlay, graph, view.clone(), internals)
        .for_edge(edge_id)
        .with_size(NodeGraphToolbarSize::Fixed(Size::new(Px(80.0), Px(40.0))));
    let edge_toolbar_node = ui.create_node_retained(edge_toolbar);
    let edge_toolbar_child =
        ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    ui.set_children(edge_toolbar_node, vec![edge_toolbar_child]);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![underlay, node_toolbar_node, edge_toolbar_node]);
    ui.set_root(editor);

    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    ui.set_focus(Some(node_toolbar_child));
    assert_eq!(ui.focus(), Some(node_toolbar_child));

    // Hide all toolbars (selection drives visibility by default) and force a layout pass.
    let _ = view.update(&mut host, |s, _cx| {
        s.selected_nodes.clear();
        s.selected_edges.clear();
    });
    let changed = host.take_changed_models();
    ui.propagate_model_changes(&mut host, &changed);
    ui.invalidate(node_toolbar_node, Invalidation::Layout);
    ui.invalidate(edge_toolbar_node, Invalidation::Layout);

    ui.layout_all(&mut host, &mut services, bounds(), 1.0);
    assert_eq!(
        ui.focus(),
        Some(underlay),
        "expected hidden toolbars to relinquish focus to the canvas node"
    );
}
