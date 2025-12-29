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

fn compute_layout_map(
    graph: &DockGraph,
    root: DockNodeId,
    bounds: Rect,
) -> std::collections::HashMap<DockNodeId, Rect> {
    let mut layout = std::collections::HashMap::new();
    compute_layout_map_impl(graph, root, bounds, &mut layout);
    layout
}

fn compute_layout_map_impl(
    graph: &DockGraph,
    node: DockNodeId,
    bounds: Rect,
    out: &mut std::collections::HashMap<DockNodeId, Rect>,
) {
    let Some(n) = graph.node(node) else {
        return;
    };

    out.insert(node, bounds);
    match n {
        DockNode::Tabs { .. } => {}
        DockNode::Split {
            axis,
            children,
            fractions,
        } => {
            let count = children.len().min(fractions.len());
            if count == 0 {
                return;
            }

            let total: f32 = fractions.iter().take(count).sum();
            let total = if total <= 0.0 { 1.0 } else { total };

            let axis_len = match axis {
                fret_core::Axis::Horizontal => bounds.size.width.0,
                fret_core::Axis::Vertical => bounds.size.height.0,
            };
            if !axis_len.is_finite() || axis_len <= 0.0 {
                return;
            }

            let gaps = count.saturating_sub(1) as f32;
            let mut gap = DOCK_SPLIT_HANDLE_GAP.0;
            if gaps == 0.0 || axis_len <= gap * gaps {
                gap = 0.0;
            }

            let available = axis_len - gap * gaps;
            if !available.is_finite() || available <= 0.0 {
                return;
            }

            let mut cursor = 0.0;
            for i in 0..count {
                let f = (fractions[i] / total).max(0.0);
                let (child_axis_len, next_cursor) = if i + 1 == count {
                    let remaining = (available - cursor).max(0.0);
                    (remaining, available)
                } else {
                    let len = available * f;
                    (len, cursor + len)
                };

                let origin_axis = cursor + gap * (i as f32);
                let child_rect = match axis {
                    fret_core::Axis::Horizontal => Rect {
                        origin: Point::new(Px(bounds.origin.x.0 + origin_axis), bounds.origin.y),
                        size: Size::new(Px(child_axis_len), bounds.size.height),
                    },
                    fret_core::Axis::Vertical => Rect {
                        origin: Point::new(bounds.origin.x, Px(bounds.origin.y.0 + origin_axis)),
                        size: Size::new(bounds.size.width, Px(child_axis_len)),
                    },
                };

                cursor = next_cursor;
                compute_layout_map_impl(graph, children[i], child_rect, out);
            }
        }
    }
}

fn hidden_bounds(size: Size) -> Rect {
    Rect {
        origin: Point::new(Px(-1_000_000.0), Px(-1_000_000.0)),
        size,
    }
}

fn active_panel_content_bounds(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
) -> std::collections::HashMap<PanelKey, Rect> {
    let mut out: std::collections::HashMap<PanelKey, Rect> = std::collections::HashMap::new();

    for (&node_id, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, active }) = graph.node(node_id) else {
            continue;
        };
        let (_tab_bar, content) = split_tab_bar(rect);
        if let Some(panel) = tabs.get(*active) {
            out.insert(panel.clone(), content);
        }
    }

    out
}

struct PaintDockParams<'a> {
    window: fret_core::AppWindowId,
    layout: &'a std::collections::HashMap<DockNodeId, Rect>,
    tab_titles: &'a HashMap<PanelKey, PreparedTabTitle>,
    hovered_tab: Option<(DockNodeId, usize)>,
    hovered_tab_close: bool,
    pressed_tab_close: Option<(DockNodeId, usize)>,
    tab_scroll: &'a HashMap<DockNodeId, Px>,
    tab_close_glyph: Option<PreparedTabTitle>,
}

