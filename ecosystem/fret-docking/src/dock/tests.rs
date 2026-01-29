use super::hit_test::hit_test_drop_target;
use super::layout::{
    active_panel_content_bounds, compute_layout_map, dock_hint_rects_with_font, dock_space_regions,
    split_tab_bar,
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

#[cfg(feature = "layout-engine-v2")]
use fret_ui::declarative;
#[cfg(feature = "layout-engine-v2")]
use fret_ui::overlay_placement::{
    Align as OverlayAlign, Side as OverlaySide, anchored_panel_bounds,
};
#[cfg(feature = "layout-engine-v2")]
use fret_ui::scroll::{ScrollHandle, VirtualListScrollHandle};
#[cfg(feature = "layout-engine-v2")]
use slotmap::KeyData;
#[cfg(feature = "layout-engine-v2")]
use std::sync::atomic::AtomicBool;

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
        let layout = compute_layout_map(
            &self.app.global::<DockManager>().unwrap().graph,
            root,
            dock_bounds,
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

#[cfg(feature = "layout-engine-v2")]
struct OverlayAssertsViewportBounds {
    left_spacer: fret_core::NodeId,
    right_spacer: fret_core::NodeId,
    expected_left: Rect,
    expected_right: Rect,
    ok: Arc<AtomicBool>,
}

#[cfg(feature = "layout-engine-v2")]
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

#[cfg(feature = "layout-engine-v2")]
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

#[cfg(feature = "layout-engine-v2")]
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

#[cfg(feature = "layout-engine-v2")]
struct OverlayAssertsWindowScopedBoundsForElement {
    window_a: AppWindowId,
    window_b: AppWindowId,
    element_a: fret_ui::elements::GlobalElementId,
    element_b: fret_ui::elements::GlobalElementId,
    expected_a_last: Rect,
    expected_b_last: Rect,
    ok: Arc<AtomicBool>,
}

#[cfg(feature = "layout-engine-v2")]
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

#[cfg(feature = "layout-engine-v2")]
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

#[cfg(feature = "layout-engine-v2")]
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

#[cfg(feature = "layout-engine-v2")]
#[test]
fn docking_viewport_panels_are_laid_out_before_overlay_layout_and_do_not_couple_fill() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(600.0), Px(200.0)),
    );

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut services = FakeTextService::default();

    let panel_left = PanelKey::new("test.viewport.left");
    let panel_right = PanelKey::new("test.viewport.right");

    let target_left = fret_core::RenderTargetId::from(KeyData::from_ffi(1));
    let target_right = fret_core::RenderTargetId::from(KeyData::from_ffi(2));

    let left_root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "dock-test-left",
        |cx| {
            let flex = fret_ui::element::FlexProps {
                direction: fret_core::Axis::Vertical,
                layout: fret_ui::element::LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: fret_ui::element::Length::Fill,
                        height: fret_ui::element::Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            };
            vec![cx.flex(flex, |cx| {
                vec![cx.spacer(fret_ui::element::SpacerProps::default())]
            })]
        },
    );
    let right_root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "dock-test-right",
        |cx| {
            let flex = fret_ui::element::FlexProps {
                direction: fret_core::Axis::Vertical,
                layout: fret_ui::element::LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: fret_ui::element::Length::Fill,
                        height: fret_ui::element::Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            };
            vec![cx.flex(flex, |cx| {
                vec![cx.spacer(fret_ui::element::SpacerProps::default())]
            })]
        },
    );

    let left_flex = ui.children(left_root)[0];
    let left_spacer = ui.children(left_flex)[0];

    let right_flex = ui.children(right_root)[0];
    let right_spacer = ui.children(right_flex)[0];

    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel_left, || DockPanel {
            title: "Left".to_string(),
            color: Color::TRANSPARENT,
            viewport: Some(super::ViewportPanel {
                target: target_left,
                target_px_size: (320, 240),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
        });
        dock.ensure_panel(&panel_right, || DockPanel {
            title: "Right".to_string(),
            color: Color::TRANSPARENT,
            viewport: Some(super::ViewportPanel {
                target: target_right,
                target_px_size: (320, 240),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
        });

        let left_tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_left.clone()],
            active: 0,
        });
        let right_tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_right.clone()],
            active: 0,
        });
        let root = dock.graph.insert_node(DockNode::Split {
            axis: fret_core::Axis::Horizontal,
            children: vec![left_tabs, right_tabs],
            fractions: vec![0.35, 0.65],
        });
        dock.graph.set_window_root(window, root);
    });

    let (expected_left, expected_right) = {
        let dock = app.global::<DockManager>().expect("dock manager");
        let root = dock.graph.window_root(window).expect("dock root");
        let (_chrome, dock_bounds) = dock_space_regions(bounds);
        let layout = compute_layout_map(&dock.graph, root, dock_bounds);
        let active = active_panel_content_bounds(&dock.graph, &layout);
        let left = active.get(&panel_left).copied().expect("left bounds");
        let right = active.get(&panel_right).copied().expect("right bounds");
        (left, right)
    };

    let dock_space = ui.create_node_retained(
        DockSpace::new(window)
            .with_panel_content(panel_left.clone(), left_root)
            .with_panel_content(panel_right.clone(), right_root),
    );
    ui.set_children(dock_space, vec![left_root, right_root]);
    ui.set_root(dock_space);

    let ok = Arc::new(AtomicBool::new(false));
    let overlay = ui.create_node_retained(OverlayAssertsViewportBounds {
        left_spacer,
        right_spacer,
        expected_left,
        expected_right,
        ok: ok.clone(),
    });
    ui.push_overlay_root(overlay, false);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(
        ok.load(Ordering::Relaxed),
        "expected overlay layout to observe viewport-laid-out spacer bounds"
    );
    assert_eq!(ui.debug_node_bounds(left_spacer), Some(expected_left));
    assert_eq!(ui.debug_node_bounds(right_spacer), Some(expected_right));

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let dock = app.global::<DockManager>().expect("dock manager");
    assert_eq!(
        dock.viewport_content_rect(window, target_left),
        Some(expected_left)
    );
    assert_eq!(
        dock.viewport_content_rect(window, target_right),
        Some(expected_right)
    );
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn docking_bounds_for_element_reports_last_frame_panel_rects() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(600.0), Px(200.0)),
    );

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut services = FakeTextService::default();

    let panel_left = PanelKey::new("test.viewport.left");
    let panel_right = PanelKey::new("test.viewport.right");

    let target_left = fret_core::RenderTargetId::from(KeyData::from_ffi(1));
    let target_right = fret_core::RenderTargetId::from(KeyData::from_ffi(2));

    let split_root = app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel_left, || DockPanel {
            title: "Left".to_string(),
            color: Color::TRANSPARENT,
            viewport: Some(super::ViewportPanel {
                target: target_left,
                target_px_size: (320, 240),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
        });
        dock.ensure_panel(&panel_right, || DockPanel {
            title: "Right".to_string(),
            color: Color::TRANSPARENT,
            viewport: Some(super::ViewportPanel {
                target: target_right,
                target_px_size: (320, 240),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
        });

        let left_tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_left.clone()],
            active: 0,
        });
        let right_tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_right.clone()],
            active: 0,
        });
        let root = dock.graph.insert_node(DockNode::Split {
            axis: fret_core::Axis::Horizontal,
            children: vec![left_tabs, right_tabs],
            fractions: vec![0.35, 0.65],
        });
        dock.graph.set_window_root(window, root);
        root
    });

    let left_root_name = "dock-geom-left";
    let right_root_name = "dock-geom-right";

    let left_node = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        left_root_name,
        |cx| vec![cx.text("left")],
    );
    let right_node = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        right_root_name,
        |cx| vec![cx.text("right")],
    );

    let dock_space = ui.create_node_retained(
        DockSpace::new(window)
            .with_panel_content(panel_left.clone(), left_node)
            .with_panel_content(panel_right.clone(), right_node),
    );
    ui.set_children(dock_space, vec![left_node, right_node]);
    ui.set_root(dock_space);

    let (expected_left_0, expected_right_0) = {
        let dock = app.global::<DockManager>().expect("dock manager");
        let root = dock.graph.window_root(window).expect("dock root");
        let (_chrome, dock_bounds) = dock_space_regions(bounds);
        let layout = compute_layout_map(&dock.graph, root, dock_bounds);
        let active = active_panel_content_bounds(&dock.graph, &layout);
        (
            active.get(&panel_left).copied().expect("left bounds"),
            active.get(&panel_right).copied().expect("right bounds"),
        )
    };

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let left_element = fret_ui::elements::global_root(window, left_root_name);
    let right_element = fret_ui::elements::global_root(window, right_root_name);
    let left_root_node =
        fret_ui::elements::node_for_element(&mut app, window, left_element).expect("left node");
    let right_root_node =
        fret_ui::elements::node_for_element(&mut app, window, right_element).expect("right node");

    assert_eq!(left_root_node, left_node);
    assert_eq!(right_root_node, right_node);
    assert_eq!(ui.debug_node_bounds(left_root_node), Some(expected_left_0));
    assert_eq!(
        ui.debug_node_bounds(right_root_node),
        Some(expected_right_0)
    );

    app.with_global_mut(DockManager::default, |dock, _app| {
        assert!(
            dock.graph
                .update_split_fractions(split_root, vec![0.5, 0.5]),
            "expected split fraction update to succeed"
        );
    });
    ui.invalidate(dock_space, Invalidation::Layout);

    let (expected_left_1, expected_right_1) = {
        let dock = app.global::<DockManager>().expect("dock manager");
        let root = dock.graph.window_root(window).expect("dock root");
        let (_chrome, dock_bounds) = dock_space_regions(bounds);
        let layout = compute_layout_map(&dock.graph, root, dock_bounds);
        let active = active_panel_content_bounds(&dock.graph, &layout);
        (
            active.get(&panel_left).copied().expect("left bounds"),
            active.get(&panel_right).copied().expect("right bounds"),
        )
    };
    assert_ne!(expected_left_0, expected_left_1);
    assert_ne!(expected_right_0, expected_right_1);

    app.advance_frame();

    let _ = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        left_root_name,
        |cx| vec![cx.text("left")],
    );
    let _ = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        right_root_name,
        |cx| vec![cx.text("right")],
    );

    let ok = Arc::new(AtomicBool::new(false));
    let overlay = ui.create_node_retained(OverlayAssertsLastFrameElementBounds {
        window,
        left_element,
        right_element,
        left_node: left_root_node,
        right_node: right_root_node,
        expected_left_last: expected_left_0,
        expected_right_last: expected_right_0,
        expected_left_now: expected_left_1,
        expected_right_now: expected_right_1,
        outer: bounds,
        desired: Size::new(Px(80.0), Px(24.0)),
        side_offset: Px(6.0),
        side: OverlaySide::Bottom,
        align: OverlayAlign::End,
        ok: ok.clone(),
    });
    ui.push_overlay_root(overlay, false);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(
        ok.load(Ordering::Relaxed),
        "expected overlay layout to read last-frame element bounds while observing current viewport layout"
    );
    assert_eq!(
        fret_ui::elements::bounds_for_element(&mut app, window, left_element),
        Some(expected_left_0)
    );
    assert_eq!(
        fret_ui::elements::bounds_for_element(&mut app, window, right_element),
        Some(expected_right_0)
    );
    assert_eq!(ui.debug_node_bounds(left_root_node), Some(expected_left_1));
    assert_eq!(
        ui.debug_node_bounds(right_root_node),
        Some(expected_right_1)
    );
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn docking_viewport_panels_keep_scroll_and_virtual_list_extents_constraint_correct() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(600.0), Px(240.0)),
    );

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut services = FakeTextService::default();

    let panel_left = PanelKey::new("test.viewport.scroll");
    let panel_right = PanelKey::new("test.viewport.vlist");

    let target_left = fret_core::RenderTargetId::from(KeyData::from_ffi(3));
    let target_right = fret_core::RenderTargetId::from(KeyData::from_ffi(4));

    let scroll_handle = ScrollHandle::default();
    let vlist_handle = VirtualListScrollHandle::new();

    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel_left, || DockPanel {
            title: "Scroll".to_string(),
            color: Color::TRANSPARENT,
            viewport: Some(super::ViewportPanel {
                target: target_left,
                target_px_size: (320, 240),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
        });
        dock.ensure_panel(&panel_right, || DockPanel {
            title: "List".to_string(),
            color: Color::TRANSPARENT,
            viewport: Some(super::ViewportPanel {
                target: target_right,
                target_px_size: (320, 240),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
        });

        let left_tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_left.clone()],
            active: 0,
        });
        let right_tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_right.clone()],
            active: 0,
        });
        let root = dock.graph.insert_node(DockNode::Split {
            axis: fret_core::Axis::Horizontal,
            children: vec![left_tabs, right_tabs],
            fractions: vec![0.5, 0.5],
        });
        dock.graph.set_window_root(window, root);
    });

    fn build_scroll_panel(
        cx: &mut fret_ui::ElementContext<'_, TestHost>,
        handle: ScrollHandle,
    ) -> Vec<fret_ui::element::AnyElement> {
        let mut props = fret_ui::element::ScrollProps::default();
        props.layout.size.width = fret_ui::element::Length::Fill;
        props.layout.size.height = fret_ui::element::Length::Fill;
        props.axis = fret_ui::element::ScrollAxis::Y;
        props.scroll_handle = Some(handle);
        props.probe_unbounded = true;

        vec![cx.scroll(props, |cx| {
            let flex = fret_ui::element::FlexProps {
                direction: fret_core::Axis::Vertical,
                layout: fret_ui::element::LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: fret_ui::element::Length::Fill,
                        height: fret_ui::element::Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            };
            vec![cx.flex(flex, |cx| {
                vec![
                    cx.text("hello"),
                    cx.spacer(fret_ui::element::SpacerProps::default()),
                ]
            })]
        })]
    }

    fn build_vlist_panel(
        cx: &mut fret_ui::ElementContext<'_, TestHost>,
        handle: &VirtualListScrollHandle,
    ) -> Vec<fret_ui::element::AnyElement> {
        let options = fret_ui::element::VirtualListOptions::new(Px(10.0), 0);
        vec![cx.virtual_list(50, options, handle, |cx, items| {
            items
                .iter()
                .copied()
                .map(|item| {
                    cx.keyed(item.key, |cx| {
                        let flex = fret_ui::element::FlexProps {
                            direction: fret_core::Axis::Vertical,
                            layout: fret_ui::element::LayoutStyle {
                                size: fret_ui::element::SizeStyle {
                                    width: fret_ui::element::Length::Fill,
                                    height: fret_ui::element::Length::Auto,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        };
                        cx.flex(flex, |cx| {
                            vec![
                                cx.text("row"),
                                cx.spacer(fret_ui::element::SpacerProps::default()),
                            ]
                        })
                    })
                })
                .collect::<Vec<_>>()
        })]
    }

    let left_root_name = "dock-scroll-panel";
    let right_root_name = "dock-vlist-panel";

    let left_node = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        left_root_name,
        |cx| build_scroll_panel(cx, scroll_handle.clone()),
    );
    let right_node = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        right_root_name,
        |cx| build_vlist_panel(cx, &vlist_handle),
    );

    let dock_space = ui.create_node_retained(
        DockSpace::new(window)
            .with_panel_content(panel_left.clone(), left_node)
            .with_panel_content(panel_right.clone(), right_node),
    );
    ui.set_children(dock_space, vec![left_node, right_node]);
    ui.set_root(dock_space);

    // Frame 0: virtual list has not recorded viewport size yet, so it will mount no rows. Scroll
    // extents should still be constraint-correct and should not explode due to unbounded probes.
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let scroll_extent_0 = scroll_handle.content_size().height;
    assert!(
        scroll_extent_0.0 > 10.0 && scroll_extent_0.0 < 300.0,
        "expected scroll content height to stay bounded, got {scroll_extent_0:?}"
    );

    // Frame 1: virtual list now mounts rows and measures them. Its content extent must still stay
    // bounded (no 1e9-style probe expansion).
    app.advance_frame();
    let _ = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        left_root_name,
        |cx| build_scroll_panel(cx, scroll_handle.clone()),
    );
    let _ = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        right_root_name,
        |cx| build_vlist_panel(cx, &vlist_handle),
    );
    ui.invalidate(dock_space, Invalidation::Layout);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let scroll_extent_1 = scroll_handle.content_size().height;
    assert!(
        scroll_extent_1.0 > 10.0 && scroll_extent_1.0 < 300.0,
        "expected scroll content height to stay bounded after second frame, got {scroll_extent_1:?}"
    );

    let list_extent = vlist_handle.content_size().height;
    assert!(
        list_extent.0 > 100.0 && list_extent.0 < 100_000.0,
        "expected virtual list extent to be finite and measured, got {list_extent:?}"
    );
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn bounds_for_element_is_window_scoped_across_multi_window_docking() {
    let window_a = AppWindowId::default();
    let window_b = AppWindowId::from(KeyData::from_ffi(42));

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(600.0), Px(240.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(480.0), Px(200.0)),
    );

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut services = FakeTextService::default();

    let panel_a = PanelKey::new("mw.viewport.a");
    let panel_b = PanelKey::new("mw.viewport.b");
    let target_a = fret_core::RenderTargetId::from(KeyData::from_ffi(10));
    let target_b = fret_core::RenderTargetId::from(KeyData::from_ffi(11));

    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel_a, || DockPanel {
            title: "A".to_string(),
            color: Color::TRANSPARENT,
            viewport: Some(super::ViewportPanel {
                target: target_a,
                target_px_size: (320, 240),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
        });
        dock.ensure_panel(&panel_b, || DockPanel {
            title: "B".to_string(),
            color: Color::TRANSPARENT,
            viewport: Some(super::ViewportPanel {
                target: target_b,
                target_px_size: (320, 240),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
        });

        let tabs_a = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_a.clone()],
            active: 0,
        });
        dock.graph.set_window_root(window_a, tabs_a);

        let tabs_b = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_b.clone()],
            active: 0,
        });
        dock.graph.set_window_root(window_b, tabs_b);
    });

    let root_a_name = "mw-panel-a";
    let root_b_name = "mw-panel-b";

    let mut ui_a: UiTree<TestHost> = UiTree::new();
    ui_a.set_window(window_a);
    let node_a = declarative::render_root(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds_a,
        root_a_name,
        |cx| vec![cx.text("a")],
    );
    let dock_space_a = ui_a
        .create_node_retained(DockSpace::new(window_a).with_panel_content(panel_a.clone(), node_a));
    ui_a.set_children(dock_space_a, vec![node_a]);
    ui_a.set_root(dock_space_a);

    let mut ui_b: UiTree<TestHost> = UiTree::new();
    ui_b.set_window(window_b);
    let node_b = declarative::render_root(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds_b,
        root_b_name,
        |cx| vec![cx.text("b")],
    );
    let dock_space_b = ui_b
        .create_node_retained(DockSpace::new(window_b).with_panel_content(panel_b.clone(), node_b));
    ui_b.set_children(dock_space_b, vec![node_b]);
    ui_b.set_root(dock_space_b);

    // Frame 0: write current bounds (stored as "cur_bounds" in the element runtime).
    ui_a.layout_all(&mut app, &mut services, bounds_a, 1.0);
    ui_b.layout_all(&mut app, &mut services, bounds_b, 1.0);

    let expected_a_0 = ui_a
        .debug_node_bounds(node_a)
        .expect("expected window a bounds");
    let expected_b_0 = ui_b
        .debug_node_bounds(node_b)
        .expect("expected window b bounds");

    let element_a = fret_ui::elements::global_root(window_a, root_a_name);
    let element_b = fret_ui::elements::global_root(window_b, root_b_name);

    // Frame 1: swap prev/cur, so `bounds_for_element` returns frame 0 bounds.
    app.advance_frame();
    let _ = declarative::render_root(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds_a,
        root_a_name,
        |cx| vec![cx.text("a")],
    );
    let _ = declarative::render_root(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds_b,
        root_b_name,
        |cx| vec![cx.text("b")],
    );
    ui_a.invalidate(dock_space_a, Invalidation::Layout);
    ui_b.invalidate(dock_space_b, Invalidation::Layout);

    let ok = Arc::new(AtomicBool::new(false));
    let overlay = ui_b.create_node_retained(OverlayAssertsWindowScopedBoundsForElement {
        window_a,
        window_b,
        element_a,
        element_b,
        expected_a_last: expected_a_0,
        expected_b_last: expected_b_0,
        ok: ok.clone(),
    });
    ui_b.push_overlay_root(overlay, false);

    ui_a.layout_all(&mut app, &mut services, bounds_a, 1.0);
    ui_b.layout_all(&mut app, &mut services, bounds_b, 1.0);

    assert!(
        ok.load(Ordering::Relaxed),
        "expected bounds_for_element to be window-scoped across multi-window docking"
    );
}

