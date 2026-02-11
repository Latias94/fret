use super::hit_test::{hit_test_drop_target, hit_test_split_handle};
use super::layout::{
    active_panel_content_bounds, compute_layout_map, dock_hint_rects_with_font, dock_space_regions,
    float_zone, split_tab_bar,
};
use super::prelude_core::*;
use super::prelude_runtime::*;
use super::prelude_ui::*;
use super::split_stabilize::{apply_same_axis_locks, compute_same_axis_locks_for_split_drag};
use super::tab_bar_geometry::TabBarGeometry;
use super::{
    DockManager, DockPanelContentService, DockPanelRegistry, DockPanelRegistryService, DockSpace,
    render_and_bind_dock_panels,
};
use super::{DockViewportLayout, ViewportPanel};
use crate::test_host::TestHost;
use fret_core::{
    AppWindowId, Event, InternalDragEvent, InternalDragKind, Modifiers, Point, Px, Scene, SceneOp,
    Size, TextBlobId, TextConstraints, TextInput, TextMetrics, TextService, UiServices,
};
use fret_runtime::DRAG_KIND_DOCK_PANEL;
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
use fret_ui::retained_bridge::UiTreeRetainedExt as _;
use fret_ui::retained_bridge::resizable_panel_group as resizable;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use fret_ui::declarative;
use fret_ui::overlay_placement::{
    Align as OverlayAlign, Side as OverlaySide, anchored_panel_bounds,
};
use fret_ui::scroll::{ScrollHandle, VirtualListScrollHandle};
use slotmap::KeyData;
use std::sync::atomic::AtomicBool;

mod bounds;
mod dock_space;
mod drag;
mod drop_hints;
mod floating;
mod split;
mod tab_bar;
mod viewport;

#[derive(Default)]
struct FakeTextService;