fn paint_dock(
    theme: fret_ui::ThemeSnapshot,
    dock: &DockManager,
    params: PaintDockParams<'_>,
    overlay_hooks: Option<&dyn DockViewportOverlayHooks>,
    scene: &mut Scene,
) {
    let PaintDockParams {
        window,
        layout,
        tab_titles,
        hovered_tab,
        hovered_tab_close,
        pressed_tab_close,
        tab_scroll,
        tab_close_glyph,
    } = params;
    let graph = &dock.graph;
    for (&node_id, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, active }) = graph.node(node_id) else {
            continue;
        };
        let (tab_bar, content) = split_tab_bar(rect);

        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(0),
            rect,
            background: theme.colors.panel_background,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });

        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(1),
            rect: tab_bar,
            background: theme.colors.surface_background,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });

        let scroll = tab_scroll_for_node(tab_scroll, node_id);
        scene.push(SceneOp::PushClipRect { rect: tab_bar });

        for (i, panel) in tabs.iter().enumerate() {
            let tab_rect = tab_rect_for_index(tab_bar, i, scroll);
            if tab_rect.origin.x.0 + tab_rect.size.width.0 < tab_bar.origin.x.0
                || tab_rect.origin.x.0 > tab_bar.origin.x.0 + tab_bar.size.width.0
            {
                continue;
            }

            let is_active = i == *active;
            let is_hovered = hovered_tab == Some((node_id, i));
            let bg = if is_active {
                theme.colors.panel_background
            } else if is_hovered {
                theme.colors.hover_background
            } else {
                Color {
                    a: 0.0,
                    ..theme.colors.panel_background
                }
            };

            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(2),
                rect: tab_rect,
                background: bg,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });

            if is_active {
                let underline_h = Px(2.0);
                let underline = Rect {
                    origin: Point::new(
                        tab_rect.origin.x,
                        Px(tab_rect.origin.y.0 + tab_rect.size.height.0 - underline_h.0),
                    ),
                    size: Size::new(tab_rect.size.width, underline_h),
                };
                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(3),
                    rect: underline,
                    background: theme.colors.accent,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });
            }

            if let Some(title) = tab_titles.get(panel) {
                let pad_x = theme.metrics.padding_md;
                let text_x = Px(tab_rect.origin.x.0 + pad_x.0);
                let inner_y = tab_rect.origin.y.0
                    + ((tab_rect.size.height.0 - title.metrics.size.height.0) * 0.5);
                let text_y = Px(inner_y + title.metrics.baseline.0);
                let text_color = if is_active || is_hovered {
                    theme.colors.text_primary
                } else {
                    theme.colors.text_muted
                };

                scene.push(SceneOp::PushClipRect { rect: tab_rect });
                scene.push(SceneOp::Text {
                    order: fret_core::DrawOrder(4),
                    origin: Point::new(text_x, text_y),
                    text: title.blob,
                    color: text_color,
                });
                scene.push(SceneOp::PopClip);
            }

            if (is_active || is_hovered) && tab_close_glyph.is_some() {
                let close_rect = tab_close_rect(theme, tab_rect);
                let close_hovered = is_hovered && hovered_tab_close;
                let close_pressed = pressed_tab_close == Some((node_id, i));

                if close_pressed || close_hovered {
                    scene.push(SceneOp::Quad {
                        order: fret_core::DrawOrder(5),
                        rect: close_rect,
                        background: theme.colors.hover_background,
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: fret_core::Corners::all(theme.metrics.radius_sm),
                    });
                }

                if let Some(glyph) = tab_close_glyph {
                    let text_x = Px(close_rect.origin.x.0
                        + (close_rect.size.width.0 - glyph.metrics.size.width.0) * 0.5);
                    let inner_y = close_rect.origin.y.0
                        + ((close_rect.size.height.0 - glyph.metrics.size.height.0) * 0.5);
                    let text_y = Px(inner_y + glyph.metrics.baseline.0);
                    let color = if close_pressed || close_hovered {
                        theme.colors.text_primary
                    } else {
                        theme.colors.text_muted
                    };
                    scene.push(SceneOp::Text {
                        order: fret_core::DrawOrder(6),
                        origin: Point::new(text_x, text_y),
                        text: glyph.blob,
                        color,
                    });
                }
            }
        }

        scene.push(SceneOp::PopClip);

        let active_panel = tabs.get(*active);
        if let Some(panel) = active_panel.and_then(|p| dock.panel(p)) {
            if let Some(vp) = panel.viewport {
                let mapping = ViewportMapping {
                    content_rect: content,
                    target_px_size: vp.target_px_size,
                    fit: vp.fit,
                };
                let draw_rect = mapping.map().draw_rect;

                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(3),
                    rect: content,
                    background: panel.color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(theme.metrics.radius_sm),
                });

                scene.push(SceneOp::PushClipRect { rect: content });
                scene.push(SceneOp::ViewportSurface {
                    order: fret_core::DrawOrder(4),
                    rect: draw_rect,
                    target: vp.target,
                    opacity: 1.0,
                });
                if let Some(hooks) = overlay_hooks
                    && let Some(panel_key) = active_panel
                {
                    hooks.paint(theme, window, panel_key, vp, mapping, draw_rect, scene);
                }
                scene.push(SceneOp::PopClip);
            } else {
                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(3),
                    rect: content,
                    background: panel.color,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(theme.metrics.radius_sm),
                });
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ViewportHit {
    panel: PanelKey,
    viewport: ViewportPanel,
    content: Rect,
    draw_rect: Rect,
}

#[derive(Debug, Clone, PartialEq)]
struct ViewportCaptureState {
    hit: ViewportHit,
    button: fret_core::MouseButton,
    start: Point,
    moved: bool,
}

