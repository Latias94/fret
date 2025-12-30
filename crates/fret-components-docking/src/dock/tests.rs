use super::hit_test::hit_test_drop_target;
use super::layout::{compute_layout_map, dock_hint_rects, dock_space_regions};
use super::prelude_core::*;
use super::prelude_runtime::*;
use super::prelude_ui::*;
use super::split_stabilize::{apply_same_axis_locks, compute_same_axis_locks_for_split_drag};
use super::{DockManager, DockSpace};
use crate::test_host::TestHost;
use fret_core::{
    AppWindowId, Event, InternalDragEvent, InternalDragKind, Modifiers, PlatformCapabilities,
    Point, Px, Scene, SceneOp, Size, TextBlobId, TextConstraints, TextMetrics, TextService,
    TextStyle,
};
use fret_ui::UiTree;
use fret_ui::retained_bridge::UiTreeRetainedExt as _;
use fret_ui::retained_bridge::resizable_panel_group as resizable;

#[derive(Default)]
struct FakeTextService;

impl TextService for FakeTextService {
    fn prepare(
        &mut self,
        _text: &str,
        _style: TextStyle,
        _constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        (
            TextBlobId::default(),
            TextMetrics {
                size: Size::new(Px(240.0), Px(34.0)),
                baseline: Px(18.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl fret_core::PathService for FakeTextService {
    fn prepare(
        &mut self,
        _commands: &[fret_core::PathCommand],
        _style: fret_core::PathStyle,
        _constraints: fret_core::PathConstraints,
    ) -> (fret_core::PathId, fret_core::PathMetrics) {
        (
            fret_core::PathId::default(),
            fret_core::PathMetrics::default(),
        )
    }

    fn release(&mut self, _path: fret_core::PathId) {}
}

impl fret_core::SvgService for FakeTextService {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
    }
}

#[derive(Default)]
struct TestStack;

impl<H: UiHost> Widget<H> for TestStack {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        for &child in cx.children {
            let _ = cx.layout_in(child, cx.bounds);
        }
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for &child in cx.children {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }
    }
}

struct DockViewportHarness {
    window: AppWindowId,
    target: fret_core::RenderTargetId,
    root: fret_core::NodeId,
    ui: UiTree<TestHost>,
    app: TestHost,
    text: FakeTextService,
}

impl DockViewportHarness {
    fn new() -> Self {
        let window = AppWindowId::default();
        let target = fret_core::RenderTargetId::default();

        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node_retained(DockSpace::new(window));
        ui.set_root(root);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.with_global_mut(DockManager::default, |dock, _app| {
            let panel_key = PanelKey::new("core.viewport");
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel_key.clone()],
                active: 0,
            });
            dock.graph.set_window_root(window, tabs);
            dock.panels.insert(
                panel_key,
                DockPanel {
                    title: "Viewport".to_string(),
                    color: Color::TRANSPARENT,
                    viewport: Some(super::ViewportPanel {
                        target,
                        target_px_size: (320, 240),
                        fit: fret_core::ViewportFit::Stretch,
                        context_menu_enabled: true,
                    }),
                },
            );
        });

        Self {
            window,
            target,
            root,
            ui,
            app,
            text: FakeTextService,
        }
    }

    fn layout(&mut self) {
        let size = Size::new(Px(800.0), Px(600.0));
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
        let _ = self
            .ui
            .layout(&mut self.app, &mut self.text, self.root, size, 1.0);
        let mut scene = Scene::default();
        self.ui.paint(
            &mut self.app,
            &mut self.text,
            self.root,
            bounds,
            &mut scene,
            1.0,
        );
    }

    fn viewport_point(&self) -> Point {
        let rect = self
            .app
            .global::<DockManager>()
            .and_then(|dock| dock.viewport_content_rect(self.window, self.target))
            .expect("expected viewport content rect to be recorded during paint");
        Point::new(Px(rect.origin.x.0 + 10.0), Px(rect.origin.y.0 + 10.0))
    }
}

#[test]
fn drag_update_fractions_updates_two_panel_split() {
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(300.0), Px(40.0)));
    let fractions = vec![0.5, 0.5];
    let next = resizable::drag_update_fractions(
        fret_core::Axis::Horizontal,
        bounds,
        2,
        &fractions,
        0,
        Px(0.0),
        Px(6.0),
        &[],
        0.0,
        Point::new(Px(200.0), Px(20.0)),
    )
    .expect("expected drag to update fractions");
    assert!(next[0] > 0.5, "expected left to grow, got {next:?}");
}