impl TextService for FakeTextService {
    fn prepare(
        &mut self,
        _input: &TextInput,
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

impl fret_core::MaterialService for FakeTextService {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Err(fret_core::MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        true
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
        let _ = self.paint_scene();
    }

    fn paint_scene(&mut self) -> Scene {
        let size = Size::new(Px(800.0), Px(600.0));
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
        render_and_bind_dock_panels(
            &mut self.ui,
            &mut self.app,
            &mut self.text,
            self.window,
            bounds,
            self.root,
        );
        self.ui
            .layout_all(&mut self.app, &mut self.text, bounds, 1.0);
        let mut scene = Scene::default();
        self.ui
            .paint_all(&mut self.app, &mut self.text, bounds, &mut scene, 1.0);
        scene
    }

    fn viewport_point(&self) -> Point {
        let layout = self
            .app
            .global::<DockManager>()
            .and_then(|dock| dock.viewport_layout(self.window, self.target))
            .expect("expected viewport layout to be recorded during paint");
        let rect = layout.content_rect;
        Point::new(Px(rect.origin.x.0 + 10.0), Px(rect.origin.y.0 + 10.0))
    }

    fn tab_point(&self, index: usize) -> Point {
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let (_chrome, dock_bounds) = dock_space_regions(bounds);
        let root = self
            .app
            .global::<DockManager>()
            .and_then(|dock| dock.graph.window_root(self.window))
            .expect("expected dock window root");
        let settings = fret_runtime::DockingInteractionSettings::default();
        let layout = compute_layout_map(
            &self.app.global::<DockManager>().unwrap().graph,
            root,
            dock_bounds,
            settings.split_handle_gap,
            settings.split_handle_hit_thickness,
        );
        let root_rect = layout.get(&root).copied().expect("expected root rect");
        let (tab_bar, _content) = split_tab_bar(root_rect);
        let tab_count = match self.app.global::<DockManager>().unwrap().graph.node(root) {
            Some(DockNode::Tabs { tabs, .. }) => tabs.len(),
            _ => 0,
        }
        .max(index + 1);
        let tab_rect = TabBarGeometry::fixed(tab_bar, tab_count).tab_rect(index, Px(0.0));
        Point::new(Px(tab_rect.origin.x.0 + 2.0), Px(tab_rect.origin.y.0 + 2.0))
    }
}

struct DockSplitViewportHarness {
    window: AppWindowId,
    target_left: fret_core::RenderTargetId,
    target_right: fret_core::RenderTargetId,
    root: fret_core::NodeId,
    ui: UiTree<TestHost>,
    app: TestHost,
    text: FakeTextService,
}

impl DockSplitViewportHarness {
    fn new() -> Self {
        let window = AppWindowId::default();
        let target_left = fret_core::RenderTargetId::default();
        let target_right = fret_core::RenderTargetId::from(slotmap::KeyData::from_ffi(42));

        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node_retained(DockSpace::new(window));
        ui.set_root(root);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.with_global_mut(DockManager::default, |dock, _app| {
            let panel_left = PanelKey::new("core.viewport.left");
            let panel_right = PanelKey::new("core.viewport.right");

            let left_tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel_left.clone()],
                active: 0,
            });
            let right_tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel_right.clone()],
                active: 0,
            });
            let split = dock.graph.insert_node(DockNode::Split {
                axis: fret_core::Axis::Horizontal,
                children: vec![left_tabs, right_tabs],
                fractions: vec![0.5, 0.5],
            });
            dock.graph.set_window_root(window, split);

            dock.panels.insert(
                panel_left,
                DockPanel {
                    title: "Viewport Left".to_string(),
                    color: Color::TRANSPARENT,
                    viewport: Some(super::ViewportPanel {
                        target: target_left,
                        target_px_size: (320, 240),
                        fit: fret_core::ViewportFit::Stretch,
                        context_menu_enabled: true,
                    }),
                },
            );
            dock.panels.insert(
                panel_right,
                DockPanel {
                    title: "Viewport Right".to_string(),
                    color: Color::TRANSPARENT,
                    viewport: Some(super::ViewportPanel {
                        target: target_right,
                        target_px_size: (320, 240),
                        fit: fret_core::ViewportFit::Stretch,
                        context_menu_enabled: true,
                    }),
                },
            );
        });

        Self {
            window,
            target_left,
            target_right,
            root,
            ui,
            app,
            text: FakeTextService,
        }
    }

    fn layout(&mut self) {
        let _ = self.paint_scene();
    }

    fn paint_scene(&mut self) -> Scene {
        let size = Size::new(Px(800.0), Px(600.0));
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
        render_and_bind_dock_panels(
            &mut self.ui,
            &mut self.app,
            &mut self.text,
            self.window,
            bounds,
            self.root,
        );
        self.ui
            .layout_all(&mut self.app, &mut self.text, bounds, 1.0);
        let mut scene = Scene::default();
        self.ui
            .paint_all(&mut self.app, &mut self.text, bounds, &mut scene, 1.0);
        scene
    }

    fn viewport_point(&self, target: fret_core::RenderTargetId) -> Point {
        let layout = self
            .app
            .global::<DockManager>()
            .and_then(|dock| dock.viewport_layout(self.window, target))
            .expect("expected viewport layout to be recorded during paint");
        let rect = layout.content_rect;
        Point::new(Px(rect.origin.x.0 + 10.0), Px(rect.origin.y.0 + 10.0))
    }
}

struct PropagationSpy {
    right_downs: Arc<AtomicUsize>,
    right_ups: Arc<AtomicUsize>,
}

impl PropagationSpy {
    fn new(right_downs: Arc<AtomicUsize>, right_ups: Arc<AtomicUsize>) -> Self {
        Self {
            right_downs,
            right_ups,
        }
    }
}

impl<H: UiHost> Widget<H> for PropagationSpy {
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        false
    }

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

    fn event(&mut self, _cx: &mut EventCx<'_, H>, event: &Event) {
        match event {
            Event::Pointer(fret_core::PointerEvent::Down { button, .. })
                if *button == fret_core::MouseButton::Right =>
            {
                self.right_downs.fetch_add(1, Ordering::SeqCst);
            }
            Event::Pointer(fret_core::PointerEvent::Up { button, .. })
                if *button == fret_core::MouseButton::Right =>
            {
                self.right_ups.fetch_add(1, Ordering::SeqCst);
            }
            _ => {}
        }
    }
}

struct DockViewportPropagationHarness {
    window: AppWindowId,
    target: fret_core::RenderTargetId,
    root: fret_core::NodeId,
    ui: UiTree<TestHost>,
    app: TestHost,
    text: FakeTextService,
    spy_right_downs: Arc<AtomicUsize>,
    spy_right_ups: Arc<AtomicUsize>,
}