fn viewport_input_from_hit(
    window: fret_core::AppWindowId,
    hit: ViewportHit,
    position: Point,
    kind: ViewportInputKind,
) -> Option<ViewportInputEvent> {
    let mapping = ViewportMapping {
        content_rect: hit.content,
        target_px_size: hit.viewport.target_px_size,
        fit: hit.viewport.fit,
    };
    let uv = mapping.window_point_to_uv(position)?;
    let target_px = mapping.window_point_to_target_px(position)?;
    Some(ViewportInputEvent {
        window,
        target: hit.viewport.target,
        uv,
        target_px,
        kind,
    })
}

fn viewport_input_from_hit_clamped(
    window: fret_core::AppWindowId,
    hit: ViewportHit,
    position: Point,
    kind: ViewportInputKind,
) -> ViewportInputEvent {
    let mapping = ViewportMapping {
        content_rect: hit.content,
        target_px_size: hit.viewport.target_px_size,
        fit: hit.viewport.fit,
    };
    let uv = mapping.window_point_to_uv_clamped(position);
    let target_px = mapping.window_point_to_target_px_clamped(position);
    ViewportInputEvent {
        window,
        target: hit.viewport.target,
        uv,
        target_px,
        kind,
    }
}

fn hit_test_active_viewport_panel(
    graph: &DockGraph,
    panels: &HashMap<PanelKey, DockPanel>,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    position: Point,
) -> Option<ViewportHit> {
    for (&node_id, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, active }) = graph.node(node_id) else {
            continue;
        };
        let Some(panel_key) = tabs.get(*active).cloned() else {
            continue;
        };
        let Some(panel) = panels.get(&panel_key) else {
            continue;
        };
        let Some(viewport) = panel.viewport else {
            continue;
        };

        let (_tab_bar, content) = split_tab_bar(rect);
        let mapping = ViewportMapping {
            content_rect: content,
            target_px_size: viewport.target_px_size,
            fit: viewport.fit,
        };
        let draw_rect = mapping.map().draw_rect;
        if draw_rect.contains(position) {
            return Some(ViewportHit {
                panel: panel_key,
                viewport,
                content,
                draw_rect,
            });
        }
    }
    None
}

fn split_tab_bar(rect: Rect) -> (Rect, Rect) {
    let tab_bar = Rect {
        origin: rect.origin,
        size: Size::new(rect.size.width, Px(DOCK_TAB_H.0.min(rect.size.height.0))),
    };
    let content = Rect {
        origin: Point::new(rect.origin.x, Px(rect.origin.y.0 + tab_bar.size.height.0)),
        size: Size::new(
            rect.size.width,
            Px((rect.size.height.0 - tab_bar.size.height.0).max(0.0)),
        ),
    };
    (tab_bar, content)
}

fn dock_drop_edge_thickness(rect: Rect) -> Px {
    let min_dim = rect.size.width.0.min(rect.size.height.0);
    // Keep split zones usable on large panels, but avoid making "center tab" drops difficult.
    // Also keep the thickness sane on small panels.
    // ImGui-style: edge splits should be easy to hit even on big panels; we still cap it so the
    // center/tab drop remains a first-class target.
    let base = (min_dim * 0.30).clamp(20.0, 120.0);
    let cap = (min_dim * 0.44).clamp(20.0, 120.0);
    Px(base.min(cap))
}

fn drop_zone_rect(rect: Rect, zone: DropZone) -> Rect {
    if zone == DropZone::Center {
        return rect;
    }
    let thickness = dock_drop_edge_thickness(rect).0;
    match zone {
        DropZone::Left => Rect {
            origin: rect.origin,
            size: Size::new(Px(thickness), rect.size.height),
        },
        DropZone::Right => Rect {
            origin: Point::new(
                Px(rect.origin.x.0 + rect.size.width.0 - thickness),
                rect.origin.y,
            ),
            size: Size::new(Px(thickness), rect.size.height),
        },
        DropZone::Top => Rect {
            origin: rect.origin,
            size: Size::new(rect.size.width, Px(thickness)),
        },
        DropZone::Bottom => Rect {
            origin: Point::new(
                rect.origin.x,
                Px(rect.origin.y.0 + rect.size.height.0 - thickness),
            ),
            size: Size::new(rect.size.width, Px(thickness)),
        },
        DropZone::Center => rect,
    }
}

fn float_zone(bounds: Rect) -> Rect {
    let size = Px(34.0);
    Rect {
        origin: Point::new(Px(bounds.origin.x.0 + 8.0), Px(bounds.origin.y.0 + 8.0)),
        size: Size::new(size, size),
    }
}

fn tab_scroll_for_node(tab_scroll: &HashMap<DockNodeId, Px>, node: DockNodeId) -> Px {
    tab_scroll.get(&node).copied().unwrap_or(Px(0.0))
}