#[cfg(feature = "layout-engine-v2")]
#[test]
fn overlay_placement_must_use_window_local_anchor_bounds() {
    let window_a = AppWindowId::default();
    let window_b = AppWindowId::from(KeyData::from_ffi(42));

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(600.0), Px(240.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(480.0), Px(200.0)),
    );

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut services = FakeTextService::default();

    let panel_a = PanelKey::new("mw.viewport.a");
    let panel_b = PanelKey::new("mw.viewport.b");
    let target_a = fret_core::RenderTargetId::from(KeyData::from_ffi(10));
    let target_b = fret_core::RenderTargetId::from(KeyData::from_ffi(11));

    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel_a, || DockPanel {
            title: "A".to_string(),
            color: Color::TRANSPARENT,
            viewport: Some(super::ViewportPanel {
                target: target_a,
                target_px_size: (320, 240),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
        });
        dock.ensure_panel(&panel_b, || DockPanel {
            title: "B".to_string(),
            color: Color::TRANSPARENT,
            viewport: Some(super::ViewportPanel {
                target: target_b,
                target_px_size: (320, 240),
                fit: fret_core::ViewportFit::Stretch,
                context_menu_enabled: true,
            }),
        });

        let tabs_a = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_a.clone()],
            active: 0,
        });
        dock.graph.set_window_root(window_a, tabs_a);

        let tabs_b = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_b.clone()],
            active: 0,
        });
        dock.graph.set_window_root(window_b, tabs_b);
    });

    let root_a_name = "mw-panel-a";
    let root_b_name = "mw-panel-b";

    let anchor_a_id: Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>> =
        Arc::new(Mutex::new(None));
    let anchor_b_id: Arc<Mutex<Option<fret_ui::elements::GlobalElementId>>> =
        Arc::new(Mutex::new(None));

    let mut ui_a: UiTree<TestHost> = UiTree::new();
    ui_a.set_window(window_a);
    let anchor_a_id_setter = anchor_a_id.clone();
    let node_a = declarative::render_root(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds_a,
        root_a_name,
        move |cx| {
            let mut layout = fret_ui::element::LayoutStyle::default();
            layout.position = fret_ui::element::PositionStyle::Absolute;
            layout.inset.top = Some(Px(20.0));
            layout.inset.left = Some(Px(240.0));
            layout.size.width = fret_ui::element::Length::Px(Px(100.0));
            layout.size.height = fret_ui::element::Length::Px(Px(30.0));

            let props = fret_ui::element::SemanticsProps {
                layout,
                ..Default::default()
            };
            vec![cx.semantics_with_id(props, move |cx, id| {
                *anchor_a_id_setter.lock().expect("anchor mutex") = Some(id);
                vec![cx.text("a")]
            })]
        },
    );
    let dock_space_a = ui_a
        .create_node_retained(DockSpace::new(window_a).with_panel_content(panel_a.clone(), node_a));
    ui_a.set_children(dock_space_a, vec![node_a]);
    ui_a.set_root(dock_space_a);

    let mut ui_b: UiTree<TestHost> = UiTree::new();
    ui_b.set_window(window_b);
    let anchor_b_id_setter = anchor_b_id.clone();
    let node_b = declarative::render_root(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds_b,
        root_b_name,
        move |cx| {
            let mut layout = fret_ui::element::LayoutStyle::default();
            layout.position = fret_ui::element::PositionStyle::Absolute;
            layout.inset.top = Some(Px(20.0));
            layout.inset.left = Some(Px(40.0));
            layout.size.width = fret_ui::element::Length::Px(Px(100.0));
            layout.size.height = fret_ui::element::Length::Px(Px(30.0));

            let props = fret_ui::element::SemanticsProps {
                layout,
                ..Default::default()
            };
            vec![cx.semantics_with_id(props, move |cx, id| {
                *anchor_b_id_setter.lock().expect("anchor mutex") = Some(id);
                vec![cx.text("b")]
            })]
        },
    );
    let dock_space_b = ui_b
        .create_node_retained(DockSpace::new(window_b).with_panel_content(panel_b.clone(), node_b));
    ui_b.set_children(dock_space_b, vec![node_b]);
    ui_b.set_root(dock_space_b);

    let element_a = anchor_a_id.lock().expect("anchor mutex").unwrap();
    let element_b = anchor_b_id.lock().expect("anchor mutex").unwrap();

    // Frame 0: record current bounds into the element runtime's "cur" storage.
    ui_a.layout_all(&mut app, &mut services, bounds_a, 1.0);
    ui_b.layout_all(&mut app, &mut services, bounds_b, 1.0);

    let node_a_anchor = fret_ui::elements::node_for_element(&mut app, window_a, element_a)
        .expect("expected window a anchor node");
    let node_b_anchor = fret_ui::elements::node_for_element(&mut app, window_b, element_b)
        .expect("expected window b anchor node");

    let expected_a_0 = ui_a
        .debug_node_bounds(node_a_anchor)
        .expect("expected window a anchor bounds");
    let expected_b_0 = ui_b
        .debug_node_bounds(node_b_anchor)
        .expect("expected window b anchor bounds");

    app.advance_frame();
    let anchor_a_id_setter = anchor_a_id.clone();
    let _ = declarative::render_root(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds_a,
        root_a_name,
        move |cx| {
            let mut layout = fret_ui::element::LayoutStyle::default();
            layout.position = fret_ui::element::PositionStyle::Absolute;
            layout.inset.top = Some(Px(20.0));
            layout.inset.left = Some(Px(240.0));
            layout.size.width = fret_ui::element::Length::Px(Px(100.0));
            layout.size.height = fret_ui::element::Length::Px(Px(30.0));

            let props = fret_ui::element::SemanticsProps {
                layout,
                ..Default::default()
            };
            vec![cx.semantics_with_id(props, move |cx, id| {
                *anchor_a_id_setter.lock().expect("anchor mutex") = Some(id);
                vec![cx.text("a")]
            })]
        },
    );
    let anchor_b_id_setter = anchor_b_id.clone();
    let _ = declarative::render_root(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds_b,
        root_b_name,
        move |cx| {
            let mut layout = fret_ui::element::LayoutStyle::default();
            layout.position = fret_ui::element::PositionStyle::Absolute;
            layout.inset.top = Some(Px(20.0));
            layout.inset.left = Some(Px(40.0));
            layout.size.width = fret_ui::element::Length::Px(Px(100.0));
            layout.size.height = fret_ui::element::Length::Px(Px(30.0));

            let props = fret_ui::element::SemanticsProps {
                layout,
                ..Default::default()
            };
            vec![cx.semantics_with_id(props, move |cx, id| {
                *anchor_b_id_setter.lock().expect("anchor mutex") = Some(id);
                vec![cx.text("b")]
            })]
        },
    );
    ui_a.invalidate(dock_space_a, Invalidation::Layout);
    ui_b.invalidate(dock_space_b, Invalidation::Layout);

    let ok = Arc::new(AtomicBool::new(false));
    let overlay = ui_b.create_node_retained(OverlayAssertsWindowLocalOverlayPlacement {
        window_a,
        window_b,
        element_a,
        element_b,
        expected_a_last: expected_a_0,
        expected_b_last: expected_b_0,
        outer: bounds_b,
        desired: Size::new(Px(80.0), Px(24.0)),
        side_offset: Px(6.0),
        side: OverlaySide::Bottom,
        align: OverlayAlign::End,
        ok: ok.clone(),
    });
    ui_b.push_overlay_root(overlay, false);

    ui_a.layout_all(&mut app, &mut services, bounds_a, 1.0);
    ui_b.layout_all(&mut app, &mut services, bounds_b, 1.0);

    assert!(
        ok.load(Ordering::Relaxed),
        "expected overlay placement to only use window-local anchor bounds"
    );
}

