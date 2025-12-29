use fret_core::{
    Color, DockGraph, DockNode, DockNodeId, DockOp, DropZone, Edges, NodeId, PanelKey,
    RenderTargetId, Scene, SceneOp, SemanticsRole, TextBlobId, TextConstraints, TextMetrics,
    TextOverflow, TextStyle, TextWrap, ViewportFit, ViewportInputEvent, ViewportInputKind,
    ViewportMapping, WindowMetricsService,
    geometry::{Point, Px, Rect, Size},
};
use fret_runtime::{CommandId, DragKind, Effect, WindowRequest};
use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    sync::Arc,
};

use fret_ui::InternalDragRouteService;
use fret_ui::UiHost;
use fret_ui::retained_bridge::{
    CommandCx, EventCx, Invalidation, LayoutCx, PaintCx, ResizeHandle, SemanticsCx, Widget,
};

mod hit_test;
mod layout;
mod paint;
mod viewport;

use self::hit_test::*;
use self::layout::*;
use self::paint::*;
use self::viewport::*;

pub struct DockPanel {
    pub title: String,
    pub color: Color,
    pub viewport: Option<ViewportPanel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportPanel {
    pub target: RenderTargetId,
    pub target_px_size: (u32, u32),
    pub fit: ViewportFit,
    pub context_menu_enabled: bool,
}

/// App/editor-owned viewport overlays (gizmos, marquee, selection, etc.).
///
/// Docking UI is policy-heavy already, but viewport overlay *shapes* are editor/app-specific
/// (ADR 0027 / ADR 0049). This hook keeps docking focused on "viewport embedding" only.
pub trait DockViewportOverlayHooks: Send + Sync + 'static {
    fn paint(
        &self,
        theme: fret_ui::ThemeSnapshot,
        window: fret_core::AppWindowId,
        panel: &PanelKey,
        viewport: ViewportPanel,
        mapping: ViewportMapping,
        draw_rect: Rect,
        scene: &mut Scene,
    );
}

#[derive(Default)]
pub struct DockViewportOverlayHooksService {
    hooks: Option<Arc<dyn DockViewportOverlayHooks>>,
}

impl DockViewportOverlayHooksService {
    pub fn set(&mut self, hooks: Arc<dyn DockViewportOverlayHooks>) {
        self.hooks = Some(hooks);
    }

    pub fn clear(&mut self) {
        self.hooks = None;
    }

    pub fn hooks(&self) -> Option<Arc<dyn DockViewportOverlayHooks>> {
        self.hooks.clone()
    }
}

#[derive(Debug, Clone)]
struct DockPanelDragPayload {
    panel: PanelKey,
    grab_offset: Point,
}

#[derive(Debug, Clone, PartialEq)]
enum DockDropTarget {
    Dock(HoverTarget),
    Float { window: fret_core::AppWindowId },
}

pub struct DockManager {
    pub graph: DockGraph,
    pub panels: HashMap<PanelKey, DockPanel>,
    dock_space_nodes: HashMap<fret_core::AppWindowId, NodeId>,
    hover: Option<DockDropTarget>,
    viewport_content_rects: HashMap<(fret_core::AppWindowId, RenderTargetId), Rect>,
}