fn tab_rect_for_index(tab_bar: Rect, index: usize, scroll: Px) -> Rect {
    Rect {
        origin: Point::new(
            Px(tab_bar.origin.x.0 + DOCK_TAB_W.0 * index as f32 - scroll.0),
            tab_bar.origin.y,
        ),
        size: Size::new(DOCK_TAB_W, tab_bar.size.height),
    }
}

fn tab_close_rect(theme: fret_ui::ThemeSnapshot, tab_rect: Rect) -> Rect {
    let pad = theme.metrics.padding_sm.0.max(0.0);
    let x = tab_rect.origin.x.0 + tab_rect.size.width.0 - pad - DOCK_TAB_CLOSE_SIZE.0;
    let y = tab_rect.origin.y.0 + (tab_rect.size.height.0 - DOCK_TAB_CLOSE_SIZE.0) * 0.5;
    Rect::new(
        Point::new(Px(x), Px(y)),
        Size::new(DOCK_TAB_CLOSE_SIZE, DOCK_TAB_CLOSE_SIZE),
    )
}

fn hit_test_tab(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    tab_scroll: &HashMap<DockNodeId, Px>,
    theme: fret_ui::ThemeSnapshot,
    position: Point,
) -> Option<(DockNodeId, usize, PanelKey, bool)> {
    for (&node, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, .. }) = graph.node(node) else {
            continue;
        };
        if tabs.is_empty() {
            continue;
        }
        let (tab_bar, _content) = split_tab_bar(rect);
        if !tab_bar.contains(position) {
            continue;
        }
        let scroll = tab_scroll_for_node(tab_scroll, node);
        let rel_x = position.x.0 - tab_bar.origin.x.0 + scroll.0;
        let idx = (rel_x / DOCK_TAB_W.0).floor() as isize;
        if idx < 0 {
            continue;
        }
        let idx = idx as usize;
        let panel = tabs.get(idx)?.clone();
        let tab_rect = tab_rect_for_index(tab_bar, idx, scroll);
        let close = tab_close_rect(theme, tab_rect).contains(position);
        return Some((node, idx, panel, close));
    }
    None
}

fn hit_test_drop_target(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    tab_scroll: &HashMap<DockNodeId, Px>,
    position: Point,
) -> Option<HoverTarget> {
    for (&node, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, .. }) = graph.node(node) else {
            continue;
        };
        if !rect.contains(position) {
            continue;
        }

        let (tab_bar, _content) = split_tab_bar(rect);
        if tab_bar.contains(position) {
            let scroll = tab_scroll_for_node(tab_scroll, node);
            let insert_index = compute_tab_insert_index(tab_bar, scroll, tabs.len(), position);
            return Some(HoverTarget {
                tabs: node,
                zone: DropZone::Center,
                insert_index: Some(insert_index),
            });
        }

        // ImGui-style direction-pad hit targets near the center of the hovered dock node.
        // This makes split docking discoverable and avoids requiring the cursor to be near edges.
        for (zone, hint_rect) in dock_hint_rects(rect) {
            if hint_rect.contains(position) {
                return Some(HoverTarget {
                    tabs: node,
                    zone,
                    insert_index: None,
                });
            }
        }

        let thickness = dock_drop_edge_thickness(rect).0;
        let left = position.x.0 - rect.origin.x.0;
        let right = rect.origin.x.0 + rect.size.width.0 - position.x.0;
        let top = position.y.0 - rect.origin.y.0;
        let bottom = rect.origin.y.0 + rect.size.height.0 - position.y.0;

        let mut zone = DropZone::Center;
        let mut best = thickness;
        for (candidate, dist) in [
            (DropZone::Left, left),
            (DropZone::Right, right),
            (DropZone::Top, top),
            (DropZone::Bottom, bottom),
        ] {
            if dist < best {
                best = dist;
                zone = candidate;
            }
        }

        return Some(HoverTarget {
            tabs: node,
            zone,
            insert_index: None,
        });
    }
    None
}

fn compute_tab_insert_index(tab_bar: Rect, scroll: Px, tab_count: usize, position: Point) -> usize {
    let rel_x = position.x.0 - tab_bar.origin.x.0 + scroll.0;
    let raw = (rel_x / DOCK_TAB_W.0) + 0.5;
    let idx = raw.floor() as isize;
    idx.clamp(0, tab_count as isize) as usize
}

fn split_children_two(graph: &DockGraph, split: DockNodeId) -> Option<(DockNodeId, DockNodeId)> {
    let Some(DockNode::Split { children, .. }) = graph.node(split) else {
        return None;
    };
    if children.len() != 2 {
        return None;
    }
    Some((children[0], children[1]))
}