#[test]
fn render_and_bind_dock_panels_keeps_non_viewport_panel_alive() {
    let window = AppWindowId::default();
    let panel = fret_core::PanelKey::new("demo.controls");

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    let dock_space = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(dock_space);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(CachedRetainedPanelRegistry::new()));
        },
    );

    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel, || crate::DockPanel {
            title: "Controls".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        dock.graph.set_window_root(window, tabs);
    });

    let mut services = FakeTextService;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(120.0)),
    );

    render_and_bind_dock_panels(&mut ui, &mut app, &mut services, window, bounds, dock_space);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let node = app
        .global::<DockPanelContentService>()
        .and_then(|svc| svc.get(window, &panel))
        .expect("expected panel node to be bound");

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(60.0)),
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        ui.focus(),
        Some(node),
        "expected bound panel node to be focusable and receive pointer events"
    );
}

#[test]
fn dock_space_layout_assigns_active_panel_content_bounds_via_panel_nodes() {
    let window = AppWindowId::default();
    let panel = fret_core::PanelKey::new("demo.controls");

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    let dock_space = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(dock_space);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    app.with_global_mut(
        DockPanelRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(CachedRetainedPanelRegistry::new()));
        },
    );

    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.ensure_panel(&panel, || crate::DockPanel {
            title: "Controls".to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        });
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        dock.graph.set_window_root(window, tabs);
    });

    let mut services = FakeTextService;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    );

    render_and_bind_dock_panels(&mut ui, &mut app, &mut services, window, bounds, dock_space);

    let root = app
        .global::<DockManager>()
        .and_then(|dock| dock.graph.window_root(window))
        .expect("expected dock window root");
    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let dock_layout = compute_layout_map(
        &app.global::<DockManager>().unwrap().graph,
        root,
        dock_bounds,
    );
    let active =
        active_panel_content_bounds(&app.global::<DockManager>().unwrap().graph, &dock_layout);
    let expected = active
        .get(&panel)
        .copied()
        .expect("expected active panel bounds");

    let _ = ui.layout(&mut app, &mut services, dock_space, bounds.size, 1.0);

    let node = app
        .global::<DockPanelContentService>()
        .and_then(|svc| svc.get(window, &panel))
        .expect("expected panel node to be bound");

    let laid_out = ui
        .debug_node_bounds(node)
        .expect("expected panel node bounds");
    assert!((laid_out.origin.x.0 - expected.origin.x.0).abs() < 0.01);
    assert!((laid_out.origin.y.0 - expected.origin.y.0).abs() < 0.01);
    assert!((laid_out.size.width.0 - expected.size.width.0).abs() < 0.01);
    assert!((laid_out.size.height.0 - expected.size.height.0).abs() < 0.01);
}