pub fn create_dock_space_node<H: UiHost>(
    ui: &mut fret_ui::UiTree<H>,
    window: fret_core::AppWindowId,
) -> NodeId {
    use fret_ui::retained_bridge::UiTreeRetainedExt as _;
    ui.create_node_retained(DockSpace::new(window))
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ActivatePanelOptions {
    pub focus: bool,
}

#[derive(Default)]
struct DockFocusRequestService {
    per_window: HashMap<fret_core::AppWindowId, PanelKey>,
}

impl DockFocusRequestService {
    fn request(&mut self, window: fret_core::AppWindowId, panel: PanelKey) {
        self.per_window.insert(window, panel);
    }

    fn take(&mut self, window: fret_core::AppWindowId) -> Option<PanelKey> {
        self.per_window.remove(&window)
    }
}

impl DockManager {
    pub fn activate_panel_tab_best_effort(
        &self,
        preferred_windows: impl IntoIterator<Item = fret_core::AppWindowId>,
        panel: &PanelKey,
    ) -> Option<(fret_core::AppWindowId, fret_core::DockOp)> {
        let mut preferred: Vec<fret_core::AppWindowId> = Vec::new();
        let mut seen: std::collections::HashSet<fret_core::AppWindowId> =
            std::collections::HashSet::new();
        for w in preferred_windows {
            if seen.insert(w) {
                preferred.push(w);
            }
        }

        for w in &preferred {
            if let Some((tabs, active)) = self.graph.find_panel_in_window(*w, panel) {
                return Some((*w, fret_core::DockOp::SetActiveTab { tabs, active }));
            }
        }

        for w in self.graph.windows() {
            if seen.contains(&w) {
                continue;
            }
            if let Some((tabs, active)) = self.graph.find_panel_in_window(w, panel) {
                return Some((w, fret_core::DockOp::SetActiveTab { tabs, active }));
            }
        }
        None
    }

    pub fn request_activate_panel<H: UiHost>(
        host: &mut H,
        sender: fret_core::AppWindowId,
        preferred_windows: impl IntoIterator<Item = fret_core::AppWindowId>,
        panel: PanelKey,
        options: ActivatePanelOptions,
    ) -> bool {
        let preferred: Vec<fret_core::AppWindowId> = preferred_windows.into_iter().collect();
        let Some((target_window, op)) = host
            .global::<DockManager>()
            .and_then(|dock| dock.activate_panel_tab_best_effort(preferred, &panel))
        else {
            return false;
        };

        host.push_effect(Effect::Dock(op));
        if options.focus {
            host.with_global_mut(DockFocusRequestService::default, |service, _host| {
                service.request(target_window, panel.clone());
            });
            host.push_effect(Effect::Command {
                window: Some(target_window),
                command: CommandId::from("dock.focus_requested_panel"),
            });
        }
        if target_window != sender {
            host.push_effect(Effect::Window(WindowRequest::Raise {
                window: target_window,
                sender: Some(sender),
            }));
        }
        true
    }
}

#[derive(Default)]
pub struct DockPanelContentService {
    per_window: HashMap<fret_core::AppWindowId, HashMap<PanelKey, NodeId>>,
}

impl DockPanelContentService {
    pub fn set(&mut self, window: fret_core::AppWindowId, panel: PanelKey, node: NodeId) {
        self.per_window
            .entry(window)
            .or_default()
            .insert(panel, node);
    }

    pub fn get(&self, window: fret_core::AppWindowId, panel: &PanelKey) -> Option<NodeId> {
        self.per_window
            .get(&window)
            .and_then(|m| m.get(panel))
            .copied()
    }

    pub fn panel_nodes(&self, window: fret_core::AppWindowId) -> Vec<(PanelKey, NodeId)> {
        self.per_window
            .get(&window)
            .map(|m| m.iter().map(|(k, v)| (k.clone(), *v)).collect())
            .unwrap_or_default()
    }
}

impl Default for DockManager {
    fn default() -> Self {
        Self {
            graph: DockGraph::new(),
            panels: HashMap::new(),
            dock_space_nodes: HashMap::new(),
            hover: None,
            viewport_content_rects: HashMap::new(),
        }
    }
}

impl DockManager {
    pub fn dock_space_node(&self, window: fret_core::AppWindowId) -> Option<NodeId> {
        self.dock_space_nodes.get(&window).copied()
    }

    pub fn register_dock_space_node(&mut self, window: fret_core::AppWindowId, node: NodeId) {
        self.dock_space_nodes.insert(window, node);
    }

    pub fn insert_panel(&mut self, key: PanelKey, panel: DockPanel) {
        self.panels.insert(key, panel);
    }

    pub fn ensure_panel(&mut self, key: &PanelKey, make: impl FnOnce() -> DockPanel) {
        self.panels.entry(key.clone()).or_insert_with(make);
    }

    pub fn panel(&self, key: &PanelKey) -> Option<&DockPanel> {
        self.panels.get(key)
    }

    pub fn viewport_content_rect(
        &self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
    ) -> Option<Rect> {
        self.viewport_content_rects.get(&(window, target)).copied()
    }

    pub fn clear_viewport_layout_for_window(&mut self, window: fret_core::AppWindowId) {
        self.viewport_content_rects.retain(|(w, _), _| *w != window);
    }

    pub fn set_viewport_content_rect(
        &mut self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
        rect: Rect,
    ) {
        self.viewport_content_rects.insert((window, target), rect);
    }

    pub fn update_viewport_target_px_size(
        &mut self,
        target: RenderTargetId,
        target_px_size: (u32, u32),
    ) {
        for panel in self.panels.values_mut() {
            let Some(mut vp) = panel.viewport else {
                continue;
            };
            if vp.target != target {
                continue;
            }
            vp.target_px_size = target_px_size;
            panel.viewport = Some(vp);
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct DividerDragState {
    split: DockNodeId,
    axis: fret_core::Axis,
    bounds: Rect,
    fraction: f32,
    grab_offset: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HoverTarget {
    tabs: DockNodeId,
    zone: DropZone,
    insert_index: Option<usize>,
}

const DOCK_TAB_H: Px = Px(28.0);
const DOCK_TAB_W: Px = Px(120.0);
const DOCK_TAB_CLOSE_SIZE: Px = Px(16.0);
const DOCK_TAB_CLOSE_GAP: Px = Px(6.0);
const DOCK_SPLIT_HANDLE_HIT_THICKNESS: Px = Px(6.0);
const DOCK_SPLIT_HANDLE_GAP: Px = Px(0.0);

#[derive(Debug, Clone, Copy)]
struct PreparedTabTitle {
    blob: TextBlobId,
    metrics: TextMetrics,
    title_hash: u64,
}

pub struct DockSpace {
    pub window: fret_core::AppWindowId,
    last_bounds: Rect,
    divider_drag: Option<DividerDragState>,
    panel_content: HashMap<PanelKey, NodeId>,
    panel_last_sizes: HashMap<PanelKey, Size>,
    viewport_capture: Option<ViewportCaptureState>,
    tear_off_in_flight: Option<DockTearOffKey>,
    tab_titles: HashMap<PanelKey, PreparedTabTitle>,
    empty_state: Option<PreparedTabTitle>,
    hovered_tab: Option<(DockNodeId, usize)>,
    hovered_tab_close: bool,
    pressed_tab_close: Option<(DockNodeId, usize, PanelKey)>,
    tab_scroll: HashMap<DockNodeId, Px>,
    tab_close_glyph: Option<PreparedTabTitle>,
    tab_text_style: TextStyle,
    tab_close_style: TextStyle,
    empty_state_style: TextStyle,
    last_empty_state_scale_factor: Option<f32>,
    last_empty_state_theme_revision: Option<u64>,
    last_tab_text_scale_factor: Option<f32>,
    last_theme_revision: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
struct DockTearOffKey {
    source_window: fret_core::AppWindowId,
    start: Point,
    panel: PanelKey,
}

impl DockSpace {
    pub fn new(window: fret_core::AppWindowId) -> Self {
        Self {
            window,
            last_bounds: Rect::default(),
            divider_drag: None,
            panel_content: HashMap::new(),
            panel_last_sizes: HashMap::new(),
            viewport_capture: None,
            tear_off_in_flight: None,
            tab_titles: HashMap::new(),
            empty_state: None,
            hovered_tab: None,
            hovered_tab_close: false,
            pressed_tab_close: None,
            tab_scroll: HashMap::new(),
            tab_close_glyph: None,
            tab_text_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
                ..Default::default()
            },
            tab_close_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
                ..Default::default()
            },
            empty_state_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
                ..Default::default()
            },
            last_empty_state_scale_factor: None,
            last_empty_state_theme_revision: None,
            last_tab_text_scale_factor: None,
            last_theme_revision: None,
        }
    }

    pub fn with_panel_content(mut self, panel: PanelKey, root: NodeId) -> Self {
        self.panel_content.insert(panel, root);
        self
    }

    fn panel_nodes<H: UiHost>(&self, app: &H) -> HashMap<PanelKey, NodeId> {
        let mut out: HashMap<PanelKey, NodeId> = HashMap::new();
        if let Some(service) = app.global::<DockPanelContentService>() {
            for (panel, node) in service.panel_nodes(self.window) {
                out.insert(panel, node);
            }
        }
        out.extend(self.panel_content.iter().map(|(k, v)| (k.clone(), *v)));
        out
    }

    fn rebuild_tab_titles(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        theme: fret_ui::ThemeSnapshot,
        scale_factor: f32,
        dock: &DockManager,
        layout: &std::collections::HashMap<DockNodeId, Rect>,
    ) {
        self.tab_text_style.size = theme.metrics.font_size;
        self.tab_close_style.size = theme.metrics.font_size;
        self.empty_state_style.size = theme.metrics.font_size;

        let mut visible_set: HashSet<PanelKey> = HashSet::new();
        for &node_id in layout.keys() {
            let Some(DockNode::Tabs { tabs, .. }) = dock.graph.node(node_id) else {
                continue;
            };
            for panel in tabs {
                visible_set.insert(panel.clone());
            }
        }

        let same_tabs = visible_set.len() == self.tab_titles.len()
            && visible_set.iter().all(|p| self.tab_titles.contains_key(p));

        let hash_title = |s: &str| -> u64 {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            s.hash(&mut hasher);
            hasher.finish()
        };

        if same_tabs
            && self.last_theme_revision == Some(theme.revision)
            && self.last_tab_text_scale_factor == Some(scale_factor)
        {
            let titles_unchanged = visible_set.iter().all(|panel| {
                let title = dock
                    .panel(panel)
                    .map(|p| p.title.as_str())
                    .unwrap_or(panel.kind.0.as_str());
                let hash = hash_title(title);
                self.tab_titles
                    .get(panel)
                    .is_some_and(|t| t.title_hash == hash)
            });
            if titles_unchanged {
                return;
            }
        }
        self.last_theme_revision = Some(theme.revision);
        self.last_tab_text_scale_factor = Some(scale_factor);

        for (_, title) in self.tab_titles.drain() {
            services.text().release(title.blob);
        }
        if let Some(glyph) = self.tab_close_glyph.take() {
            services.text().release(glyph.blob);
        }

        let pad_x = theme.metrics.padding_md;
        let reserve = Px(DOCK_TAB_CLOSE_SIZE.0 + DOCK_TAB_CLOSE_GAP.0);
        let inner_max_w = Px((DOCK_TAB_W.0 - pad_x.0 * 2.0 - reserve.0).max(0.0));
        let constraints = TextConstraints {
            max_width: Some(inner_max_w),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor,
        };

        let (close_blob, close_metrics) = services.text().prepare(
            "×",
            self.tab_close_style,
            TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor,
            },
        );
        self.tab_close_glyph = Some(PreparedTabTitle {
            blob: close_blob,
            metrics: close_metrics,
            title_hash: 0,
        });

        for panel in visible_set {
            let title = dock
                .panel(&panel)
                .map(|p| p.title.as_str())
                .unwrap_or(panel.kind.0.as_str());
            let title_hash = hash_title(title);
            let (blob, metrics) = services
                .text()
                .prepare(title, self.tab_text_style, constraints);
            self.tab_titles.insert(
                panel,
                PreparedTabTitle {
                    blob,
                    metrics,
                    title_hash,
                },
            );
        }
    }

    fn tab_scroll_for(&self, tabs: DockNodeId) -> Px {
        self.tab_scroll.get(&tabs).copied().unwrap_or(Px(0.0))
    }

    fn set_tab_scroll_for(&mut self, tabs: DockNodeId, scroll: Px) {
        if scroll.0 <= 0.0 {
            self.tab_scroll.remove(&tabs);
        } else {
            self.tab_scroll.insert(tabs, scroll);
        }
    }

    fn max_tab_scroll(tab_bar: Rect, tab_count: usize) -> Px {
        let total = DOCK_TAB_W.0 * tab_count as f32;
        Px((total - tab_bar.size.width.0).max(0.0))
    }

    fn clamp_and_ensure_active_visible(
        &mut self,
        tabs: DockNodeId,
        tab_bar: Rect,
        tab_count: usize,
        active: usize,
    ) {
        if tab_count == 0 {
            self.tab_scroll.remove(&tabs);
            return;
        }

        let max_scroll = Self::max_tab_scroll(tab_bar, tab_count);
        let mut scroll = self.tab_scroll_for(tabs);

        if max_scroll.0 <= 0.0 {
            self.tab_scroll.remove(&tabs);
            return;
        }

        scroll = Px(scroll.0.clamp(0.0, max_scroll.0));

        let tab_left = DOCK_TAB_W.0 * active as f32 - scroll.0;
        let tab_right = tab_left + DOCK_TAB_W.0;
        if tab_left < 0.0 {
            scroll = Px(DOCK_TAB_W.0 * active as f32);
        } else if tab_right > tab_bar.size.width.0 {
            scroll = Px(DOCK_TAB_W.0 * (active + 1) as f32 - tab_bar.size.width.0);
        }

        scroll = Px(scroll.0.clamp(0.0, max_scroll.0));
        self.set_tab_scroll_for(tabs, scroll);
    }

    fn rebuild_empty_state(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        theme: fret_ui::ThemeSnapshot,
        scale_factor: f32,
        max_width: Px,
    ) {
        self.empty_state_style.size = theme.metrics.font_size;
        if self.last_empty_state_theme_revision == Some(theme.revision)
            && self.last_empty_state_scale_factor == Some(scale_factor)
        {
            return;
        }
        self.last_empty_state_theme_revision = Some(theme.revision);
        self.last_empty_state_scale_factor = Some(scale_factor);

        if let Some(prev) = self.empty_state.take() {
            services.text().release(prev.blob);
        }

        let constraints = TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor,
        };
        let (blob, metrics) = services.text().prepare(
            "No panels in this window.\nUse File → Layout → Reset Layout.",
            self.empty_state_style,
            constraints,
        );
        self.empty_state = Some(PreparedTabTitle {
            blob,
            metrics,
            title_hash: 0,
        });
    }

    fn paint_empty_state<H: UiHost>(&mut self, cx: &mut PaintCx<'_, H>) {
        let theme = cx.theme().snapshot();
        cx.scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(0),
            rect: cx.bounds,
            background: theme.colors.panel_background,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });

        let pad = theme.metrics.padding_md.0.max(0.0);
        let max_w = Px((cx.bounds.size.width.0 - pad * 2.0).max(0.0));
        self.rebuild_empty_state(cx.services, theme, cx.scale_factor, max_w);

        let Some(text) = self.empty_state else {
            return;
        };

        let x = (cx.bounds.origin.x.0 + (cx.bounds.size.width.0 - text.metrics.size.width.0) * 0.5)
            .max(cx.bounds.origin.x.0 + pad);
        let inner_y =
            cx.bounds.origin.y.0 + (cx.bounds.size.height.0 - text.metrics.size.height.0) * 0.5;
        let y = inner_y + text.metrics.baseline.0;

        cx.scene.push(SceneOp::Text {
            order: fret_core::DrawOrder(1),
            origin: Point::new(Px(x), Px(y)),
            text: text.blob,
            color: theme.colors.text_muted,
        });
    }
}