fn hit_test_split_handle(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    position: Point,
) -> Option<DividerDragState> {
    for (&node, &bounds) in layout.iter() {
        let Some(DockNode::Split {
            axis,
            children,
            fractions,
        }) = graph.node(node)
        else {
            continue;
        };
        if children.len() != 2 {
            continue;
        }
        if !bounds.contains(position) {
            continue;
        }

        let Some(left) = layout.get(&children[0]).copied() else {
            continue;
        };
        let Some(right) = layout.get(&children[1]).copied() else {
            continue;
        };

        let handle = split_handle_rect(*axis, bounds, left, right, DOCK_SPLIT_HANDLE_HIT_THICKNESS);

        if handle.contains(position) {
            let total = fractions.iter().take(2).sum::<f32>();
            let total = if total <= 0.0 { 1.0 } else { total };
            let f0 = fractions.first().copied().unwrap_or(0.5) / total;
            let center = split_handle_center(*axis, left, right);
            let grab_offset = match axis {
                fret_core::Axis::Horizontal => position.x.0 - center,
                fret_core::Axis::Vertical => position.y.0 - center,
            };
            return Some(DividerDragState {
                split: node,
                axis: *axis,
                bounds,
                fraction: f0,
                grab_offset,
            });
        }
    }

    None
}

fn split_gap(axis: fret_core::Axis, first: Rect, second: Rect) -> f32 {
    let gap = match axis {
        fret_core::Axis::Horizontal => second.origin.x.0 - (first.origin.x.0 + first.size.width.0),
        fret_core::Axis::Vertical => second.origin.y.0 - (first.origin.y.0 + first.size.height.0),
    };
    if gap.is_finite() { gap.max(0.0) } else { 0.0 }
}

fn split_handle_center(axis: fret_core::Axis, first: Rect, second: Rect) -> f32 {
    let gap = split_gap(axis, first, second);
    match axis {
        fret_core::Axis::Horizontal => {
            let start = first.origin.x.0 + first.size.width.0;
            if gap > 0.0 { start + gap * 0.5 } else { start }
        }
        fret_core::Axis::Vertical => {
            let start = first.origin.y.0 + first.size.height.0;
            if gap > 0.0 { start + gap * 0.5 } else { start }
        }
    }
}

fn split_handle_rect(
    axis: fret_core::Axis,
    bounds: Rect,
    first: Rect,
    second: Rect,
    thickness: Px,
) -> Rect {
    let gap = split_gap(axis, first, second);
    if gap > 0.0 {
        match axis {
            fret_core::Axis::Horizontal => Rect {
                origin: Point::new(Px(first.origin.x.0 + first.size.width.0), bounds.origin.y),
                size: Size::new(Px(gap), bounds.size.height),
            },
            fret_core::Axis::Vertical => Rect {
                origin: Point::new(bounds.origin.x, Px(first.origin.y.0 + first.size.height.0)),
                size: Size::new(bounds.size.width, Px(gap)),
            },
        }
    } else {
        let center = split_handle_center(axis, first, second);
        match axis {
            fret_core::Axis::Horizontal => Rect {
                origin: Point::new(Px(center - thickness.0 * 0.5), bounds.origin.y),
                size: Size::new(thickness, bounds.size.height),
            },
            fret_core::Axis::Vertical => Rect {
                origin: Point::new(bounds.origin.x, Px(center - thickness.0 * 0.5)),
                size: Size::new(bounds.size.width, thickness),
            },
        }
    }
}

fn compute_split_fraction(
    axis: fret_core::Axis,
    bounds: Rect,
    first: Rect,
    second: Rect,
    grab_offset: f32,
    position: Point,
) -> Option<f32> {
    let min_px = 120.0;
    match axis {
        fret_core::Axis::Horizontal => {
            let w = bounds.size.width.0;
            if !w.is_finite() {
                return None;
            }
            let gap = split_gap(axis, first, second);
            let avail = w - gap;
            if !avail.is_finite() || avail <= min_px * 2.0 {
                return None;
            }
            let max_x = (avail - min_px).max(min_px);
            let anchor = position.x.0 - grab_offset - bounds.origin.x.0;
            let x = (anchor - gap * 0.5).clamp(min_px, max_x);
            Some(x / avail)
        }
        fret_core::Axis::Vertical => {
            let h = bounds.size.height.0;
            if !h.is_finite() {
                return None;
            }
            let gap = split_gap(axis, first, second);
            let avail = h - gap;
            if !avail.is_finite() || avail <= min_px * 2.0 {
                return None;
            }
            let max_y = (avail - min_px).max(min_px);
            let anchor = position.y.0 - grab_offset - bounds.origin.y.0;
            let y = (anchor - gap * 0.5).clamp(min_px, max_y);
            Some(y / avail)
        }
    }
}

