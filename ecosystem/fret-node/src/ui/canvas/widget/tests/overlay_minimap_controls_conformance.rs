use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fret_core::{
    AppWindowId, Event, KeyCode, Modifiers, MouseButton, MouseButtons, Point, PointerEvent, Px,
    Rect, Size,
};
use fret_runtime::{CommandId, Effect};
use fret_ui::UiTree;
use fret_ui::retained_bridge::UiTreeRetainedExt as _;

use crate::core::{Graph, GraphId};
use crate::io::NodeGraphViewState;
use crate::runtime::store::NodeGraphStore;
use crate::ui::commands::CMD_NODE_GRAPH_ZOOM_IN;
use crate::ui::{
    NodeGraphControlsOverlay, NodeGraphEditor, NodeGraphInternalsSnapshot, NodeGraphInternalsStore,
    NodeGraphMiniMapOverlay, NodeGraphStyle,
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

fn test_style() -> NodeGraphStyle {
    let mut style = NodeGraphStyle::default();
    style.minimap_width = 200.0;
    style.minimap_height = 120.0;
    style.minimap_margin = 10.0;
    style.minimap_world_padding = 0.0;

    style.controls_button_size = 20.0;
    style.controls_padding = 4.0;
    style.controls_gap = 2.0;
    style.controls_margin = 10.0;
    style
}

fn minimap_rect(bounds: Rect, style: &NodeGraphStyle) -> Rect {
    let w = style.minimap_width.max(40.0);
    let h = style.minimap_height.max(30.0);
    let margin = style.minimap_margin.max(0.0);
    let x = bounds.origin.x.0 + (bounds.size.width.0 - margin - w).max(0.0);
    let y = bounds.origin.y.0 + (bounds.size.height.0 - margin - h).max(0.0);
    Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
}

fn controls_panel_rect(bounds: Rect, style: &NodeGraphStyle) -> Rect {
    let margin = style.controls_margin.max(0.0);
    let pad = style.controls_padding.max(0.0);
    let gap = style.controls_gap.max(0.0);
    let button = style.controls_button_size.max(10.0);

    let panel_w = button + 2.0 * pad;
    let items = 6.0_f32;
    let panel_h = items * button + (items - 1.0) * gap + 2.0 * pad;

    let x = bounds.origin.x.0 + (bounds.size.width.0 - margin - panel_w).max(0.0);
    let y = bounds.origin.y.0 + margin;
    Rect::new(
        Point::new(Px(x), Px(y)),
        Size::new(Px(panel_w), Px(panel_h)),
    )
}

#[test]
fn controls_overlay_pointer_events_fall_through_outside_panel() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let underlay_downs = Arc::new(AtomicUsize::new(0));
    let underlay = ui.create_node_retained(PointerDownCounter::new(underlay_downs.clone()));

    let view = host.models.insert(NodeGraphViewState::default());
    let controls = NodeGraphControlsOverlay::new(underlay, view, test_style());
    let controls_node = ui.create_node_retained(controls);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![underlay, controls_node]);
    ui.set_root(editor);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(10.0), Px(10.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(underlay_downs.load(Ordering::Relaxed), 1);
}

#[test]
fn controls_overlay_blocks_canvas_input_within_panel_even_off_button() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let style = test_style();
    let panel = controls_panel_rect(bounds(), &style);

    let underlay_downs = Arc::new(AtomicUsize::new(0));
    let underlay = ui.create_node_retained(PointerDownCounter::new(underlay_downs.clone()));

    let view = host.models.insert(NodeGraphViewState::default());
    let controls = NodeGraphControlsOverlay::new(underlay, view, style);
    let controls_node = ui.create_node_retained(controls);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![underlay, controls_node]);
    ui.set_root(editor);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    // Inside panel, but outside the first button (pad area).
    let inside_panel_non_button =
        Point::new(Px(panel.origin.x.0 + 1.0), Px(panel.origin.y.0 + 1.0));
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: inside_panel_non_button,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        underlay_downs.load(Ordering::Relaxed),
        0,
        "expected controls overlay to block pointer-down within its panel bounds"
    );
}