impl DockViewportPropagationHarness {
    fn new() -> Self {
        let window = AppWindowId::default();
        let target = fret_core::RenderTargetId::default();

        let spy_right_downs = Arc::new(AtomicUsize::new(0));
        let spy_right_ups = Arc::new(AtomicUsize::new(0));

        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node_retained(PropagationSpy::new(
            spy_right_downs.clone(),
            spy_right_ups.clone(),
        ));
        let dock_space = ui.create_node_retained(DockSpace::new(window));
        ui.add_child(root, dock_space);
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
            spy_right_downs,
            spy_right_ups,
        }
    }

    fn layout(&mut self) {
        let _ = self.paint_scene();
    }

    fn paint_scene(&mut self) -> Scene {
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
        scene
    }

    fn viewport_point(&self) -> Point {
        let layout = self
            .app
            .global::<DockManager>()
            .and_then(|dock| dock.viewport_layout(self.window, self.target))
            .expect("expected viewport layout to be recorded during paint");
        let rect = layout.content_rect;
        Point::new(Px(rect.origin.x.0 + 10.0), Px(rect.origin.y.0 + 10.0))
    }

    fn reset_spy(&self) {
        self.spy_right_downs.store(0, Ordering::SeqCst);
        self.spy_right_ups.store(0, Ordering::SeqCst);
    }

    fn spy_counts(&self) -> (usize, usize) {
        (
            self.spy_right_downs.load(Ordering::SeqCst),
            self.spy_right_ups.load(Ordering::SeqCst),
        )
    }
}

struct FocusOnDown;