fn paint_split_handles(
    theme: fret_ui::ThemeSnapshot,
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    active: Option<DockNodeId>,
    scale_factor: f32,
    scene: &mut Scene,
) {
    for (&node, &bounds) in layout.iter() {
        let Some(DockNode::Split { axis, children, .. }) = graph.node(node) else {
            continue;
        };
        if children.len() != 2 {
            continue;
        }
        let Some(first) = layout.get(&children[0]).copied() else {
            continue;
        };
        let Some(second) = layout.get(&children[1]).copied() else {
            continue;
        };

        let center = split_handle_center(*axis, first, second);

        let background = if active == Some(node) {
            theme.colors.focus_ring
        } else {
            theme.colors.panel_border
        };

        ResizeHandle {
            axis: *axis,
            hit_thickness: DOCK_SPLIT_HANDLE_HIT_THICKNESS,
            paint_device_px: 1.0,
        }
        .paint(
            scene,
            // Keep split handle under component focus rings (typically DrawOrder(1)),
            // while still painting above panel backgrounds (DrawOrder(0)).
            fret_core::DrawOrder(0),
            bounds,
            center,
            scale_factor,
            background,
        );
    }
}

fn paint_drop_overlay(
    theme: fret_ui::ThemeSnapshot,
    target: Option<DockDropTarget>,
    window: fret_core::AppWindowId,
    bounds: Rect,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    tab_scroll: &HashMap<DockNodeId, Px>,
    scene: &mut Scene,
) {
    let Some(target) = target else {
        return;
    };

    match target {
        DockDropTarget::Float { window: w } => {
            if w != window {
                return;
            }
            let zone = bounds;
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(10_000),
                rect: zone,
                background: Color {
                    a: 0.10,
                    ..theme.colors.accent
                },
                border: Edges::all(Px(3.0)),
                border_color: Color {
                    a: 0.85,
                    ..theme.colors.accent
                },
                corner_radii: fret_core::Corners::all(Px(theme.metrics.radius_md.0.max(6.0))),
            });
        }
        DockDropTarget::Dock(target) => {
            let Some(rect) = layout.get(&target.tabs).copied() else {
                return;
            };

            if target.zone == DropZone::Center {
                let (tab_bar, _content) = split_tab_bar(rect);
                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(9_990),
                    rect: tab_bar,
                    background: Color {
                        a: 0.14,
                        ..theme.colors.accent
                    },
                    border: Edges::all(Px(1.0)),
                    border_color: Color {
                        a: 0.45,
                        ..theme.colors.accent
                    },
                    corner_radii: fret_core::Corners::all(Px(theme.metrics.radius_sm.0.max(4.0))),
                });
                if let Some(i) = target.insert_index {
                    let scroll = tab_scroll_for_node(tab_scroll, target.tabs);
                    let x = tab_bar.origin.x.0 + DOCK_TAB_W.0 * i as f32 - scroll.0;
                    let marker = Rect::new(
                        Point::new(Px(x - 3.0), Px(tab_bar.origin.y.0 + 3.0)),
                        Size::new(Px(6.0), Px((tab_bar.size.height.0 - 6.0).max(0.0))),
                    );
                    scene.push(SceneOp::Quad {
                        order: fret_core::DrawOrder(10_000),
                        rect: marker,
                        background: Color {
                            a: 0.85,
                            ..theme.colors.accent
                        },
                        border: Edges::all(Px(1.0)),
                        border_color: Color {
                            a: 1.0,
                            ..theme.colors.accent
                        },
                        corner_radii: fret_core::Corners::all(Px(3.0)),
                    });

                    let cap_w = Px(14.0);
                    let cap_h = Px(3.0);
                    let cap_x = Px(x - cap_w.0 * 0.5);
                    let cap_top =
                        Rect::new(Point::new(cap_x, marker.origin.y), Size::new(cap_w, cap_h));
                    let cap_bottom = Rect::new(
                        Point::new(
                            cap_x,
                            Px(marker.origin.y.0 + marker.size.height.0 - cap_h.0),
                        ),
                        Size::new(cap_w, cap_h),
                    );
                    for cap in [cap_top, cap_bottom] {
                        scene.push(SceneOp::Quad {
                            order: fret_core::DrawOrder(10_001),
                            rect: cap,
                            background: Color {
                                a: 0.92,
                                ..theme.colors.accent
                            },
                            border: Edges::all(Px(0.0)),
                            border_color: Color::TRANSPARENT,
                            corner_radii: fret_core::Corners::all(Px(2.0)),
                        });
                    }
                }
                return;
            }

            let overlay = drop_zone_rect(rect, target.zone);
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(10_000),
                rect: overlay,
                background: Color {
                    a: 0.16,
                    ..theme.colors.accent
                },
                border: Edges::all(Px(2.0)),
                border_color: Color {
                    a: 0.85,
                    ..theme.colors.accent
                },
                corner_radii: fret_core::Corners::all(Px(theme.metrics.radius_sm.0.max(4.0))),
            });
        }
    }
}