#[test]
fn dock_space_installs_internal_drag_route_anchor() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    let dock_space = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(dock_space);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut services = FakeTextService;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(120.0)),
    );

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let route = fret_ui::internal_drag::route(&app, window, DRAG_KIND_DOCK_PANEL);

    assert_eq!(
        route,
        Some(dock_space),
        "expected DockSpace to install an internal drag route anchor during paint"
    );
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
            pointer_id: fret_core::PointerId(0),
        }),
    );

    let hover = app.global::<DockManager>().and_then(|d| d.hover.clone());
    assert!(hover.is_none(), "dock hover should be cleared on drop");
}

#[test]
fn dock_space_kicks_paint_cache_on_drag_transition_for_cache_root() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);
    ui.set_view_cache_enabled(true);
    ui.set_paint_cache_enabled(true);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_node_view_cache_flags(root, true, false, false);
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
    });

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);

    // Frame 0: establish a paint cache entry while no drag is active.
    app.advance_frame();
    ui.layout(&mut app, &mut text, root, size, 1.0);
    let mut scene = Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);
    let effects = app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::RequestAnimationFrame(_))),
        "expected no animation-frame requests when no drag is active"
    );

    // Start a cross-window dock drag between frames, without dispatching any events to the dock.
    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.hierarchy"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
        drag.position = Point::new(Px(48.0), Px(22.0));
    }

    // Frame 1: prepaint should kick the paint cache so `DockSpace::paint()` runs and can
    // establish the animation-frame loop.
    app.advance_frame();
    ui.layout(&mut app, &mut text, root, size, 1.0);
    let mut scene = Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);

    let effects = app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == window)),
        "expected DockSpace to request animation frames during a dock drag"
    );
    assert_eq!(
        ui.debug_stats().paint_cache_hits,
        0,
        "expected DockSpace paint to run (not replay) on drag transition"
    );
}