#[test]
fn controls_overlay_button_click_requests_focus_to_canvas_node() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let style = test_style();
    let panel = controls_panel_rect(bounds(), &style);
    let pad = style.controls_padding.max(0.0);

    let underlay_downs = Arc::new(AtomicUsize::new(0));
    let underlay = ui.create_node_retained(PointerDownCounter::new(underlay_downs.clone()));

    let view = host.models.insert(NodeGraphViewState::default());
    let controls = NodeGraphControlsOverlay::new(underlay, view, style);
    let controls_node = ui.create_node_retained(controls);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![underlay, controls_node]);
    ui.set_root(editor);

    ui.layout_all(&mut host, &mut services, bounds(), 1.0);
    ui.set_focus(Some(controls_node));
    assert_eq!(ui.focus(), Some(controls_node));

    // First button (ToggleConnectionMode): top-left inside the panel.
    let inside_button = Point::new(
        Px(panel.origin.x.0 + pad + 1.0),
        Px(panel.origin.y.0 + pad + 1.0),
    );

    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: inside_button,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: inside_button,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(underlay_downs.load(Ordering::Relaxed), 0);
    assert_eq!(
        ui.focus(),
        Some(underlay),
        "expected controls button activation to request focus to the canvas node"
    );
}

#[test]
fn controls_overlay_keyboard_navigation_and_activation_dispatches_command_and_returns_focus_to_canvas()
 {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let underlay = ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    let view = host.models.insert(NodeGraphViewState::default());
    let controls = NodeGraphControlsOverlay::new(underlay, view, test_style());
    let controls_node = ui.create_node_retained(controls);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![underlay, controls_node]);
    ui.set_root(editor);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    ui.set_focus(Some(controls_node));
    assert_eq!(ui.focus(), Some(controls_node));

    // Default active button is the first one. ArrowDown selects ZoomIn, then Enter activates it.
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
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

    assert!(
        host.effects.iter().any(|e| matches!(
            e,
            Effect::Command { command, .. } if *command == CommandId::from(CMD_NODE_GRAPH_ZOOM_IN)
        )),
        "expected keyboard activation to dispatch zoom-in command"
    );
    assert_eq!(
        ui.focus(),
        Some(underlay),
        "expected activation to request focus to the canvas node"
    );
}

#[test]
fn controls_overlay_escape_returns_focus_to_canvas_without_dispatching_command() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let underlay = ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));
    let view = host.models.insert(NodeGraphViewState::default());
    let controls = NodeGraphControlsOverlay::new(underlay, view, test_style());
    let controls_node = ui.create_node_retained(controls);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![underlay, controls_node]);
    ui.set_root(editor);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    ui.set_focus(Some(controls_node));
    assert_eq!(ui.focus(), Some(controls_node));

    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::Escape,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert!(
        !host
            .effects
            .iter()
            .any(|e| matches!(e, Effect::Command { .. })),
        "expected Escape to only change focus, not dispatch commands"
    );
    assert_eq!(ui.focus(), Some(underlay));
}

#[test]
fn minimap_pointer_events_fall_through_outside_rect() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let underlay_downs = Arc::new(AtomicUsize::new(0));
    let underlay = ui.create_node_retained(PointerDownCounter::new(underlay_downs.clone()));

    let style = test_style();
    let graph = host.models.insert(Graph::new(GraphId::new()));
    let view = host.models.insert(NodeGraphViewState::default());
    let internals = Arc::new(NodeGraphInternalsStore::new());
    let mut snap = NodeGraphInternalsSnapshot::default();
    snap.transform.bounds_size = bounds().size;
    internals.update(snap);

    let minimap = NodeGraphMiniMapOverlay::new(underlay, graph, view, internals, style);
    let minimap_node = ui.create_node_retained(minimap);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![underlay, minimap_node]);
    ui.set_root(editor);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(10.0), Px(10.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(underlay_downs.load(Ordering::Relaxed), 1);
}