fn paint_drop_hints(
    theme: fret_ui::ThemeSnapshot,
    target: Option<DockDropTarget>,
    _window: fret_core::AppWindowId,
    _bounds: Rect,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    scene: &mut Scene,
) {
    let Some(target) = target else {
        return;
    };

    let DockDropTarget::Dock(target) = target else {
        return;
    };

    let Some(rect) = layout.get(&target.tabs).copied() else {
        return;
    };

    let hint_rects = dock_hint_rects(rect);

    let inactive_bg = Color {
        a: 0.64,
        ..theme.colors.panel_background
    };
    let inactive_border = Color {
        a: 0.95,
        ..theme.colors.panel_border
    };
    let active_bg = Color {
        a: 0.92,
        ..theme.colors.accent
    };
    let active_border = Color {
        a: 1.0,
        ..theme.colors.accent
    };

    let order = fret_core::DrawOrder(9_500);
    let border = Edges::all(Px(2.0));
    let corner_radii = fret_core::Corners::all(Px(theme.metrics.radius_sm.0.max(4.0)));

    // Draw a plate behind the 5-way pad, closer to ImGui/Godot affordances.
    let pad = Px(theme.metrics.padding_sm.0.max(6.0));
    let mut min_x: f32 = f32::INFINITY;
    let mut min_y: f32 = f32::INFINITY;
    let mut max_x: f32 = f32::NEG_INFINITY;
    let mut max_y: f32 = f32::NEG_INFINITY;
    for &(_zone, r) in hint_rects.iter() {
        min_x = min_x.min(r.origin.x.0);
        min_y = min_y.min(r.origin.y.0);
        max_x = max_x.max(r.origin.x.0 + r.size.width.0);
        max_y = max_y.max(r.origin.y.0 + r.size.height.0);
    }
    if min_x.is_finite() && min_y.is_finite() && max_x.is_finite() && max_y.is_finite() {
        let plate = Rect::new(
            Point::new(Px(min_x - pad.0), Px(min_y - pad.0)),
            Size::new(
                Px((max_x - min_x + pad.0 * 2.0).max(0.0)),
                Px((max_y - min_y + pad.0 * 2.0).max(0.0)),
            ),
        );
        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(order.0 - 2),
            rect: plate,
            background: Color {
                a: 0.70,
                ..theme.colors.surface_background
            },
            border: Edges::all(Px(1.0)),
            border_color: Color {
                a: 0.70,
                ..theme.colors.panel_border
            },
            corner_radii: fret_core::Corners::all(Px(theme.metrics.radius_md.0.max(6.0))),
        });
    }

    for &(zone, hint_rect) in hint_rects.iter() {
        let is_active = zone == target.zone;
        let bg = if is_active { active_bg } else { inactive_bg };
        let stroke = if is_active {
            active_border
        } else {
            inactive_border
        };

        scene.push(SceneOp::Quad {
            order,
            rect: hint_rect,
            background: bg,
            border,
            border_color: stroke,
            corner_radii,
        });
        paint_drop_hint_icon(theme, zone, hint_rect, is_active, scene, order.0 + 1);
    }
}