#[test]
fn dock_drag_suppresses_viewport_hover_and_wheel_forwarding() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    harness
        .app
        .set_global(fret_runtime::WindowInteractionDiagnosticsStore::default());

    harness.app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        harness.window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.viewport"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = harness.app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
    }

    // Ensure the dock interaction state is publishable to diagnostics (so suppression is
    // debuggable without relying on logs).
    harness.layout();
    let _ = harness.app.take_effects();

    let position = harness.viewport_point();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Wheel {
            position,
            delta: Point::new(Px(0.0), Px(12.0)),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = harness.app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "dock drag must suppress viewport hover/wheel forwarding (ADR 0072), got: {effects:?}",
    );

    let dock = harness
        .app
        .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
        .and_then(|store| store.docking_for_window(harness.window, harness.app.frame_id()))
        .expect("expected docking interaction diagnostics to be published for the window/frame");
    assert!(
        dock.dock_drag.is_some(),
        "expected dock drag to be recorded as the suppression reason, got: {dock:?}"
    );
}

#[test]
fn pointer_occlusion_blocks_viewport_hover_and_down_but_allows_wheel_forwarding() {
    struct HitTestTransparent;

    impl<H: UiHost> Widget<H> for HitTestTransparent {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut harness = DockViewportHarness::new();
    harness.layout();

    // Install a window-level pointer occlusion layer (Radix `disableOutsidePointerEvents`-like).
    let overlay_root = harness.ui.create_node_retained(HitTestTransparent);
    let overlay_layer = harness.ui.push_overlay_root_ex(overlay_root, false, true);
    harness.ui.set_layer_pointer_occlusion(
        overlay_layer,
        fret_ui::tree::PointerOcclusion::BlockMouseExceptScroll,
    );
    harness.layout();
    let _ = harness.app.take_effects();

    let position = harness.viewport_point();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let effects = harness.app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "expected pointer occlusion to suppress viewport hover forwarding, got: {effects:?}",
    );

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let effects = harness.app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "expected pointer occlusion to suppress viewport capture start, got: {effects:?}",
    );
    assert_eq!(
        harness.ui.captured_for(fret_core::PointerId(0)),
        None,
        "expected pointer occlusion to prevent viewport capture from requesting pointer capture"
    );

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Wheel {
            position,
            delta: Point::new(Px(0.0), Px(12.0)),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let effects = harness.app.take_effects();
    let Some(Effect::ViewportInput(input)) = effects
        .iter()
        .find(|e| matches!(e, Effect::ViewportInput(_)))
    else {
        panic!(
            "expected wheel forwarding to still emit a ViewportInput under pointer occlusion, got: {effects:?}"
        );
    };
    assert!(
        matches!(input.kind, fret_core::ViewportInputKind::Wheel { .. }),
        "expected wheel forwarding to remain active under BlockMouseExceptScroll, got: {input:?}",
    );
}

#[test]
fn foreign_pointer_capture_suppresses_viewport_capture_start() {
    struct CaptureOverlay;

    impl<H: UiHost> Widget<H> for CaptureOverlay {
        fn hit_test(&self, _bounds: Rect, position: Point) -> bool {
            position.x.0 >= 0.0
                && position.y.0 >= 0.0
                && position.x.0 <= 20.0
                && position.y.0 <= 20.0
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(fret_core::PointerEvent::Down { .. })) {
                cx.capture_pointer(cx.node);
                cx.stop_propagation();
            }
        }
    }

    let mut harness = DockViewportHarness::new();
    harness.layout();

    let overlay_root = harness.ui.create_node_retained(CaptureOverlay);
    let _overlay_layer = harness.ui.push_overlay_root_ex(overlay_root, false, true);
    harness.layout();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(2.0), Px(2.0)),
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        harness.ui.captured_for(fret_core::PointerId(1)),
        Some(overlay_root),
        "expected overlay to capture pointer 1"
    );

    // Advance a frame so docking can observe the runtime arbitration snapshot.
    harness.layout();
    let _ = harness.app.take_effects();

    let position = harness.viewport_point();
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let effects = harness.app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "expected foreign pointer capture to suppress viewport capture start, got: {effects:?}",
    );
    assert_eq!(
        harness.ui.captured_for(fret_core::PointerId(0)),
        None,
        "expected foreign pointer capture to prevent viewport capture from requesting pointer capture"
    );
}

#[test]
fn pending_dock_drag_suppresses_viewport_hover_and_wheel_forwarding() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    let tab_pos = harness.tab_point(0);

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: tab_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    assert!(
        harness.app.drag(fret_core::PointerId(0)).is_none(),
        "pending tab press should not start a cross-window drag session yet",
    );

    let position = harness.viewport_point();
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Wheel {
            position,
            delta: Point::new(Px(0.0), Px(12.0)),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = harness.app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "pending dock drag must suppress viewport hover/wheel forwarding (ADR 0072), got: {effects:?}",
    );

    let activate_pos = Point::new(Px(tab_pos.x.0 + 20.0), tab_pos.y);
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: activate_pos,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let drag = harness
        .app
        .drag(fret_core::PointerId(0))
        .and_then(|d| d.payload::<DockPanelDragPayload>().map(|_| d))
        .expect("expected pending dock drag to create a DragSession after activation");
    assert!(
        drag.dragging,
        "expected drag session to start in dragging state"
    );
}

#[test]
fn pending_dock_drag_does_not_start_drag_session_on_pointer_up_before_activation() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    let tab_pos = harness.tab_point(0);
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: tab_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position: tab_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    assert!(
        harness.app.drag(fret_core::PointerId(0)).is_none(),
        "pending dock drag must not create a drag session if released before activation",
    );

    // After releasing, viewport hover forwarding should resume.
    let position = harness.viewport_point();
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let effects = harness.app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "expected viewport hover forwarding after pending drag is released, got: {effects:?}",
    );
}

#[test]
fn pending_dock_drag_clears_on_pointer_cancel() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    let tab_pos = harness.tab_point(0);
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: tab_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::PointerCancel(fret_core::PointerCancelEvent {
            pointer_id: fret_core::PointerId(0),
            position: Some(tab_pos),
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            reason: fret_core::PointerCancelReason::LeftWindow,
        }),
    );
    let _ = harness.app.take_effects();

    assert!(
        harness.app.drag(fret_core::PointerId(0)).is_none(),
        "pending dock drag must not create a drag session on cancel",
    );

    let position = harness.viewport_point();
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let effects = harness.app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "expected viewport hover forwarding after pending drag cancel, got: {effects:?}",
    );
}

#[test]
fn pending_dock_drag_arbitration_is_pointer_keyed() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    let tab_pos = harness.tab_point(0);
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: tab_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    let position = harness.viewport_point();
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let effects = harness.app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "pending dock drag for one pointer must not suppress viewport hover for other pointers, got: {effects:?}",
    );
}

#[test]
fn docking_tab_drag_threshold_is_configurable_via_settings() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    harness
        .app
        .set_global(fret_runtime::DockingInteractionSettings {
            tab_drag_threshold: Px(1000.0),
            ..Default::default()
        });

    let tab_pos = harness.tab_point(0);
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: tab_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    let move_pos = Point::new(Px(tab_pos.x.0 + 40.0), tab_pos.y);
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: move_pos,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    assert!(
        harness.app.drag(fret_core::PointerId(0)).is_none(),
        "expected large threshold to prevent activation",
    );

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position: move_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    harness
        .app
        .set_global(fret_runtime::DockingInteractionSettings {
            tab_drag_threshold: Px(0.0),
            ..Default::default()
        });

    let tab_pos = harness.tab_point(0);
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: tab_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: tab_pos,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let drag = harness
        .app
        .drag(fret_core::PointerId(0))
        .and_then(|d| d.payload::<DockPanelDragPayload>().map(|_| d));
    assert!(
        drag.is_some_and(|d| d.dragging),
        "expected zero threshold to activate immediately on first move",
    );
}