#[test]
fn same_axis_nested_split_drag_preserves_inner_sibling_width() {
    let mut graph = DockGraph::new();

    let a = graph.insert_node(DockNode::Tabs {
        tabs: vec![PanelKey::new("test.a")],
        active: 0,
    });
    let b = graph.insert_node(DockNode::Tabs {
        tabs: vec![PanelKey::new("test.b")],
        active: 0,
    });
    let c = graph.insert_node(DockNode::Tabs {
        tabs: vec![PanelKey::new("test.c")],
        active: 0,
    });

    let inner = graph.insert_node(DockNode::Split {
        axis: fret_core::Axis::Horizontal,
        children: vec![a, b],
        fractions: vec![0.5, 0.5],
    });
    let root = graph.insert_node(DockNode::Split {
        axis: fret_core::Axis::Horizontal,
        children: vec![inner, c],
        fractions: vec![0.5, 0.5],
    });

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(600.0), Px(80.0)));
    let layout0 = compute_layout_map(&graph, root, bounds);

    let a0 = layout0.get(&a).expect("missing a").size.width.0;
    let b0 = layout0.get(&b).expect("missing b").size.width.0;

    let locks = compute_same_axis_locks_for_split_drag(
        &graph,
        &layout0,
        root,
        fret_core::Axis::Horizontal,
        0,
    );
    assert!(
        !locks.is_empty(),
        "expected nested locks for same-axis split"
    );

    let fractions = match graph.node(root).expect("root") {
        DockNode::Split { fractions, .. } => fractions.clone(),
        _ => unreachable!(),
    };

    // Drag the root splitter rightward (increase left subtree width).
    let next = resizable::drag_update_fractions(
        fret_core::Axis::Horizontal,
        bounds,
        2,
        &fractions,
        0,
        Px(0.0),
        Px(6.0),
        &[],
        0.0,
        Point::new(Px(420.0), Px(40.0)),
    )
    .expect("expected root drag to update fractions");
    assert!(graph.update_split_fractions(root, next));

    apply_same_axis_locks(
        &mut graph,
        root,
        bounds,
        fret_core::Axis::Horizontal,
        &locks,
    );

    let layout1 = compute_layout_map(&graph, root, bounds);
    let a1 = layout1.get(&a).expect("missing a").size.width.0;
    let b1 = layout1.get(&b).expect("missing b").size.width.0;

    assert!(
        (a1 - a0).abs() <= 1.0,
        "expected inner sibling width preserved (a), before={a0}, after={a1}"
    );
    assert!(
        b1 > b0 + 10.0,
        "expected touching node to grow (b), before={b0}, after={b1}"
    );

    let inner_f0 = match graph.node(inner).expect("inner") {
        DockNode::Split { fractions, .. } => fractions[0],
        _ => unreachable!(),
    };
    assert!(
        inner_f0 < 0.5,
        "expected inner split fraction to change to keep (a) stable, got {inner_f0}"
    );
}

#[test]
fn drag_update_fractions_handles_nan_bounds() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(f32::NAN), Px(300.0)),
    );
    let fractions = vec![0.5, 0.5];
    let next = resizable::drag_update_fractions(
        fret_core::Axis::Horizontal,
        bounds,
        2,
        &fractions,
        0,
        Px(0.0),
        Px(6.0),
        &[],
        0.0,
        Point::new(Px(60.0), Px(10.0)),
    );
    assert!(next.is_none());
}

#[test]
fn dock_space_paints_empty_state_when_no_window_root() {
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node_retained(DockSpace::new(AppWindowId::default()));
    ui.set_root(root);

    let mut app = TestHost::new();
    let mut text = FakeTextService;

    let size = Size::new(Px(800.0), Px(600.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);

    let _ = ui.layout(&mut app, &mut text, root, size, 1.0);
    let mut scene = Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);

    assert!(
        scene
            .ops()
            .iter()
            .any(|op| matches!(op, SceneOp::Quad { .. }))
    );
    assert!(
        scene
            .ops()
            .iter()
            .any(|op| matches!(op, SceneOp::Text { .. }))
    );
}