#[test]
fn minimap_drag_updates_view_state_and_store_when_attached() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let style = test_style();
    let minimap = minimap_rect(bounds(), &style);

    let underlay_downs = Arc::new(AtomicUsize::new(0));
    let underlay = ui.create_node_retained(PointerDownCounter::new(underlay_downs.clone()));

    let graph_value = Graph::new(GraphId::new());
    let graph = host.models.insert(graph_value.clone());
    let view = host.models.insert(NodeGraphViewState::default());

    let store = host.models.insert(NodeGraphStore::new(
        graph_value,
        NodeGraphViewState::default(),
    ));

    let internals = Arc::new(NodeGraphInternalsStore::new());
    let mut snap = NodeGraphInternalsSnapshot::default();
    snap.transform.bounds_size = bounds().size;
    internals.update(snap);

    let minimap_widget =
        NodeGraphMiniMapOverlay::new(underlay, graph, view.clone(), internals, style)
            .with_store(store.clone());
    let minimap_node = ui.create_node_retained(minimap_widget);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![underlay, minimap_node]);
    ui.set_root(editor);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    ui.set_focus(Some(minimap_node));
    assert_eq!(ui.focus(), Some(minimap_node));

    let start = Point::new(
        Px(minimap.origin.x.0 + 0.5 * minimap.size.width.0),
        Px(minimap.origin.y.0 + 0.5 * minimap.size.height.0),
    );
    let moved = Point::new(Px(start.x.0 + 10.0), Px(start.y.0));

    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: start,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        ui.focus(),
        Some(underlay),
        "expected minimap interactions to keep focus on the canvas node"
    );

    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: moved,
            buttons: MouseButtons {
                left: true,
                right: false,
                middle: false,
            },
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: moved,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    // With empty graphs, world == viewport. scale = min(w/800, h/600) = min(0.25, 0.2) = 0.2.
    // Moving +10px in minimap maps to +50 canvas units, so pan shifts by -50.
    let expected_pan_x = -50.0;
    let pan = view.read_ref(&host, |s| s.pan).ok().expect("view state").x;
    assert!(
        (pan - expected_pan_x).abs() <= 1.0e-4,
        "{pan} != {expected_pan_x}"
    );

    let store_pan_x = store
        .read_ref(&host, |s| s.view_state().pan.x)
        .ok()
        .expect("store view state pan");
    assert!(
        (store_pan_x - expected_pan_x).abs() <= 1.0e-4,
        "{store_pan_x} != {expected_pan_x}"
    );

    // Underlay does not see the pointer-down that started the minimap drag.
    assert_eq!(underlay_downs.load(Ordering::Relaxed), 0);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    assert!(
        snap.nodes
            .iter()
            .any(|n| n.test_id.as_deref() == Some("node_graph.minimap")),
        "expected minimap overlay to contribute a stable semantics test_id"
    );
}

#[test]
fn minimap_keyboard_pan_updates_view_state_and_store_when_attached() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let style = test_style();
    let underlay = ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));

    let graph_value = Graph::new(GraphId::new());
    let graph = host.models.insert(graph_value.clone());
    let view = host.models.insert(NodeGraphViewState::default());
    let store = host.models.insert(NodeGraphStore::new(
        graph_value,
        NodeGraphViewState::default(),
    ));

    let internals = Arc::new(NodeGraphInternalsStore::new());
    let mut snap = NodeGraphInternalsSnapshot::default();
    snap.transform.bounds_size = bounds().size;
    internals.update(snap);

    let minimap_widget =
        NodeGraphMiniMapOverlay::new(underlay, graph, view.clone(), internals, style)
            .with_store(store.clone());
    let minimap_node = ui.create_node_retained(minimap_widget);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![underlay, minimap_node]);
    ui.set_root(editor);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    ui.set_focus(Some(minimap_node));
    assert_eq!(ui.focus(), Some(minimap_node));

    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    // Pan step is expressed in screen px, divided by zoom=1.
    let expected_pan_x = -24.0;
    let pan_x = view
        .read_ref(&host, |s| s.pan.x)
        .ok()
        .expect("view state pan");
    assert!(
        (pan_x - expected_pan_x).abs() <= 1.0e-4,
        "{pan_x} != {expected_pan_x}"
    );

    let store_pan_x = store
        .read_ref(&host, |s| s.view_state().pan.x)
        .ok()
        .expect("store view state pan");
    assert!(
        (store_pan_x - expected_pan_x).abs() <= 1.0e-4,
        "{store_pan_x} != {expected_pan_x}"
    );
}