#[test]
fn dock_drag_latches_dock_preview_policy_on_activation() {
    let settings = fret_runtime::DockingInteractionSettings {
        tab_drag_threshold: Px(0.0),
        drag_inversion: fret_runtime::DockDragInversionSettings {
            modifier: fret_runtime::DockDragInversionModifier::Ctrl,
            policy: fret_runtime::DockDragInversionPolicy::DockOnlyWhenModifier,
        },
        ..Default::default()
    };

    // Case 1: drag starts without modifier, then modifier changes during drag.
    // The "dock previews enabled" flag must be latched at activation, not recomputed per event.
    {
        let mut harness = DockViewportHarness::new();
        harness.layout();
        harness.app.set_global(settings);

        let tab_pos = harness.tab_point(0);
        harness.ui.dispatch_event(
            &mut harness.app,
            &mut harness.text,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: tab_pos,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let _ = harness.app.take_effects();

        harness.ui.dispatch_event(
            &mut harness.app,
            &mut harness.text,
            &Event::Pointer(fret_core::PointerEvent::Move {
                position: tab_pos,
                buttons: fret_core::MouseButtons {
                    left: true,
                    ..Default::default()
                },
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let _ = harness.app.take_effects();

        let dock_previews_enabled = harness
            .app
            .drag(fret_core::PointerId(0))
            .and_then(|d| {
                d.payload::<DockPanelDragPayload>()
                    .map(|p| p.dock_previews_enabled)
            })
            .expect("expected an active dock drag session");
        assert!(
            !dock_previews_enabled,
            "expected docking previews to be disabled without modifier"
        );

        let position = Point::new(Px(400.0), Px(300.0));
        harness.ui.dispatch_event(
            &mut harness.app,
            &mut harness.text,
            &Event::InternalDrag(InternalDragEvent {
                position,
                kind: InternalDragKind::Over,
                modifiers: Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                pointer_id: fret_core::PointerId(0),
            }),
        );

        let hover = harness
            .app
            .global::<DockManager>()
            .and_then(|d| d.hover.clone());
        assert!(
            matches!(hover, Some(DockDropTarget::Float { window }) if window == harness.window),
            "expected latched preview state to keep hover as Float even when modifier changes, got: {hover:?}",
        );
    }

    // Case 2: drag starts with modifier, then modifier is released. Must remain dock-enabled.
    {
        let mut harness = DockViewportHarness::new();
        harness.layout();
        harness.app.set_global(settings);

        let tab_pos = harness.tab_point(0);
        harness.ui.dispatch_event(
            &mut harness.app,
            &mut harness.text,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: tab_pos,
                button: fret_core::MouseButton::Left,
                modifiers: Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let _ = harness.app.take_effects();

        harness.ui.dispatch_event(
            &mut harness.app,
            &mut harness.text,
            &Event::Pointer(fret_core::PointerEvent::Move {
                position: tab_pos,
                buttons: fret_core::MouseButtons {
                    left: true,
                    ..Default::default()
                },
                modifiers: Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let _ = harness.app.take_effects();

        let dock_previews_enabled = harness
            .app
            .drag(fret_core::PointerId(0))
            .and_then(|d| {
                d.payload::<DockPanelDragPayload>()
                    .map(|p| p.dock_previews_enabled)
            })
            .expect("expected an active dock drag session");
        assert!(
            dock_previews_enabled,
            "expected docking previews to be enabled with modifier"
        );

        let position = Point::new(Px(400.0), Px(300.0));
        harness.ui.dispatch_event(
            &mut harness.app,
            &mut harness.text,
            &Event::InternalDrag(InternalDragEvent {
                position,
                kind: InternalDragKind::Over,
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
            }),
        );

        let hover = harness
            .app
            .global::<DockManager>()
            .and_then(|d| d.hover.clone());
        assert!(
            matches!(hover, Some(DockDropTarget::Dock(_))),
            "expected latched preview state to keep hover as Dock even when modifier is released, got: {hover:?}",
        );
    }
}

#[test]
fn dock_drag_requests_animation_frames_while_dragging() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    harness.app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        harness.window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.viewport"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = harness.app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
    }
    let _ = harness.app.take_effects();

    harness.layout();

    let effects = harness.app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == harness.window)),
        "expected dock drag to request animation frames, got: {effects:?}",
    );
}

#[test]
fn split_viewports_forward_input_to_captured_viewport() {
    let mut harness = DockSplitViewportHarness::new();
    harness.layout();

    let down_pos = harness.viewport_point(harness.target_left);
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = harness.app.take_effects();
    let Some(Effect::ViewportInput(input)) = effects
        .iter()
        .find(|e| matches!(e, Effect::ViewportInput(_)))
    else {
        panic!("expected a ViewportInput effect on down, got: {effects:?}");
    };
    assert_eq!(
        input.target, harness.target_left,
        "expected pointer down to forward to the left viewport"
    );

    let move_pos = harness.viewport_point(harness.target_right);
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: move_pos,
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = harness.app.take_effects();
    let Some(Effect::ViewportInput(input)) = effects
        .iter()
        .find(|e| matches!(e, Effect::ViewportInput(_)))
    else {
        panic!("expected a ViewportInput effect on move, got: {effects:?}");
    };
    assert_eq!(
        input.target, harness.target_left,
        "expected viewport capture to keep forwarding to the captured viewport"
    );
    assert!(
        (0.0..=1.0).contains(&input.uv.0) && (0.0..=1.0).contains(&input.uv.1),
        "expected clamped uv during capture, got: {:?}",
        input.uv
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
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
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
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
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
fn viewport_overlay_hooks_can_implement_layout_api_only() {
    use fret_core::DrawOrder;

    #[derive(Debug)]
    struct LayoutOnlyHooks;

    impl DockViewportOverlayHooks for LayoutOnlyHooks {
        fn paint_with_layout(
            &self,
            _theme: fret_ui::ThemeSnapshot,
            _window: AppWindowId,
            _panel: &fret_core::PanelKey,
            _viewport: super::ViewportPanel,
            layout: super::DockViewportLayout,
            scene: &mut Scene,
        ) {
            scene.push(SceneOp::Quad {
                order: DrawOrder(9999),
                rect: layout.draw_rect,
                background: Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.35,
                },
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        }
    }

    let mut harness = DockViewportHarness::new();
    harness.app.with_global_mut(
        super::DockViewportOverlayHooksService::default,
        |svc, _app| {
            svc.set(Arc::new(LayoutOnlyHooks));
        },
    );

    let scene = harness.paint_scene();
    let layout = harness
        .app
        .global::<DockManager>()
        .and_then(|dock| dock.viewport_layout(harness.window, harness.target))
        .expect("expected viewport layout to be recorded during paint");

    assert!(
        scene.ops().iter().any(|op| match op {
            SceneOp::Quad { order, rect, .. } => {
                *order == DrawOrder(9999) && *rect == layout.draw_rect
            }
            _ => false,
        }),
        "expected overlay hook quad to be painted using layout.draw_rect"
    );
}

#[test]
fn viewport_capture_requests_animation_frames_while_active() {
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
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    harness.layout();

    let effects = harness.app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::RequestAnimationFrame(w) if *w == harness.window)),
        "expected viewport capture to request animation frames, got: {effects:?}",
    );
}

#[test]
fn viewport_capture_emits_pointer_cancel_and_releases_capture() {
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
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    assert_eq!(
        harness.ui.captured_for(fret_core::PointerId(0)),
        Some(harness.root),
        "expected viewport capture to request pointer capture on down"
    );

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::PointerCancel(fret_core::PointerCancelEvent {
            pointer_id: fret_core::PointerId(0),
            position: None,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            reason: fret_core::PointerCancelReason::LeftWindow,
        }),
    );

    assert_eq!(
        harness.ui.captured_for(fret_core::PointerId(0)),
        None,
        "expected pointer capture to be released on cancel",
    );

    let effects = harness.app.take_effects();
    let Some(Effect::ViewportInput(evt)) = effects
        .iter()
        .find(|e| matches!(e, Effect::ViewportInput(_)))
    else {
        panic!("expected viewport cancel input effect, got: {effects:?}");
    };
    assert!(
        matches!(evt.kind, fret_core::ViewportInputKind::PointerCancel { .. }),
        "expected ViewportInputKind::PointerCancel, got: {evt:?}",
    );
}

#[test]
fn dock_drag_suppresses_viewport_capture_start_for_other_pointer() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    harness.app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(7),
        DRAG_KIND_DOCK_PANEL,
        harness.window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.viewport"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            dock_previews_enabled: true,
        },
    );
    let _ = harness.app.take_effects();

    let down_pos = harness.viewport_point();
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = harness.app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "expected viewport capture not to start during dock drag, got: {effects:?}"
    );
    assert_eq!(
        harness.ui.captured_for(fret_core::PointerId(0)),
        None,
        "expected viewport capture not to request pointer capture during dock drag"
    );
}

#[test]
fn viewport_capture_suppresses_viewport_moves_for_other_pointers() {
    let mut harness = DockViewportHarness::new();
    harness.layout();

    let position = harness.viewport_point();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons: fret_core::MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = harness.app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::ViewportInput(_))),
        "viewport capture must suppress viewport moves for other pointers, got: {effects:?}",
    );
}