#[test]
fn dock_space_clears_hover_on_drop_without_drag_session() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let mut app = TestHost::new();
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("core.hierarchy")],
            active: 0,
        });
        dock.graph.set_window_root(window, tabs);
        dock.panels.insert(
            PanelKey::new("core.hierarchy"),
            DockPanel {
                title: "Hierarchy".to_string(),
                color: Color::TRANSPARENT,
                viewport: None,
            },
        );
        dock.hover = Some(DockDropTarget::Float { window });
    });

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let _bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    ui.layout(&mut app, &mut text, root, size, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: Point::new(Px(12.0), Px(12.0)),
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
        }),
    );

    let hover = app.global::<DockManager>().and_then(|d| d.hover.clone());
    assert!(hover.is_none(), "dock hover should be cleared on drop");
}

#[test]
fn dock_drag_suppresses_viewport_hover_and_wheel_forwarding() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    harness.app.begin_cross_window_drag_with_kind(
        DragKind::DockPanel,
        harness.window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.viewport"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
        },
    );

    let position = harness.viewport_point();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
        }),
    );
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Wheel {
            position,
            delta: Point::new(Px(0.0), Px(12.0)),
            modifiers: Modifiers::default(),
        }),
    );

    let effects = harness.app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "dock drag must suppress viewport hover/wheel forwarding (ADR 0072), got: {effects:?}",
    );
}

#[test]
fn viewport_capture_emits_clamped_pointer_moves_outside_draw_rect() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    let down_pos = harness.viewport_point();
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
        }),
    );
    let _ = harness.app.take_effects();

    let outside = Point::new(Px(-50.0), Px(-50.0));
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: outside,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
        }),
    );

    let effects = harness.app.take_effects();
    let Some(Effect::ViewportInput(input)) = effects
        .iter()
        .find(|e| matches!(e, Effect::ViewportInput(_)))
    else {
        panic!("expected a ViewportInput effect during viewport capture, got: {effects:?}");
    };

    assert_eq!(
        input.kind,
        ViewportInputKind::PointerMove {
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
        }
    );
    assert_eq!(input.uv, (0.0, 0.0), "expected clamped uv at top-left");
    assert_eq!(
        input.target_px,
        (0, 0),
        "expected clamped target_px at top-left"
    );
}

#[test]
fn dock_split_handle_hover_sets_resize_cursor_effect() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.with_global_mut(DockManager::default, |dock, _app| {
        let left = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("core.left")],
            active: 0,
        });
        let right = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("core.right")],
            active: 0,
        });
        let split = dock.graph.insert_node(DockNode::Split {
            axis: fret_core::Axis::Horizontal,
            children: vec![left, right],
            fractions: vec![0.5, 0.5],
        });
        dock.graph.set_window_root(window, split);
    });

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    let _ = ui.layout(&mut app, &mut text, root, size, 1.0);

    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let x = dock_bounds.origin.x.0 + dock_bounds.size.width.0 * 0.5;
    let y = dock_bounds.origin.y.0 + 10.0;

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(x), Px(y)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
        }),
    );

    let effects = app.take_effects();
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::CursorSetIcon { window: w, icon }
                if *w == window && *icon == fret_core::CursorIcon::ColResize
        )),
        "expected a col-resize cursor effect when hovering the split handle gap"
    );
}

#[test]
fn dock_tab_drop_outside_window_requests_float() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("core.hierarchy")],
            active: 0,
        });
        dock.graph.set_window_root(window, tabs);
        dock.panels.insert(
            PanelKey::new("core.hierarchy"),
            DockPanel {
                title: "Hierarchy".to_string(),
                color: Color::TRANSPARENT,
                viewport: None,
            },
        );
    });

    app.begin_cross_window_drag_with_kind(
        DragKind::DockPanel,
        window,
        Point::new(Px(24.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.hierarchy"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
        },
    );
    if let Some(drag) = app.drag_mut() {
        drag.dragging = true;
    }

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let _ = ui.layout(&mut app, &mut text, root, size, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: Point::new(Px(-32.0), Px(12.0)),
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
        }),
    );

    let effects = app.take_effects();
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::Dock(DockOp::RequestFloatPanelToNewWindow { panel, .. })
                if *panel == PanelKey::new("core.hierarchy")
        )),
        "expected a float request effect when dropping outside the window"
    );
}