impl<H: UiHost> Widget<H> for DockSpace {
    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Panel);
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &fret_core::Event) {
        let theme = cx.theme().snapshot();

        let mut pending_effects: Vec<Effect> = Vec::new();
        let mut pending_redraws: Vec<fret_core::AppWindowId> = Vec::new();
        let mut invalidate_paint = false;
        let mut invalidate_layout = false;
        let mut request_focus: Option<NodeId> = None;
        let mut request_focus_panel: Option<PanelKey> = None;
        let mut request_pointer_capture: Option<Option<NodeId>> = None;
        let mut request_cursor: Option<fret_core::CursorIcon> = None;

        #[derive(Clone)]
        struct DockDragSnapshot {
            source_window: fret_core::AppWindowId,
            start: Point,
            dragging: bool,
            panel: PanelKey,
            grab_offset: Point,
        }

        fn is_outside_bounds_with_margin(bounds: Rect, position: Point, margin: Px) -> bool {
            position.x.0 < bounds.origin.x.0 - margin.0
                || position.y.0 < bounds.origin.y.0 - margin.0
                || position.x.0 > bounds.origin.x.0 + bounds.size.width.0 + margin.0
                || position.y.0 > bounds.origin.y.0 + bounds.size.height.0 + margin.0
        }

        let allow_viewport_hover = cx.app.drag().is_none_or(|d| !d.dragging);
        let dock_drag = cx.app.drag().and_then(|d| {
            d.payload::<DockPanelDragPayload>()
                .map(|p| DockDragSnapshot {
                    source_window: d.source_window,
                    start: d.start,
                    dragging: d.dragging,
                    panel: p.panel.clone(),
                    grab_offset: p.grab_offset,
                })
        });
        let window_bounds = cx
            .app
            .global::<WindowMetricsService>()
            .and_then(|svc| svc.inner_bounds(self.window))
            .unwrap_or(self.last_bounds);
        let (_chrome, dock_bounds) = dock_space_regions(self.last_bounds);

        match dock_drag.as_ref() {
            Some(drag) => {
                let key = DockTearOffKey {
                    source_window: drag.source_window,
                    start: drag.start,
                    panel: drag.panel.clone(),
                };
                if self.tear_off_in_flight.as_ref() != Some(&key) {
                    self.tear_off_in_flight = None;
                }
            }
            None => self.tear_off_in_flight = None,
        }

        let mut begin_drag: Option<(Point, PanelKey, Point)> = None;
        let mut update_drag: Option<(Point, bool)> = None;
        let mut end_dock_drag = false;

        {
            cx.app
                .with_global_mut(InternalDragRouteService::default, |routes, _app| {
                    routes.set(self.window, DragKind::DockPanel, cx.node);
                });
            let Some(dock) = cx.app.global_mut::<DockManager>() else {
                return;
            };
            dock.register_dock_space_node(self.window, cx.node);
            let Some(root) = dock.graph.window_root(self.window) else {
                return;
            };

            match event {
                fret_core::Event::Pointer(p) => match p {
                    fret_core::PointerEvent::Down {
                        position,
                        button,
                        modifiers,
                    } => {
                        let (_chrome, dock_bounds) = dock_space_regions(self.last_bounds);
                        let layout = compute_layout_map(&dock.graph, root, dock_bounds);
                        let mut handled = false;
                        if *button == fret_core::MouseButton::Left {
                            if let Some(handle) =
                                hit_test_split_handle(&dock.graph, &layout, *position)
                            {
                                self.divider_drag = Some(handle);
                                request_pointer_capture = Some(Some(cx.node));
                                request_cursor = Some(match handle.axis {
                                    fret_core::Axis::Horizontal => fret_core::CursorIcon::ColResize,
                                    fret_core::Axis::Vertical => fret_core::CursorIcon::RowResize,
                                });
                                invalidate_paint = true;
                                pending_redraws.push(self.window);
                                handled = true;
                            }
                            if !handled
                                && let Some((tabs_node, tab_index, panel_key, close)) = hit_test_tab(
                                    &dock.graph,
                                    &layout,
                                    &self.tab_scroll,
                                    theme,
                                    *position,
                                )
                            {
                                if close {
                                    self.pressed_tab_close =
                                        Some((tabs_node, tab_index, panel_key.clone()));
                                    request_pointer_capture = Some(Some(cx.node));
                                    dock.hover = None;
                                    invalidate_paint = true;
                                    pending_redraws.push(self.window);
                                    handled = true;
                                } else {
                                    pending_effects.push(Effect::Dock(DockOp::SetActiveTab {
                                        tabs: tabs_node,
                                        active: tab_index,
                                    }));
                                    request_focus_panel = Some(panel_key.clone());
                                    invalidate_layout = true;
                                    // For tear-off, we want the tab itself to stay under the
                                    // cursor after it becomes index 0 in its own floating window
                                    // (ImGui-style). Our `DockFloating` windows render only a
                                    // `DockSpace` starting at (0,0), so the correct anchor is the
                                    // *tab-local* grab offset (not the window-local cursor pos).
                                    let tab_rect = layout
                                        .get(&tabs_node)
                                        .copied()
                                        .map(split_tab_bar)
                                        .map(|(bar, _content)| {
                                            tab_rect_for_index(
                                                bar,
                                                tab_index,
                                                self.tab_scroll_for(tabs_node),
                                            )
                                        })
                                        .unwrap_or_else(|| Rect::new(*position, Size::default()));
                                    let tab_local = Point::new(
                                        Px((position.x.0 - tab_rect.origin.x.0).max(0.0)),
                                        Px((position.y.0 - tab_rect.origin.y.0).max(0.0)),
                                    );
                                    begin_drag = Some((*position, panel_key, tab_local));
                                    dock.hover = None;
                                    invalidate_paint = true;
                                    handled = true;
                                }
                            }
                        }

                        if !handled
                            && *button == fret_core::MouseButton::Right
                            && let Some((tabs_node, tab_index, panel_key, _close)) = hit_test_tab(
                                &dock.graph,
                                &layout,
                                &self.tab_scroll,
                                theme,
                                *position,
                            )
                        {
                            pending_effects.push(Effect::Dock(DockOp::SetActiveTab {
                                tabs: tabs_node,
                                active: tab_index,
                            }));
                            request_focus_panel = Some(panel_key.clone());
                            invalidate_layout = true;
                            dock.hover = None;
                            invalidate_paint = true;
                            handled = true;
                        }

                        if !handled
                            && let Some(hit) = hit_test_active_viewport_panel(
                                &dock.graph,
                                &dock.panels,
                                &layout,
                                *position,
                            )
                            && (*button == fret_core::MouseButton::Left
                                || *button == fret_core::MouseButton::Right
                                || *button == fret_core::MouseButton::Middle)
                        {
                            if let Some(e) = viewport_input_from_hit(
                                self.window,
                                hit.clone(),
                                *position,
                                ViewportInputKind::PointerDown {
                                    button: *button,
                                    modifiers: *modifiers,
                                },
                            ) {
                                pending_effects.push(Effect::ViewportInput(e));
                                pending_redraws.push(self.window);
                            }

                            self.viewport_capture = Some(ViewportCaptureState {
                                hit,
                                button: *button,
                                start: *position,
                                moved: false,
                            });
                            request_pointer_capture = Some(Some(cx.node));
                        }
                    }
                    fret_core::PointerEvent::Move {
                        position,
                        buttons,
                        modifiers,
                    } => {
                        if self.viewport_capture.is_none()
                            && self.divider_drag.is_none()
                            && dock_drag.is_none()
                        {
                            let (_chrome, dock_bounds) = dock_space_regions(self.last_bounds);
                            let layout = compute_layout_map(&dock.graph, root, dock_bounds);
                            if let Some(handle) =
                                hit_test_split_handle(&dock.graph, &layout, *position)
                            {
                                request_cursor = Some(match handle.axis {
                                    fret_core::Axis::Horizontal => fret_core::CursorIcon::ColResize,
                                    fret_core::Axis::Vertical => fret_core::CursorIcon::RowResize,
                                });
                            }
                        }

                        let hovered = if self.viewport_capture.is_none()
                            && self.divider_drag.is_none()
                            && dock_drag.is_none()
                        {
                            let (_chrome, dock_bounds) = dock_space_regions(self.last_bounds);
                            let layout = compute_layout_map(&dock.graph, root, dock_bounds);
                            hit_test_tab(&dock.graph, &layout, &self.tab_scroll, theme, *position)
                                .map(|(node, idx, _panel, close)| (node, idx, close))
                        } else {
                            None
                        };
                        let next_tab = hovered.map(|(node, idx, _close)| (node, idx));
                        let next_close = hovered.map(|(_node, _idx, close)| close).unwrap_or(false);
                        if next_tab != self.hovered_tab || next_close != self.hovered_tab_close {
                            self.hovered_tab = next_tab;
                            self.hovered_tab_close = next_close;
                            invalidate_paint = true;
                            pending_redraws.push(self.window);
                        }

                        if let Some(mut divider) = self.divider_drag {
                            cx.requested_cursor = Some(match divider.axis {
                                fret_core::Axis::Horizontal => fret_core::CursorIcon::ColResize,
                                fret_core::Axis::Vertical => fret_core::CursorIcon::RowResize,
                            });
                            let (_chrome, dock_bounds) = dock_space_regions(self.last_bounds);
                            let layout = compute_layout_map(&dock.graph, root, dock_bounds);
                            if let Some((left, right)) =
                                split_children_two(&dock.graph, divider.split).and_then(|(a, b)| {
                                    Some((layout.get(&a).copied()?, layout.get(&b).copied()?))
                                })
                                && let Some(f0) = compute_split_fraction(
                                    divider.axis,
                                    divider.bounds,
                                    left,
                                    right,
                                    divider.grab_offset,
                                    *position,
                                )
                            {
                                dock.graph.update_split_two(divider.split, f0);
                                divider.fraction = f0;
                                self.divider_drag = Some(divider);
                                cx.invalidate(cx.node, Invalidation::Layout);
                                cx.invalidate(cx.node, Invalidation::Paint);
                            }
                            return;
                        }

                        if let Some(capture) = self.viewport_capture.as_mut() {
                            let hit = capture.hit.clone();
                            let e = viewport_input_from_hit_clamped(
                                self.window,
                                hit,
                                *position,
                                ViewportInputKind::PointerMove {
                                    buttons: *buttons,
                                    modifiers: *modifiers,
                                },
                            );
                            pending_effects.push(Effect::ViewportInput(e));
                            pending_redraws.push(self.window);
                        } else {
                            let (_chrome, dock_bounds) = dock_space_regions(self.last_bounds);
                            if allow_viewport_hover && dock_bounds.contains(*position) {
                                let layout = compute_layout_map(&dock.graph, root, dock_bounds);
                                let hit = hit_test_active_viewport_panel(
                                    &dock.graph,
                                    &dock.panels,
                                    &layout,
                                    *position,
                                );

                                if let Some(hit) = hit
                                    && let Some(e) = viewport_input_from_hit(
                                        self.window,
                                        hit,
                                        *position,
                                        ViewportInputKind::PointerMove {
                                            buttons: *buttons,
                                            modifiers: *modifiers,
                                        },
                                    )
                                {
                                    pending_effects.push(Effect::ViewportInput(e));
                                    pending_redraws.push(self.window);
                                }
                            }
                        }

                        // Dock/tab dragging is handled via `Event::InternalDrag` so it can work
                        // across windows without relying on pointer-event broadcasting.
                    }
                    fret_core::PointerEvent::Wheel {
                        position,
                        delta,
                        modifiers,
                    } => {
                        let bounds = self.last_bounds;
                        if !bounds.contains(*position) {
                            return;
                        }
                        let layout = compute_layout_map(&dock.graph, root, bounds);
                        let mut scrolled_tabs = false;
                        for (&node_id, &rect) in &layout {
                            let Some(DockNode::Tabs { tabs, active }) = dock.graph.node(node_id)
                            else {
                                continue;
                            };
                            if tabs.is_empty() {
                                continue;
                            }
                            let (tab_bar, _content) = split_tab_bar(rect);
                            if !tab_bar.contains(*position) {
                                continue;
                            }

                            self.clamp_and_ensure_active_visible(
                                node_id,
                                tab_bar,
                                tabs.len(),
                                *active,
                            );

                            let max_scroll = Self::max_tab_scroll(tab_bar, tabs.len());
                            if max_scroll.0 <= 0.0 {
                                scrolled_tabs = true;
                                break;
                            }

                            let wheel = delta.x.0 + delta.y.0;
                            let scroll = self.tab_scroll_for(node_id);
                            let next = Px((scroll.0 - wheel).clamp(0.0, max_scroll.0));
                            self.set_tab_scroll_for(node_id, next);
                            invalidate_paint = true;
                            pending_redraws.push(self.window);
                            scrolled_tabs = true;
                            break;
                        }

                        if !scrolled_tabs
                            && let Some(hit) = hit_test_active_viewport_panel(
                                &dock.graph,
                                &dock.panels,
                                &layout,
                                *position,
                            )
                            && let Some(e) = viewport_input_from_hit(
                                self.window,
                                hit,
                                *position,
                                ViewportInputKind::Wheel {
                                    delta: *delta,
                                    modifiers: *modifiers,
                                },
                            )
                        {
                            pending_effects.push(Effect::ViewportInput(e));
                            pending_redraws.push(self.window);
                        }
                    }
                    fret_core::PointerEvent::Up {
                        position,
                        button,
                        modifiers,
                    } => {
                        let mut handled = false;
                        if *button == fret_core::MouseButton::Left && self.divider_drag.is_some() {
                            self.divider_drag = None;
                            request_pointer_capture = Some(None);
                            invalidate_layout = true;
                            invalidate_paint = true;
                            pending_redraws.push(self.window);
                            handled = true;
                        }
                        if *button == fret_core::MouseButton::Left
                            && let Some((tabs_node, tab_index, panel_key)) =
                                self.pressed_tab_close.take()
                        {
                            request_pointer_capture = Some(None);

                            let (_chrome, dock_bounds) = dock_space_regions(self.last_bounds);
                            let layout = compute_layout_map(&dock.graph, root, dock_bounds);
                            let clicked = hit_test_tab(
                                &dock.graph,
                                &layout,
                                &self.tab_scroll,
                                theme,
                                *position,
                            )
                            .is_some_and(|(n, i, p, close)| {
                                close && n == tabs_node && i == tab_index && p == panel_key
                            });

                            if clicked {
                                pending_effects.push(Effect::Dock(DockOp::ClosePanel {
                                    window: self.window,
                                    panel: panel_key,
                                }));
                                invalidate_layout = true;
                            }
                            invalidate_paint = true;
                            pending_redraws.push(self.window);
                            handled = true;
                        }

                        if handled {
                            dock.hover = None;
                            invalidate_paint = true;
                        }

                        if !handled {
                            let released_capture = self
                                .viewport_capture
                                .as_ref()
                                .is_some_and(|c| c.button == *button);
                            if released_capture {
                                let capture = self.viewport_capture.take().unwrap();
                                let e = viewport_input_from_hit_clamped(
                                    self.window,
                                    capture.hit.clone(),
                                    *position,
                                    ViewportInputKind::PointerUp {
                                        button: *button,
                                        modifiers: *modifiers,
                                    },
                                );
                                pending_effects.push(Effect::ViewportInput(e));
                                pending_redraws.push(self.window);

                                dock.hover = None;
                                request_pointer_capture = Some(None);
                                invalidate_paint = true;
                            }

                            if !released_capture
                                && *button == fret_core::MouseButton::Left
                                && let Some(divider) = self.divider_drag.take()
                            {
                                pending_effects.push(Effect::Dock(DockOp::SetSplitFractionTwo {
                                    split: divider.split,
                                    first_fraction: divider.fraction,
                                }));
                                invalidate_layout = true;
                            }

                            if !released_capture
                                && *button == fret_core::MouseButton::Left
                                && let Some(drag) = dock_drag.as_ref()
                            {
                                let allow_tear_off = cx.input_ctx.caps.ui.window_tear_off;

                                if drag.dragging {
                                    match dock.hover.clone() {
                                        Some(DockDropTarget::Dock(target)) => {
                                            pending_effects.push(Effect::Dock(DockOp::MovePanel {
                                                source_window: drag.source_window,
                                                panel: drag.panel.clone(),
                                                target_window: self.window,
                                                target_tabs: target.tabs,
                                                zone: target.zone,
                                                insert_index: target.insert_index,
                                            }));
                                            invalidate_layout = true;
                                        }
                                        Some(DockDropTarget::Float { .. }) => {
                                            if allow_tear_off {
                                                pending_effects.push(Effect::Dock(
                                                    DockOp::RequestFloatPanelToNewWindow {
                                                        source_window: drag.source_window,
                                                        panel: drag.panel.clone(),
                                                        anchor: Some(fret_core::WindowAnchor {
                                                            window: self.window,
                                                            position: drag.grab_offset,
                                                        }),
                                                    },
                                                ));
                                                invalidate_layout = true;
                                            }
                                        }
                                        None => {
                                            if allow_tear_off && {
                                                !window_bounds.contains(*position)
                                                    || float_zone(dock_bounds).contains(*position)
                                            } {
                                                pending_effects.push(Effect::Dock(
                                                    DockOp::RequestFloatPanelToNewWindow {
                                                        source_window: drag.source_window,
                                                        panel: drag.panel.clone(),
                                                        anchor: Some(fret_core::WindowAnchor {
                                                            window: self.window,
                                                            position: drag.grab_offset,
                                                        }),
                                                    },
                                                ));
                                                invalidate_layout = true;
                                            }
                                        }
                                    }
                                }

                                dock.hover = None;
                                end_dock_drag = true;
                                invalidate_paint = true;
                            } else if !released_capture {
                                let (_chrome, dock_bounds) = dock_space_regions(self.last_bounds);
                                if dock_bounds.contains(*position) {
                                    let layout = compute_layout_map(&dock.graph, root, dock_bounds);
                                    if let Some(hit) = hit_test_active_viewport_panel(
                                        &dock.graph,
                                        &dock.panels,
                                        &layout,
                                        *position,
                                    ) && let Some(e) = viewport_input_from_hit(
                                        self.window,
                                        hit,
                                        *position,
                                        ViewportInputKind::PointerUp {
                                            button: *button,
                                            modifiers: *modifiers,
                                        },
                                    ) {
                                        pending_effects.push(Effect::ViewportInput(e));
                                        pending_redraws.push(self.window);
                                    }
                                }
                                dock.hover = None;
                                invalidate_paint = true;
                            }
                        }
                    }
                },
                fret_core::Event::InternalDrag(e) => {
                    let position = e.position;
                    match e.kind {
                        fret_core::InternalDragKind::Enter | fret_core::InternalDragKind::Over => {
                            if let Some(drag) = dock_drag.as_ref() {
                                let prev_hover = dock.hover.clone();
                                let mut dragging = drag.dragging;
                                if drag.source_window == self.window {
                                    let dx = position.x.0 - drag.start.x.0;
                                    let dy = position.y.0 - drag.start.y.0;
                                    let dist2 = dx * dx + dy * dy;
                                    // Match ImGui's default drag threshold (~6px).
                                    if !dragging && dist2 > 36.0 {
                                        dragging = true;
                                    }
                                } else if !dragging {
                                    dragging = true;
                                }

                                update_drag = Some((position, dragging));

                                if dragging {
                                    let allow_tear_off = cx.input_ctx.caps.ui.window_tear_off;
                                    let margin = Px(10.0);
                                    let requested_tear_off = allow_tear_off
                                        && drag.source_window == self.window
                                        && is_outside_bounds_with_margin(
                                            window_bounds,
                                            position,
                                            margin,
                                        )
                                        && self.tear_off_in_flight.is_none();

                                    if requested_tear_off {
                                        self.tear_off_in_flight = Some(DockTearOffKey {
                                            source_window: drag.source_window,
                                            start: drag.start,
                                            panel: drag.panel.clone(),
                                        });
                                        pending_effects.push(Effect::Dock(
                                            DockOp::RequestFloatPanelToNewWindow {
                                                source_window: drag.source_window,
                                                panel: drag.panel.clone(),
                                                anchor: Some(fret_core::WindowAnchor {
                                                    window: self.window,
                                                    position: drag.grab_offset,
                                                }),
                                            },
                                        ));
                                        invalidate_layout = true;
                                        dock.hover = None;
                                        pending_redraws.push(self.window);
                                        invalidate_paint = true;
                                    }

                                    if !requested_tear_off {
                                        if allow_tear_off
                                            && (!window_bounds.contains(position)
                                                || float_zone(dock_bounds).contains(position))
                                        {
                                            dock.hover = Some(DockDropTarget::Float {
                                                window: self.window,
                                            });
                                        } else if dock_bounds.contains(position) {
                                            let layout =
                                                compute_layout_map(&dock.graph, root, dock_bounds);
                                            dock.hover = hit_test_drop_target(
                                                &dock.graph,
                                                &layout,
                                                &self.tab_scroll,
                                                position,
                                            )
                                            .map(DockDropTarget::Dock);
                                        } else {
                                            dock.hover = None;
                                        }
                                    }
                                } else {
                                    dock.hover = None;
                                }

                                if dock.hover != prev_hover {
                                    pending_redraws.push(self.window);
                                    invalidate_paint = true;
                                }
                            } else {
                                dock.hover = None;
                            }
                        }
                        fret_core::InternalDragKind::Leave
                        | fret_core::InternalDragKind::Cancel => {
                            if dock.hover.take().is_some() {
                                pending_redraws.push(self.window);
                                invalidate_paint = true;
                            }
                            dock.hover = None;
                        }
                        fret_core::InternalDragKind::Drop => {
                            let prev_hover = dock.hover.clone();
                            if let Some(drag) = dock_drag.as_ref() {
                                let mut dragging = drag.dragging;
                                if !dragging && drag.source_window != self.window {
                                    dragging = true;
                                }

                                if dragging {
                                    let allow_tear_off = cx.input_ctx.caps.ui.window_tear_off;
                                    if allow_tear_off && float_zone(dock_bounds).contains(position)
                                    {
                                        dock.hover = Some(DockDropTarget::Float {
                                            window: self.window,
                                        });
                                    } else if dock_bounds.contains(position) {
                                        let layout =
                                            compute_layout_map(&dock.graph, root, dock_bounds);
                                        dock.hover = hit_test_drop_target(
                                            &dock.graph,
                                            &layout,
                                            &self.tab_scroll,
                                            position,
                                        )
                                        .map(DockDropTarget::Dock);
                                    } else {
                                        dock.hover = None;
                                    }

                                    match dock.hover.clone() {
                                        Some(DockDropTarget::Dock(target)) => {
                                            pending_effects.push(Effect::Dock(DockOp::MovePanel {
                                                source_window: drag.source_window,
                                                panel: drag.panel.clone(),
                                                target_window: self.window,
                                                target_tabs: target.tabs,
                                                zone: target.zone,
                                                insert_index: target.insert_index,
                                            }));
                                            invalidate_layout = true;
                                        }
                                        Some(DockDropTarget::Float { .. }) => {
                                            if allow_tear_off {
                                                pending_effects.push(Effect::Dock(
                                                    DockOp::RequestFloatPanelToNewWindow {
                                                        source_window: drag.source_window,
                                                        panel: drag.panel.clone(),
                                                        anchor: Some(fret_core::WindowAnchor {
                                                            window: self.window,
                                                            position: drag.grab_offset,
                                                        }),
                                                    },
                                                ));
                                                invalidate_layout = true;
                                            }
                                        }
                                        None => {
                                            if allow_tear_off
                                                && (!window_bounds.contains(position)
                                                    || float_zone(dock_bounds).contains(position))
                                            {
                                                pending_effects.push(Effect::Dock(
                                                    DockOp::RequestFloatPanelToNewWindow {
                                                        source_window: drag.source_window,
                                                        panel: drag.panel.clone(),
                                                        anchor: Some(fret_core::WindowAnchor {
                                                            window: self.window,
                                                            position: drag.grab_offset,
                                                        }),
                                                    },
                                                ));
                                                invalidate_layout = true;
                                            }
                                        }
                                    }
                                }

                                dock.hover = None;
                                end_dock_drag = true;
                                invalidate_paint = true;
                                pending_redraws.push(self.window);
                            } else {
                                // Drop can be delivered after the drag session is already cleared
                                // by the runner/driver. Always clear stale hover so the UI doesn't
                                // get stuck in a highlighted state.
                                dock.hover = None;
                                if prev_hover.is_some() {
                                    invalidate_paint = true;
                                    pending_redraws.push(self.window);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if request_focus.is_none()
            && let Some(panel) = request_focus_panel
        {
            let panel_nodes = self.panel_nodes(cx.app);
            request_focus = panel_nodes.get(&panel).copied();
        }

        if let Some((start, panel, grab_offset)) = begin_drag {
            cx.app.begin_cross_window_drag_with_kind(
                DragKind::DockPanel,
                self.window,
                start,
                DockPanelDragPayload { panel, grab_offset },
            );
        }

        if let Some(request) = request_pointer_capture {
            match request {
                Some(node) => cx.capture_pointer(node),
                None => cx.release_pointer_capture(),
            }
        }

        if let Some((position, dragging)) = update_drag
            && let Some(drag) = cx.app.drag_mut()
            && drag.payload::<DockPanelDragPayload>().is_some()
        {
            drag.position = position;
            drag.dragging = dragging;
        }

        if end_dock_drag
            && cx
                .app
                .drag()
                .and_then(|d| d.payload::<DockPanelDragPayload>())
                .is_some()
        {
            self.tear_off_in_flight = None;
            cx.app.cancel_drag();
        }

        if let Some(node) = request_focus {
            cx.request_focus(node);
        }
        if let Some(icon) = request_cursor {
            cx.set_cursor_icon(icon);
        }
        if invalidate_layout {
            cx.invalidate(cx.node, Invalidation::Layout);
        }
        if invalidate_paint {
            cx.invalidate(cx.node, Invalidation::Paint);
        }

        for window in pending_redraws {
            cx.app.request_redraw(window);
        }
        for effect in pending_effects {
            cx.app.push_effect(effect);
        }
    }

    fn command(&mut self, cx: &mut CommandCx<'_, H>, command: &CommandId) -> bool {
        match command.as_str() {
            "dock.focus_requested_panel" => {
                let Some(panel) = cx.app.with_global_mut(
                    DockFocusRequestService::default,
                    |service: &mut DockFocusRequestService, _app| service.take(self.window),
                ) else {
                    return false;
                };

                let panel_nodes = self.panel_nodes(cx.app);
                if let Some(node) = panel_nodes.get(&panel).copied() {
                    cx.request_focus(node);
                } else {
                    cx.request_focus(cx.node);
                }
                cx.request_redraw();
                cx.stop_propagation();
                true
            }
            _ => false,
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.last_bounds = cx.bounds;
        let hidden = hidden_bounds(Size::new(Px(0.0), Px(0.0)));

        cx.app
            .with_global_mut(InternalDragRouteService::default, |routes, _app| {
                routes.set(self.window, DragKind::DockPanel, cx.node);
            });
        if let Some(dock) = cx.app.global_mut::<DockManager>() {
            dock.register_dock_space_node(self.window, cx.node);
        }

        let Some((active_bounds, layout)) = (|| {
            let dock = cx.app.global::<DockManager>()?;
            let root = dock.graph.window_root(self.window)?;
            let (_chrome, dock_bounds) = dock_space_regions(cx.bounds);
            let layout = compute_layout_map(&dock.graph, root, dock_bounds);
            Some((active_panel_content_bounds(&dock.graph, &layout), layout))
        })() else {
            for &child in cx.children {
                let _ = cx.layout_in(child, hidden);
            }
            return cx.available;
        };

        if let Some(dock) = cx.app.global::<DockManager>() {
            let mut visible_tabs_nodes: HashSet<DockNodeId> = HashSet::new();
            for (&node_id, &rect) in &layout {
                let Some(DockNode::Tabs { tabs, active }) = dock.graph.node(node_id) else {
                    continue;
                };
                visible_tabs_nodes.insert(node_id);

                let (tab_bar, _content) = split_tab_bar(rect);
                self.clamp_and_ensure_active_visible(node_id, tab_bar, tabs.len(), *active);
            }
            self.tab_scroll
                .retain(|tabs_node, _| visible_tabs_nodes.contains(tabs_node));
        }

        let panel_nodes = self.panel_nodes(cx.app);
        let mut laid_out: HashSet<NodeId> = HashSet::new();
        for (panel, node) in &panel_nodes {
            let bounds = match active_bounds.get(panel).copied() {
                Some(rect) => {
                    self.panel_last_sizes.insert(panel.clone(), rect.size);
                    rect
                }
                None => hidden_bounds(
                    self.panel_last_sizes
                        .get(panel)
                        .copied()
                        .unwrap_or(Size::new(Px(0.0), Px(0.0))),
                ),
            };
            let _ = cx.layout_in(*node, bounds);
            laid_out.insert(*node);
        }

        for &child in cx.children {
            if laid_out.contains(&child) {
                continue;
            }
            let _ = cx.layout_in(child, hidden);
        }

        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.last_bounds = cx.bounds;
        let Some((_chrome, layout, active_bounds, hover)) = (|| {
            let dock = cx.app.global::<DockManager>()?;
            let root = dock.graph.window_root(self.window)?;
            let (chrome, dock_bounds) = dock_space_regions(cx.bounds);
            let layout = compute_layout_map(&dock.graph, root, dock_bounds);
            let active_bounds = active_panel_content_bounds(&dock.graph, &layout);
            Some((chrome, layout, active_bounds, dock.hover.clone()))
        })() else {
            self.paint_empty_state(cx);
            return;
        };

        let theme = cx.theme().snapshot();
        if let Some(dock) = cx.app.global::<DockManager>() {
            self.rebuild_tab_titles(cx.services, theme, cx.scale_factor, dock, &layout);
        }

        if let Some(dock) = cx.app.global_mut::<DockManager>() {
            dock.clear_viewport_layout_for_window(self.window);
            for (&node_id, &rect) in layout.iter() {
                let (_tab_bar, content) = split_tab_bar(rect);
                let target = (|| {
                    let DockNode::Tabs { tabs, active } = dock.graph.node(node_id)?.clone() else {
                        return None;
                    };
                    let panel_key = tabs.get(active)?;
                    let panel = dock.panel(panel_key)?;
                    panel.viewport.map(|vp| vp.target)
                })();
                if let Some(target) = target {
                    dock.set_viewport_content_rect(self.window, target, content);
                }
            }
        }
        if let Some(dock) = cx.app.global::<DockManager>() {
            let overlay_hooks = cx
                .app
                .global::<DockViewportOverlayHooksService>()
                .and_then(|svc| svc.hooks());
            paint_dock(
                cx.theme().snapshot(),
                dock,
                PaintDockParams {
                    window: self.window,
                    layout: &layout,
                    tab_titles: &self.tab_titles,
                    hovered_tab: self.hovered_tab,
                    hovered_tab_close: self.hovered_tab_close,
                    pressed_tab_close: self.pressed_tab_close.as_ref().map(|(n, i, _)| (*n, *i)),
                    tab_scroll: &self.tab_scroll,
                    tab_close_glyph: self.tab_close_glyph,
                },
                overlay_hooks.as_deref(),
                cx.scene,
            );
        }

        let panel_nodes = self.panel_nodes(cx.app);
        for (panel, rect) in active_bounds {
            let Some(node) = panel_nodes.get(&panel) else {
                continue;
            };
            if let Some(bounds) = cx.child_bounds(*node) {
                cx.paint(*node, bounds);
            } else {
                cx.paint(*node, rect);
            }
        }

        if let Some(dock) = cx.app.global::<DockManager>() {
            paint_split_handles(
                cx.theme().snapshot(),
                &dock.graph,
                &layout,
                self.divider_drag.map(|d| d.split),
                cx.scale_factor,
                cx.scene,
            );
        }
        let is_dock_dragging = cx
            .app
            .drag()
            .is_some_and(|d| d.dragging && d.payload::<DockPanelDragPayload>().is_some());
        if is_dock_dragging {
            paint_drop_hints(
                cx.theme().snapshot(),
                hover.clone(),
                self.window,
                cx.bounds,
                &layout,
                cx.scene,
            );
        }
        paint_drop_overlay(
            cx.theme().snapshot(),
            hover,
            self.window,
            cx.bounds,
            &layout,
            &self.tab_scroll,
            cx.scene,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_host::TestHost;
    use fret_core::{
        AppWindowId, Event, InternalDragEvent, InternalDragKind, PlatformCapabilities, Point, Px,
        Scene, SceneOp, Size, TextConstraints, TextMetrics, TextService, TextStyle,
    };
    use fret_ui::UiTree;
    use fret_ui::retained_bridge::UiTreeRetainedExt as _;

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

    #[test]
    fn compute_split_fraction_handles_small_bounds() {
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(300.0)),
        );
        let first = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(60.0), Px(300.0)));
        let second = Rect::new(
            Point::new(Px(60.0), Px(0.0)),
            Size::new(Px(60.0), Px(300.0)),
        );
        let pos = Point::new(Px(60.0), Px(10.0));
        assert_eq!(
            compute_split_fraction(fret_core::Axis::Horizontal, bounds, first, second, 0.0, pos),
            None
        );
    }

    #[test]
    fn compute_split_fraction_handles_nan_bounds() {
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(f32::NAN), Px(300.0)),
        );
        let first = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(0.0), Px(300.0)));
        let second = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(0.0), Px(300.0)));
        let pos = Point::new(Px(60.0), Px(10.0));
        assert_eq!(
            compute_split_fraction(fret_core::Axis::Horizontal, bounds, first, second, 0.0, pos),
            None
        );
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
            }),
        );

        let hover = app.global::<DockManager>().and_then(|d| d.hover.clone());
        assert!(hover.is_none(), "dock hover should be cleared on drop");
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
}