#[test]
fn viewport_capture_does_not_clear_on_other_pointer_up() {
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
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position: down_pos,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
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
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
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
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
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
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(24.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.hierarchy"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
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
            pointer_id: fret_core::PointerId(0),
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
fn dock_tab_drop_outside_window_does_not_request_tear_off_twice() {
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
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(24.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.hierarchy"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
    }

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let _ = ui.layout(&mut app, &mut text, root, size, 1.0);

    let outside = Point::new(Px(-32.0), Px(12.0));

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: outside,
            kind: InternalDragKind::Over,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: outside,
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );

    let effects = app.take_effects();
    let count = effects
        .iter()
        .filter(|e| {
            matches!(
                e,
                Effect::Dock(DockOp::RequestFloatPanelToNewWindow { panel, .. })
                    if *panel == PanelKey::new("core.hierarchy")
            )
        })
        .count();
    assert_eq!(
        count, 1,
        "expected at most one tear-off request for a single drag session"
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
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(24.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.hierarchy"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
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
            pointer_id: fret_core::PointerId(0),
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
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(24.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.hierarchy"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
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
            pointer_id: fret_core::PointerId(0),
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
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(24.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.hierarchy"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
    }

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position: Point::new(Px(-32.0), Px(12.0)),
            kind: InternalDragKind::Drop,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
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

    for (expected, hint_rect) in dock_hint_rects_with_font(rect, Px(13.0), false) {
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

#[test]
fn dock_outer_drop_rects_target_window_root_even_when_root_is_split() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut root_split: Option<DockNodeId> = None;
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
        root_split = Some(split);
        dock.graph.set_window_root(window, split);
        for (key, title) in [
            (PanelKey::new("core.left"), "Left"),
            (PanelKey::new("core.right"), "Right"),
        ] {
            dock.panels.insert(
                key,
                DockPanel {
                    title: title.to_string(),
                    color: Color::TRANSPARENT,
                    viewport: None,
                },
            );
        }
    });
    let root_split = root_split.expect("expected window root split");

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    ui.layout(&mut app, &mut text, root, size, 1.0);
    let mut scene = Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(0),
        DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(12.0), Px(12.0)),
        DockPanelDragPayload {
            panel: PanelKey::new("core.left"),
            grab_offset: Point::new(Px(0.0), Px(0.0)),
            start_tick: fret_runtime::TickId(0),
            tear_off_requested: false,
            dock_previews_enabled: true,
        },
    );
    if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
        drag.dragging = true;
    }

    let root_rect = bounds;
    let outer_left = dock_hint_rects_with_font(root_rect, Px(13.0), true)
        .into_iter()
        .find_map(|(zone, rect)| (zone == DropZone::Left).then_some(rect))
        .expect("expected outer left rect");
    let position = Point::new(
        Px(outer_left.origin.x.0 + outer_left.size.width.0 * 0.5),
        Px(outer_left.origin.y.0 + outer_left.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut text,
        &Event::InternalDrag(InternalDragEvent {
            position,
            kind: InternalDragKind::Over,
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
        }),
    );

    let hover = app.global::<DockManager>().and_then(|d| d.hover.clone());
    assert!(
        matches!(hover, Some(DockDropTarget::Dock(t)) if t.tabs == root_split && t.zone == DropZone::Left && t.outer),
        "expected outer docking to target window root split, got: {hover:?}",
    );
}

#[test]
fn dock_center_drop_overlay_excludes_tab_bar() {
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
        dock.hover = Some(DockDropTarget::Dock(HoverTarget {
            tabs,
            root: tabs,
            leaf_tabs: tabs,
            zone: DropZone::Center,
            insert_index: None,
            outer: false,
        }));
    });

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    ui.layout(&mut app, &mut text, root, size, 1.0);
    let mut scene = Scene::default();
    ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);

    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let (tab_bar, content) = split_tab_bar(dock_bounds);

    let has_tab_quad = scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::Quad { rect, .. } if *rect == tab_bar
        )
    });
    assert!(has_tab_quad, "expected a tab-bar overlay quad");

    let has_content_quad = scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::Quad { rect, .. } if *rect == content
        )
    });
    assert!(
        has_content_quad,
        "expected center drop overlay quad to cover content rect (excluding tab bar)"
    );
}

#[test]
fn dock_tab_bar_insert_index_respects_before_after_halves() {
    let window = AppWindowId::default();

    let mut dock = DockManager::default();
    let tabs = dock.graph.insert_node(DockNode::Tabs {
        tabs: vec![
            PanelKey::new("core.a"),
            PanelKey::new("core.b"),
            PanelKey::new("core.c"),
        ],
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

    let (tab_bar, _content) = split_tab_bar(rect);
    let scroll = Px(0.0);

    let tab_b = TabBarGeometry::fixed(tab_bar, 3).tab_rect(1, scroll);
    let y = Px(tab_b.origin.y.0 + tab_b.size.height.0 * 0.5);

    let left_half = Point::new(Px(tab_b.origin.x.0 + tab_b.size.width.0 * 0.25), y);
    let hit_left = hit_test_drop_target(&dock.graph, &layout, &tab_scroll, left_half)
        .expect("hit should resolve to a dock target");
    assert_eq!(hit_left.tabs, tabs);
    assert_eq!(hit_left.zone, DropZone::Center);
    assert_eq!(hit_left.insert_index, Some(1));

    let right_half = Point::new(Px(tab_b.origin.x.0 + tab_b.size.width.0 * 0.75), y);
    let hit_right = hit_test_drop_target(&dock.graph, &layout, &tab_scroll, right_half)
        .expect("hit should resolve to a dock target");
    assert_eq!(hit_right.tabs, tabs);
    assert_eq!(hit_right.zone, DropZone::Center);
    assert_eq!(hit_right.insert_index, Some(2));

    let far_right = Point::new(Px(tab_bar.origin.x.0 + tab_bar.size.width.0 - 1.0), y);
    let hit_end = hit_test_drop_target(&dock.graph, &layout, &tab_scroll, far_right)
        .expect("hit should resolve to a dock target");
    assert_eq!(hit_end.tabs, tabs);
    assert_eq!(hit_end.zone, DropZone::Center);
    assert_eq!(hit_end.insert_index, Some(3));
}

#[test]
fn dock_tab_drop_emits_insert_index_based_on_over_tab_halves() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(root);

    let panel_a = PanelKey::new("core.a");
    let panel_b = PanelKey::new("core.b");
    let panel_c = PanelKey::new("core.c");

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let tabs = app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_a.clone(), panel_b.clone(), panel_c.clone()],
            active: 0,
        });
        dock.graph.set_window_root(window, tabs);
        for panel in [&panel_a, &panel_b, &panel_c] {
            dock.panels.insert(
                panel.clone(),
                DockPanel {
                    title: "Panel".to_string(),
                    color: Color::TRANSPARENT,
                    viewport: None,
                },
            );
        }
        tabs
    });

    let mut text = FakeTextService;
    let size = Size::new(Px(800.0), Px(600.0));
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
    let _ = ui.layout(&mut app, &mut text, root, size, 1.0);

    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let (tab_bar, _content) = split_tab_bar(dock_bounds);
    let scroll = Px(0.0);

    let over_rect = TabBarGeometry::fixed(tab_bar, 3).tab_rect(1, scroll);
    let y = Px(over_rect.origin.y.0 + over_rect.size.height.0 * 0.5);

    let check_drop = |app: &mut TestHost,
                      ui: &mut UiTree<TestHost>,
                      position: Point,
                      expect: usize| {
        app.begin_cross_window_drag_with_kind(
            fret_core::PointerId(0),
            DRAG_KIND_DOCK_PANEL,
            window,
            Point::new(Px(24.0), Px(12.0)),
            DockPanelDragPayload {
                panel: panel_a.clone(),
                grab_offset: Point::new(Px(0.0), Px(0.0)),
                start_tick: fret_runtime::TickId(0),
                tear_off_requested: false,
                dock_previews_enabled: true,
            },
        );
        if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
            drag.dragging = true;
        }

        let mut services = FakeTextService;
        ui.dispatch_event(
            app,
            &mut services,
            &Event::InternalDrag(InternalDragEvent {
                position,
                kind: InternalDragKind::Over,
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
            }),
        );
        ui.dispatch_event(
            app,
            &mut services,
            &Event::InternalDrag(InternalDragEvent {
                position,
                kind: InternalDragKind::Drop,
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
            }),
        );

        let effects = app.take_effects();
        let moves: Vec<_> = effects
            .iter()
            .filter_map(|e| match e {
                Effect::Dock(DockOp::MovePanel {
                    panel,
                    target_tabs,
                    zone,
                    insert_index,
                    ..
                }) if panel == &panel_a && *target_tabs == tabs && *zone == DropZone::Center => {
                    Some(*insert_index)
                }
                _ => None,
            })
            .collect();
        assert_eq!(moves, vec![Some(expect)]);
    };

    let left_half = Point::new(Px(over_rect.origin.x.0 + over_rect.size.width.0 * 0.25), y);
    check_drop(&mut app, &mut ui, left_half, 1);

    let right_half = Point::new(Px(over_rect.origin.x.0 + over_rect.size.width.0 * 0.75), y);
    check_drop(&mut app, &mut ui, right_half, 2);
}