#[test]
fn minimap_keyboard_zoom_updates_view_state_and_store_zoom_about_center() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let style = test_style();
    let underlay = ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));

    let graph_value = Graph::new(GraphId::new());
    let graph = host.models.insert(graph_value.clone());
    let view = host.models.insert(NodeGraphViewState::default());
    let store = host.models.insert(NodeGraphStore::new(
        graph_value,
        NodeGraphViewState::default(),
    ));

    let internals = Arc::new(NodeGraphInternalsStore::new());
    let mut snap = NodeGraphInternalsSnapshot::default();
    snap.transform.bounds_size = bounds().size;
    internals.update(snap);

    let minimap_widget =
        NodeGraphMiniMapOverlay::new(underlay, graph, view.clone(), internals, style)
            .with_store(store.clone());
    let minimap_node = ui.create_node_retained(minimap_widget);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![underlay, minimap_node]);
    ui.set_root(editor);
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    ui.set_focus(Some(minimap_node));
    assert_eq!(ui.focus(), Some(minimap_node));

    ui.dispatch_event(
        &mut host,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::NumpadAdd,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    let expected_zoom = 1.1;
    let zoom = view
        .read_ref(&host, |s| s.zoom)
        .ok()
        .expect("view state zoom");
    assert!(
        (zoom - expected_zoom).abs() <= 1.0e-6,
        "{zoom} != {expected_zoom}"
    );

    let pan = view
        .read_ref(&host, |s| s.pan)
        .ok()
        .expect("view state pan");

    // Zoom about center of an 800x600 viewport.
    let expected_pan_x = 800.0 / (2.0 * expected_zoom) - 400.0;
    let expected_pan_y = 600.0 / (2.0 * expected_zoom) - 300.0;
    assert!((pan.x - expected_pan_x).abs() <= 1.0e-3, "{pan:?}");
    assert!((pan.y - expected_pan_y).abs() <= 1.0e-3, "{pan:?}");

    let store_zoom = store
        .read_ref(&host, |s| s.view_state().zoom)
        .ok()
        .expect("store view state zoom");
    assert!((store_zoom - expected_zoom).abs() <= 1.0e-6);
}

#[test]
fn controls_overlay_contributes_semantics_test_id() {
    let mut host = TestUiHostImpl::default();
    let mut services = NullServices::default();
    let mut ui = UiTree::<TestUiHostImpl>::default();
    ui.set_window(AppWindowId::default());

    let underlay = ui.create_node_retained(PointerDownCounter::new(Arc::new(AtomicUsize::new(0))));

    let view = host.models.insert(NodeGraphViewState::default());
    let controls = NodeGraphControlsOverlay::new(underlay, view, test_style());
    let controls_node = ui.create_node_retained(controls);

    let editor = ui.create_node_retained(NodeGraphEditor::new());
    ui.set_children(editor, vec![underlay, controls_node]);
    ui.set_root(editor);

    ui.layout_all(&mut host, &mut services, bounds(), 1.0);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut host, &mut services, bounds(), 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    assert!(
        snap.nodes
            .iter()
            .any(|n| n.test_id.as_deref() == Some("node_graph.controls")),
        "expected controls overlay to contribute a stable semantics test_id"
    );
}