fn paint_drop_hint_icon(
    theme: fret_ui::ThemeSnapshot,
    zone: DropZone,
    hint_rect: Rect,
    is_active: bool,
    scene: &mut Scene,
    order: u32,
) {
    fn inset(rect: Rect, inset: Px) -> Rect {
        let w = (rect.size.width.0 - inset.0 * 2.0).max(0.0);
        let h = (rect.size.height.0 - inset.0 * 2.0).max(0.0);
        Rect::new(
            Point::new(Px(rect.origin.x.0 + inset.0), Px(rect.origin.y.0 + inset.0)),
            Size::new(Px(w), Px(h)),
        )
    }

    let min_dim = hint_rect.size.width.0.min(hint_rect.size.height.0);
    let pad = Px((min_dim * 0.18).clamp(6.0, 10.0));
    let frame = inset(hint_rect, pad);
    let inner = inset(frame, Px((min_dim * 0.08).clamp(2.0, 4.0)));

    let stroke = Color {
        a: if is_active { 0.92 } else { 0.80 },
        ..theme.colors.text_primary
    };
    let base = Color {
        a: if is_active { 0.16 } else { 0.12 },
        ..theme.colors.text_primary
    };
    let fill = Color {
        a: if is_active { 0.90 } else { 0.72 },
        ..theme.colors.text_primary
    };

    let frame_radius = Px(theme.metrics.radius_sm.0.clamp(2.0, 4.0));
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(order),
        rect: frame,
        background: Color::TRANSPARENT,
        border: Edges::all(Px(2.0)),
        border_color: stroke,
        corner_radii: fret_core::Corners::all(frame_radius),
    });

    // Base fill so the highlighted region reads as "target placement" (ImGui-like).
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(order + 1),
        rect: inner,
        background: base,
        border: Edges::all(Px(0.0)),
        border_color: Color::TRANSPARENT,
        corner_radii: fret_core::Corners::all(Px(0.0)),
    });

    let split_ratio = 0.42_f32;
    let tab_ratio = 0.24_f32;
    let line_thickness = Px((min_dim * 0.04).clamp(1.5, 2.5));

    match zone {
        DropZone::Center => {
            let tab_h = Px((inner.size.height.0 * tab_ratio).max(0.0));
            let tab = Rect::new(inner.origin, Size::new(inner.size.width, tab_h));
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(order + 2),
                rect: tab,
                background: fill,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        }
        DropZone::Left | DropZone::Right => {
            let w = Px((inner.size.width.0 * split_ratio).max(0.0));
            let (highlight, line_x) = if zone == DropZone::Left {
                (
                    Rect::new(inner.origin, Size::new(w, inner.size.height)),
                    Px(inner.origin.x.0 + w.0),
                )
            } else {
                (
                    Rect::new(
                        Point::new(Px(inner.origin.x.0 + inner.size.width.0 - w.0), inner.origin.y),
                        Size::new(w, inner.size.height),
                    ),
                    Px(inner.origin.x.0 + inner.size.width.0 - w.0),
                )
            };
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(order + 2),
                rect: highlight,
                background: fill,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
            let line = Rect::new(
                Point::new(Px(line_x.0 - line_thickness.0 * 0.5), inner.origin.y),
                Size::new(line_thickness, inner.size.height),
            );
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(order + 3),
                rect: line,
                background: stroke,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        }
        DropZone::Top | DropZone::Bottom => {
            let h = Px((inner.size.height.0 * split_ratio).max(0.0));
            let (highlight, line_y) = if zone == DropZone::Top {
                (
                    Rect::new(inner.origin, Size::new(inner.size.width, h)),
                    Px(inner.origin.y.0 + h.0),
                )
            } else {
                (
                    Rect::new(
                        Point::new(inner.origin.x, Px(inner.origin.y.0 + inner.size.height.0 - h.0)),
                        Size::new(inner.size.width, h),
                    ),
                    Px(inner.origin.y.0 + inner.size.height.0 - h.0),
                )
            };
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(order + 2),
                rect: highlight,
                background: fill,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
            let line = Rect::new(
                Point::new(inner.origin.x, Px(line_y.0 - line_thickness.0 * 0.5)),
                Size::new(inner.size.width, line_thickness),
            );
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(order + 3),
                rect: line,
                background: stroke,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        }
    }
}

fn dock_hint_rects(rect: Rect) -> [(DropZone, Rect); 5] {
    // Match the mental model of ImGui docking: an explicit 5-way “direction pad” near the
    // center of the hovered dock node. Hit-testing uses the same rects.
    let cx = rect.origin.x.0 + rect.size.width.0 * 0.5;
    let cy = rect.origin.y.0 + rect.size.height.0 * 0.5;

    let min_dim = rect.size.width.0.min(rect.size.height.0);
    // Scale targets up on larger panels to make split docking feel effortless (Unity/ImGui-like),
    // while keeping it usable on small panels.
    let size = Px((min_dim * 0.095).clamp(34.0, 56.0));
    let gap = Px((size.0 * 0.35).clamp(10.0, 16.0));
    let step = Px(size.0 + gap.0);

    let mk = |dx: f32, dy: f32| -> Rect {
        Rect::new(
            Point::new(Px(cx + dx - size.0 * 0.5), Px(cy + dy - size.0 * 0.5)),
            Size::new(size, size),
        )
    };

    [
        (DropZone::Center, mk(0.0, 0.0)),
        (DropZone::Left, mk(-step.0, 0.0)),
        (DropZone::Right, mk(step.0, 0.0)),
        (DropZone::Top, mk(0.0, -step.0)),
        (DropZone::Bottom, mk(0.0, step.0)),
    ]
}

fn dock_space_regions(bounds: Rect) -> (Rect, Rect) {
    let chrome_h = Px(0.0);
    let chrome = Rect {
        origin: bounds.origin,
        size: Size::new(bounds.size.width, Px(chrome_h.0.min(bounds.size.height.0))),
    };
    let dock = Rect {
        origin: Point::new(
            bounds.origin.x,
            Px(bounds.origin.y.0 + chrome.size.height.0),
        ),
        size: Size::new(
            bounds.size.width,
            Px((bounds.size.height.0 - chrome.size.height.0).max(0.0)),
        ),
    };
    (chrome, dock)
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