impl<H: UiHost> Widget<H> for FocusOnDown {
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        true
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        if matches!(event, Event::Pointer(fret_core::PointerEvent::Down { .. })) {
            cx.request_focus(cx.node);
            cx.stop_propagation();
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

struct CachedRetainedPanelRegistry {
    nodes: Mutex<std::collections::HashMap<fret_core::PanelKey, fret_core::NodeId>>,
}

impl CachedRetainedPanelRegistry {
    fn new() -> Self {
        Self {
            nodes: Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl DockPanelRegistry<TestHost> for CachedRetainedPanelRegistry {
    fn render_panel(
        &self,
        ui: &mut UiTree<TestHost>,
        _app: &mut TestHost,
        _services: &mut dyn fret_core::UiServices,
        _window: AppWindowId,
        _bounds: Rect,
        panel: &fret_core::PanelKey,
    ) -> Option<fret_core::NodeId> {
        let mut nodes = self.nodes.lock().expect("registry mutex");
        if let Some(node) = nodes.get(panel).copied() {
            return Some(node);
        }
        let node = ui.create_node_retained(FocusOnDown);
        nodes.insert(panel.clone(), node);
        Some(node)
    }
}

struct OverlayAssertsViewportBounds {
    left_spacer: fret_core::NodeId,
    right_spacer: fret_core::NodeId,
    expected_left: Rect,
    expected_right: Rect,
    ok: Arc<AtomicBool>,
}

impl<H: UiHost> Widget<H> for OverlayAssertsViewportBounds {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let left = cx.tree.debug_node_bounds(self.left_spacer);
        let right = cx.tree.debug_node_bounds(self.right_spacer);
        self.ok.store(
            left == Some(self.expected_left) && right == Some(self.expected_right),
            Ordering::Relaxed,
        );
        cx.available
    }
}

struct OverlayAssertsLastFrameElementBounds {
    window: AppWindowId,
    left_element: fret_ui::elements::GlobalElementId,
    right_element: fret_ui::elements::GlobalElementId,
    left_node: fret_core::NodeId,
    right_node: fret_core::NodeId,
    expected_left_last: Rect,
    expected_right_last: Rect,
    expected_left_now: Rect,
    expected_right_now: Rect,
    outer: Rect,
    desired: Size,
    side_offset: Px,
    side: OverlaySide,
    align: OverlayAlign,
    ok: Arc<AtomicBool>,
}

impl<H: UiHost> Widget<H> for OverlayAssertsLastFrameElementBounds {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let left_last =
            fret_ui::elements::bounds_for_element(cx.app, self.window, self.left_element);
        let right_last =
            fret_ui::elements::bounds_for_element(cx.app, self.window, self.right_element);
        let left_now = cx.tree.debug_node_bounds(self.left_node);
        let right_now = cx.tree.debug_node_bounds(self.right_node);

        let placed = left_last.map(|anchor| {
            anchored_panel_bounds(
                self.outer,
                anchor,
                self.desired,
                self.side_offset,
                self.side,
                self.align,
            )
        });
        let expected_placed_last = anchored_panel_bounds(
            self.outer,
            self.expected_left_last,
            self.desired,
            self.side_offset,
            self.side,
            self.align,
        );
        let expected_placed_now = anchored_panel_bounds(
            self.outer,
            self.expected_left_now,
            self.desired,
            self.side_offset,
            self.side,
            self.align,
        );

        self.ok.store(
            left_last == Some(self.expected_left_last)
                && right_last == Some(self.expected_right_last)
                && left_now == Some(self.expected_left_now)
                && right_now == Some(self.expected_right_now),
            Ordering::Relaxed,
        );
        assert!(
            expected_placed_last != expected_placed_now,
            "expected anchor change to affect overlay placement"
        );
        assert_eq!(
            placed,
            Some(expected_placed_last),
            "expected overlay placement to use last-frame anchor bounds"
        );

        cx.available
    }
}

struct OverlayAssertsWindowScopedBoundsForElement {
    window_a: AppWindowId,
    window_b: AppWindowId,
    element_a: fret_ui::elements::GlobalElementId,
    element_b: fret_ui::elements::GlobalElementId,
    expected_a_last: Rect,
    expected_b_last: Rect,
    ok: Arc<AtomicBool>,
}

impl<H: UiHost> Widget<H> for OverlayAssertsWindowScopedBoundsForElement {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let a_from_a = fret_ui::elements::bounds_for_element(cx.app, self.window_a, self.element_a);
        let b_from_b = fret_ui::elements::bounds_for_element(cx.app, self.window_b, self.element_b);
        let a_from_b = fret_ui::elements::bounds_for_element(cx.app, self.window_b, self.element_a);
        let b_from_a = fret_ui::elements::bounds_for_element(cx.app, self.window_a, self.element_b);

        self.ok.store(
            a_from_a == Some(self.expected_a_last)
                && b_from_b == Some(self.expected_b_last)
                && a_from_b.is_none()
                && b_from_a.is_none(),
            Ordering::Relaxed,
        );

        cx.available
    }
}

struct OverlayAssertsWindowLocalOverlayPlacement {
    window_a: AppWindowId,
    window_b: AppWindowId,
    element_a: fret_ui::elements::GlobalElementId,
    element_b: fret_ui::elements::GlobalElementId,
    expected_a_last: Rect,
    expected_b_last: Rect,
    outer: Rect,
    desired: Size,
    side_offset: Px,
    side: OverlaySide,
    align: OverlayAlign,
    ok: Arc<AtomicBool>,
}

impl<H: UiHost> Widget<H> for OverlayAssertsWindowLocalOverlayPlacement {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let anchor_b = fret_ui::elements::bounds_for_element(cx.app, self.window_b, self.element_b);
        let anchor_a_wrong =
            fret_ui::elements::bounds_for_element(cx.app, self.window_b, self.element_a);
        let anchor_a_other_window =
            fret_ui::elements::bounds_for_element(cx.app, self.window_a, self.element_a);

        assert_eq!(
            anchor_b,
            Some(self.expected_b_last),
            "expected element_b bounds_for_element to return last-frame bounds"
        );
        assert!(
            anchor_a_wrong.is_none(),
            "expected bounds_for_element to be window-scoped (window_b must not see window_a elements)"
        );
        assert_eq!(
            anchor_a_other_window,
            Some(self.expected_a_last),
            "expected window_a bounds_for_element to return last-frame bounds"
        );

        let placed_b = anchored_panel_bounds(
            self.outer,
            anchor_b.expect("anchor b exists"),
            self.desired,
            self.side_offset,
            self.side,
            self.align,
        );
        let expected_placed_b = anchored_panel_bounds(
            self.outer,
            self.expected_b_last,
            self.desired,
            self.side_offset,
            self.side,
            self.align,
        );
        assert_eq!(
            placed_b, expected_placed_b,
            "expected overlay placement to use window-local anchor bounds"
        );

        self.ok.store(true, Ordering::Relaxed);

        cx.available
    }
}