#[test]
fn dock_tab_drop_outside_window_floats_in_window_when_tear_off_disabled() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let mut app = TestHost::new();
    let mut caps = PlatformCapabilities::default();
    caps.ui.window_tear_off = false;
    app.set_global(caps);
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("core.hierarchy")],
            active: 0,
        });
        dock.graph.set_window_root(window, tabs);
        dock.panels.insert(
            PanelKey::new("core.hierarchy"),
            DockPanel {
                title: "Hierarchy".to_string(),
                color: Color::TRANSPARENT,
                viewport: None,
            },
        );
    });

    app.begin_cross_window_drag_with_kind(
        DragKind::DockPanel,
        window,
        Point::new(Px(24.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.hierarchy"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
        },
    );
    if let Some(drag) = app.drag_mut() {
        drag.dragging = true;
    }

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let _ = ui.layout(&mut app, &mut text, root, size, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: Point::new(Px(-32.0), Px(12.0)),
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
        }),
    );

    let effects = app.take_effects();
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::Dock(DockOp::FloatPanelInWindow { panel, .. })
                if *panel == PanelKey::new("core.hierarchy")
        )),
        "expected an in-window float effect when dropping outside with tear-off disabled"
    );
}

#[test]
fn dock_tab_drop_outside_window_floats_in_window_when_multi_window_is_disabled() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let mut app = TestHost::new();
    let mut caps = PlatformCapabilities::default();
    caps.ui.multi_window = false;
    caps.ui.window_tear_off = true;
    app.set_global(caps);
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("core.hierarchy")],
            active: 0,
        });
        dock.graph.set_window_root(window, tabs);
        dock.panels.insert(
            PanelKey::new("core.hierarchy"),
            DockPanel {
                title: "Hierarchy".to_string(),
                color: Color::TRANSPARENT,
                viewport: None,
            },
        );
    });

    app.begin_cross_window_drag_with_kind(
        DragKind::DockPanel,
        window,
        Point::new(Px(24.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.hierarchy"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
        },
    );
    if let Some(drag) = app.drag_mut() {
        drag.dragging = true;
    }

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let _ = ui.layout(&mut app, &mut text, root, size, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: Point::new(Px(-32.0), Px(12.0)),
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
        }),
    );

    let effects = app.take_effects();
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::Dock(DockOp::FloatPanelInWindow { panel, .. })
                if *panel == PanelKey::new("core.hierarchy")
        )),
        "expected an in-window float effect when multi-window is disabled"
    );
}

#[test]
fn dock_tab_drop_outside_routes_to_dock_space() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(TestStack);
    let dock_space = ui.create_node_retained(DockSpace::new(window));
    ui.add_child(root, dock_space);
    ui.set_root(root);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("core.hierarchy")],
            active: 0,
        });
        dock.graph.set_window_root(window, tabs);
        dock.panels.insert(
            PanelKey::new("core.hierarchy"),
            DockPanel {
                title: "Hierarchy".to_string(),
                color: Color::TRANSPARENT,
                viewport: None,
            },
        );
    });

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    app.begin_cross_window_drag_with_kind(
        DragKind::DockPanel,
        window,
        Point::new(Px(24.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.hierarchy"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
        },
    );
    if let Some(drag) = app.drag_mut() {
        drag.dragging = true;
    }

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: Point::new(Px(-32.0), Px(12.0)),
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
        }),
    );

    let effects = app.take_effects();
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::Dock(DockOp::RequestFloatPanelToNewWindow { panel, .. })
                if *panel == PanelKey::new("core.hierarchy")
        )),
        "expected DockSpace to receive the drop even when hit-testing fails"
    );
}

#[test]
fn dock_drop_hint_rects_can_select_zone() {
    let window = AppWindowId::default();

    let mut dock = DockManager::default();
    let tabs = dock.graph.insert_node(DockNode::Tabs {
        tabs: vec![PanelKey::new("core.hierarchy")],
        active: 0,
    });
    dock.graph.set_window_root(window, tabs);

    let rect = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut layout = std::collections::HashMap::new();
    layout.insert(tabs, rect);
    let tab_scroll = std::collections::HashMap::new();

    for (expected, hint_rect) in dock_hint_rects(rect) {
        if expected == DropZone::Center {
            continue;
        }
        let position = Point::new(
            Px(hint_rect.origin.x.0 + hint_rect.size.width.0 * 0.5),
            Px(hint_rect.origin.y.0 + hint_rect.size.height.0 * 0.5),
        );
        let hit = hit_test_drop_target(&dock.graph, &layout, &tab_scroll, position)
            .expect("hit should resolve to a dock target");
        assert_eq!(hit.zone, expected);
        assert!(hit.insert_index.is_none());
    }
}