#[test]
fn dock_tab_drop_reorders_tabs_when_applying_move_panel() {
    fn run(position_in_tab_bar: Point) -> Vec<PanelKey> {
        let window = AppWindowId::default();

        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node_retained(DockSpace::new(window));
        ui.set_root(root);

        let panel_a = PanelKey::new("core.a");
        let panel_b = PanelKey::new("core.b");
        let panel_c = PanelKey::new("core.c");
        let panel_d = PanelKey::new("core.d");

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let tabs = app.with_global_mut(DockManager::default, |dock, _app| {
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![
                    panel_a.clone(),
                    panel_b.clone(),
                    panel_c.clone(),
                    panel_d.clone(),
                ],
                active: 0,
            });
            dock.graph.set_window_root(window, tabs);
            for panel in [&panel_a, &panel_b, &panel_c, &panel_d] {
                dock.panels.insert(
                    panel.clone(),
                    DockPanel {
                        title: "Panel".to_string(),
                        color: Color::TRANSPARENT,
                        viewport: None,
                    },
                );
            }
            tabs
        });

        let mut text = FakeTextService;
        let size = Size::new(Px(800.0), Px(600.0));
        let _ = ui.layout(&mut app, &mut text, root, size, 1.0);

        app.begin_cross_window_drag_with_kind(
            fret_core::PointerId(0),
            DRAG_KIND_DOCK_PANEL,
            window,
            Point::new(Px(24.0), Px(12.0)),
            DockPanelDragPayload {
                panel: panel_d.clone(),
                grab_offset: Point::new(Px(0.0), Px(0.0)),
                start_tick: fret_runtime::TickId(0),
                tear_off_requested: false,
                dock_previews_enabled: true,
            },
        );
        if let Some(drag) = app.drag_mut(fret_core::PointerId(0)) {
            drag.dragging = true;
        }

        ui.dispatch_event(
            &mut app,
            &mut text,
            &Event::InternalDrag(InternalDragEvent {
                position: position_in_tab_bar,
                kind: InternalDragKind::Over,
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut text,
            &Event::InternalDrag(InternalDragEvent {
                position: position_in_tab_bar,
                kind: InternalDragKind::Drop,
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(0),
            }),
        );

        let effects = app.take_effects();
        let op = effects
            .iter()
            .find_map(|e| match e {
                Effect::Dock(DockOp::MovePanel {
                    panel,
                    target_tabs,
                    zone,
                    insert_index,
                    ..
                }) if panel == &panel_d && *target_tabs == tabs && *zone == DropZone::Center => {
                    Some(DockOp::MovePanel {
                        source_window: window,
                        panel: panel.clone(),
                        target_window: window,
                        target_tabs: *target_tabs,
                        zone: *zone,
                        insert_index: *insert_index,
                    })
                }
                _ => None,
            })
            .expect("expected a MovePanel op for the drop");

        app.with_global_mut(DockManager::default, |dock, _app| {
            assert!(dock.graph.apply_op(&op));
            match dock.graph.node(tabs) {
                Some(DockNode::Tabs { tabs, .. }) => tabs.clone(),
                other => panic!("expected tabs node, got {other:?}"),
            }
        })
    }

    let rect = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let (_chrome, dock_bounds) = dock_space_regions(rect);
    let (tab_bar, _content) = split_tab_bar(dock_bounds);
    let scroll = Px(0.0);

    let over_rect = TabBarGeometry::fixed(tab_bar, 3).tab_rect(1, scroll);
    let y = Px(over_rect.origin.y.0 + over_rect.size.height.0 * 0.5);

    let left_half = Point::new(Px(over_rect.origin.x.0 + over_rect.size.width.0 * 0.25), y);
    assert_eq!(
        run(left_half),
        vec![
            PanelKey::new("core.a"),
            PanelKey::new("core.d"),
            PanelKey::new("core.b"),
            PanelKey::new("core.c"),
        ]
    );

    let right_half = Point::new(Px(over_rect.origin.x.0 + over_rect.size.width.0 * 0.75), y);
    assert_eq!(
        run(right_half),
        vec![
            PanelKey::new("core.a"),
            PanelKey::new("core.b"),
            PanelKey::new("core.d"),
            PanelKey::new("core.c"),
        ]
    );
}

#[test]
fn render_and_bind_panels_falls_back_to_placeholder_for_missing_ui() {
    let window = AppWindowId::default();

    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let dock_space = ui.create_node_retained(DockSpace::new(window));
    ui.set_root(dock_space);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let panel = PanelKey::new("core.missing");
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        dock.graph.set_window_root(window, tabs);
        dock.panels.insert(
            panel.clone(),
            DockPanel {
                title: "Missing".to_string(),
                color: Color::TRANSPARENT,
                viewport: None,
            },
        );
    });

    struct AlwaysMissingRegistry;
    impl DockPanelRegistry<TestHost> for AlwaysMissingRegistry {
        fn render_panel(
            &self,
            _ui: &mut UiTree<TestHost>,
            _app: &mut TestHost,
            _services: &mut dyn UiServices,
            _window: AppWindowId,
            _bounds: Rect,
            _panel: &PanelKey,
        ) -> Option<NodeId> {
            None
        }
    }

    app.with_global_mut(
        DockPanelRegistryService::<TestHost>::default,
        |svc, _app| {
            svc.set(Arc::new(AlwaysMissingRegistry));
        },
    );

    let mut text = FakeTextService;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(640.0), Px(480.0)),
    );
    render_and_bind_dock_panels(&mut ui, &mut app, &mut text, window, bounds, dock_space);

    let service = app
        .global::<DockPanelContentService>()
        .expect("DockPanelContentService should exist after render_and_bind_dock_panels");
    assert!(
        service.get(window, &panel).is_some(),
        "expected a placeholder node for a non-viewport panel with missing UI"
    );
}

#[test]
fn viewport_capture_suppresses_secondary_right_click_bubbling() {
    let mut harness = DockViewportPropagationHarness::new();
    harness.layout();

    let position = harness.viewport_point();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: fret_core::MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let _ = harness.app.take_effects();

    harness.reset_spy();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: fret_core::MouseButton::Right,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button: fret_core::MouseButton::Right,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let (downs, ups) = harness.spy_counts();
    assert_eq!(
        (downs, ups),
        (0, 0),
        "secondary right click must not bubble while viewport capture is active, got downs={downs} ups={ups}",
    );
}

#[test]
fn viewport_right_click_bubbles_when_not_dragging() {
    let mut harness = DockViewportPropagationHarness::new();
    harness.layout();
    harness.reset_spy();

    let position = harness.viewport_point();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: fret_core::MouseButton::Right,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button: fret_core::MouseButton::Right,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let (downs, ups) = harness.spy_counts();
    assert_eq!(
        (downs, ups),
        (0, 1),
        "right click without drag should bubble on release so context menus can trigger, got downs={downs} ups={ups}",
    );
}

#[test]
fn viewport_right_drag_suppresses_context_menu_bubbling_on_release() {
    let mut harness = DockViewportPropagationHarness::new();
    harness.layout();
    harness.reset_spy();

    let start = harness.viewport_point();
    let end = Point::new(Px(start.x.0 + 20.0), Px(start.y.0 + 20.0));

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: start,
            button: fret_core::MouseButton::Right,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Move {
            position: end,
            buttons: fret_core::MouseButtons {
                right: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    harness.reset_spy();

    harness.ui.dispatch_event(
        &mut harness.app,
        &mut harness.text,
        &Event::Pointer(fret_core::PointerEvent::Up {
            position: end,
            button: fret_core::MouseButton::Right,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let (downs, ups) = harness.spy_counts();
    assert_eq!(
        (downs, ups),
        (0, 0),
        "right-drag release must not bubble to avoid triggering context menus, got downs={downs} ups={ups}",
    );
}
