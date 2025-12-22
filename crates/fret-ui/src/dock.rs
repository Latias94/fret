use fret_app::{CommandId, DragKind, Effect, InputContext, Menu, MenuItem};
use fret_core::{
    Color, DockGraph, DockNode, DockNodeId, DockOp, DropZone, Edges, NodeId, PanelKey,
    RenderTargetId, Scene, SceneOp, TextBlobId, TextConstraints, TextMetrics, TextService,
    TextStyle, TextWrap, ViewportFit, ViewportInputEvent, ViewportInputKind, ViewportMapping,
    WindowAnchor,
    geometry::{Point, Px, Rect, Size},
};
use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::{
    widget::{EventCx, LayoutCx, PaintCx, Widget},
    widgets::{ContextMenuRequest, ContextMenuService},
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

#[derive(Debug, Clone)]
struct DockPanelDragPayload {
    panel: PanelKey,
}

#[derive(Debug, Clone, PartialEq)]
enum DockDropTarget {
    Dock(HoverTarget),
    Float { window: fret_core::AppWindowId },
}

#[derive(Debug, Clone)]
struct DockTabContextMenu {
    window: fret_core::AppWindowId,
    tabs: DockNodeId,
    panel: PanelKey,
    position: Point,
}

#[derive(Debug, Clone, PartialEq)]
struct ViewportHover {
    window: fret_core::AppWindowId,
    panel: PanelKey,
    position: Point,
}

pub struct DockManager {
    pub graph: DockGraph,
    pub panels: HashMap<PanelKey, DockPanel>,
    hover: Option<DockDropTarget>,
    dock_tab_context_menu: Option<DockTabContextMenu>,
    viewport_hover: Option<ViewportHover>,
    viewport_context_menu: Option<ViewportInputEvent>,
    viewport_overlays: HashMap<(fret_core::AppWindowId, RenderTargetId), ViewportOverlay>,
    viewport_content_rects: HashMap<(fret_core::AppWindowId, RenderTargetId), Rect>,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportOverlay {
    pub marquee: Option<ViewportMarquee>,
    pub drag_line: Option<ViewportDragLine>,
    pub selection_rect: Option<ViewportSelectionRect>,
    pub gizmo: Option<ViewportGizmo>,
    pub rotate_gizmo: Option<ViewportRotateGizmo>,
    pub marker: Option<ViewportMarker>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportMarquee {
    pub a_uv: (f32, f32),
    pub b_uv: (f32, f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportSelectionRect {
    pub min_uv: (f32, f32),
    pub max_uv: (f32, f32),
    pub fill: Color,
    pub stroke: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportGizmoPart {
    X,
    Y,
    Handle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportGizmo {
    pub center_uv: (f32, f32),
    pub axis_len_px: Px,
    pub highlight: Option<ViewportGizmoPart>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportRotateGizmo {
    pub center_uv: (f32, f32),
    pub radius_px: Px,
    pub highlight: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportMarker {
    pub uv: (f32, f32),
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportDragLine {
    pub a_uv: (f32, f32),
    pub b_uv: (f32, f32),
    pub color: Color,
}

impl Default for DockManager {
    fn default() -> Self {
        Self {
            graph: DockGraph::new(),
            panels: HashMap::new(),
            hover: None,
            dock_tab_context_menu: None,
            viewport_hover: None,
            viewport_context_menu: None,
            viewport_overlays: HashMap::new(),
            viewport_content_rects: HashMap::new(),
        }
    }
}

impl DockManager {
    pub fn take_dock_tab_context_menu(
        &mut self,
    ) -> Option<(fret_core::AppWindowId, DockNodeId, PanelKey, Point)> {
        self.dock_tab_context_menu
            .take()
            .map(|m| (m.window, m.tabs, m.panel, m.position))
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

    fn upsert_viewport_overlay(
        &mut self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
        update: impl FnOnce(&mut ViewportOverlay),
    ) {
        let key = (window, target);
        let mut overlay = self
            .viewport_overlays
            .get(&key)
            .copied()
            .unwrap_or(ViewportOverlay {
                marquee: None,
                drag_line: None,
                selection_rect: None,
                gizmo: None,
                rotate_gizmo: None,
                marker: None,
            });
        update(&mut overlay);
        if overlay.marquee.is_none()
            && overlay.drag_line.is_none()
            && overlay.selection_rect.is_none()
            && overlay.gizmo.is_none()
            && overlay.rotate_gizmo.is_none()
            && overlay.marker.is_none()
        {
            self.viewport_overlays.remove(&key);
        } else {
            self.viewport_overlays.insert(key, overlay);
        }
    }

    pub fn set_viewport_overlay(
        &mut self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
        overlay: Option<ViewportOverlay>,
    ) {
        let key = (window, target);
        if let Some(overlay) = overlay {
            self.viewport_overlays.insert(key, overlay);
        } else {
            self.viewport_overlays.remove(&key);
        }
    }

    pub fn set_viewport_marquee(
        &mut self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
        marquee: Option<ViewportMarquee>,
    ) {
        self.upsert_viewport_overlay(window, target, |o| o.marquee = marquee);
    }

    pub fn set_viewport_drag_line(
        &mut self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
        line: Option<ViewportDragLine>,
    ) {
        self.upsert_viewport_overlay(window, target, |o| o.drag_line = line);
    }

    pub fn set_viewport_selection_rect(
        &mut self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
        rect: Option<ViewportSelectionRect>,
    ) {
        self.upsert_viewport_overlay(window, target, |o| o.selection_rect = rect);
    }

    pub fn set_viewport_gizmo(
        &mut self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
        gizmo: Option<ViewportGizmo>,
    ) {
        self.upsert_viewport_overlay(window, target, |o| o.gizmo = gizmo);
    }

    pub fn set_viewport_rotate_gizmo(
        &mut self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
        gizmo: Option<ViewportRotateGizmo>,
    ) {
        self.upsert_viewport_overlay(window, target, |o| o.rotate_gizmo = gizmo);
    }

    pub fn set_viewport_marker(
        &mut self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
        marker: Option<ViewportMarker>,
    ) {
        self.upsert_viewport_overlay(window, target, |o| o.marker = marker);
    }
}

#[derive(Debug, Clone, Copy)]
struct DividerDragState {
    split: DockNodeId,
    axis: fret_core::Axis,
    bounds: Rect,
    fraction: f32,
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

impl DockSpace {
    pub fn new(window: fret_core::AppWindowId) -> Self {
        Self {
            window,
            last_bounds: Rect::default(),
            divider_drag: None,
            panel_content: HashMap::new(),
            panel_last_sizes: HashMap::new(),
            viewport_capture: None,
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
            },
            tab_close_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
            },
            empty_state_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
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

    fn panel_nodes(&self, app: &fret_app::App) -> HashMap<PanelKey, NodeId> {
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
        text: &mut dyn TextService,
        theme: crate::ThemeSnapshot,
        scale_factor: f32,
        dock: &DockManager,
        layout: &std::collections::HashMap<DockNodeId, Rect>,
    ) {
        let mut visible_set: HashSet<PanelKey> = HashSet::new();
        for (&node_id, _rect) in layout {
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
            text.release(title.blob);
        }
        if let Some(glyph) = self.tab_close_glyph.take() {
            text.release(glyph.blob);
        }

        let pad_x = theme.metrics.padding_md;
        let reserve = Px(DOCK_TAB_CLOSE_SIZE.0 + DOCK_TAB_CLOSE_GAP.0);
        let inner_max_w = Px((DOCK_TAB_W.0 - pad_x.0 * 2.0 - reserve.0).max(0.0));
        let constraints = TextConstraints {
            max_width: Some(inner_max_w),
            wrap: TextWrap::None,
            scale_factor,
        };

        let (close_blob, close_metrics) = text.prepare(
            "×",
            self.tab_close_style,
            TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
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
            let (blob, metrics) = text.prepare(title, self.tab_text_style, constraints);
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
        text: &mut dyn TextService,
        theme: crate::ThemeSnapshot,
        scale_factor: f32,
        max_width: Px,
    ) {
        if self.last_empty_state_theme_revision == Some(theme.revision)
            && self.last_empty_state_scale_factor == Some(scale_factor)
        {
            return;
        }
        self.last_empty_state_theme_revision = Some(theme.revision);
        self.last_empty_state_scale_factor = Some(scale_factor);

        if let Some(prev) = self.empty_state.take() {
            text.release(prev.blob);
        }

        let constraints = TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::Word,
            scale_factor,
        };
        let (blob, metrics) = text.prepare(
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

    fn paint_empty_state(&mut self, cx: &mut PaintCx<'_>) {
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
        self.rebuild_empty_state(cx.text, theme, cx.scale_factor, max_w);

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

impl Widget for DockSpace {
    fn event(&mut self, cx: &mut EventCx<'_>, event: &fret_core::Event) {
        let theme = cx.theme().snapshot();

        let mut pending_effects: Vec<Effect> = Vec::new();
        let mut pending_redraws: Vec<fret_core::AppWindowId> = Vec::new();
        let mut invalidate_paint = false;
        let mut invalidate_layout = false;
        let mut open_dock_tab_menu: Option<Point> = None;
        let mut open_viewport_menu: Option<(Point, ViewportInputEvent)> = None;
        let mut request_focus: Option<NodeId> = None;
        let mut request_focus_panel: Option<PanelKey> = None;
        let mut request_pointer_capture: Option<Option<NodeId>> = None;

        #[derive(Clone)]
        struct DockDragSnapshot {
            source_window: fret_core::AppWindowId,
            start: Point,
            dragging: bool,
            panel: PanelKey,
        }

        let allow_viewport_hover = cx.app.drag().map_or(true, |d| !d.dragging);
        let dock_drag = cx.app.drag().and_then(|d| {
            d.payload::<DockPanelDragPayload>()
                .map(|p| DockDragSnapshot {
                    source_window: d.source_window,
                    start: d.start,
                    dragging: d.dragging,
                    panel: p.panel.clone(),
                })
        });

        let mut begin_drag: Option<(Point, PanelKey)> = None;
        let mut update_drag: Option<(Point, bool)> = None;
        let mut end_dock_drag = false;

        {
            let Some(dock) = cx.app.global_mut::<DockManager>() else {
                return;
            };
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
                                cx.invalidate(cx.node, crate::widget::Invalidation::Paint);
                                return;
                            }
                            if let Some((tabs_node, tab_index, panel_key, close)) = hit_test_tab(
                                &dock.graph,
                                &layout,
                                &self.tab_scroll,
                                theme,
                                *position,
                            ) {
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
                                    begin_drag = Some((*position, panel_key));
                                    dock.hover = None;
                                    invalidate_paint = true;
                                    handled = true;
                                }
                            }
                        }

                        if !handled && *button == fret_core::MouseButton::Right {
                            if let Some((tabs_node, tab_index, panel_key, _close)) = hit_test_tab(
                                &dock.graph,
                                &layout,
                                &self.tab_scroll,
                                theme,
                                *position,
                            ) {
                                pending_effects.push(Effect::Dock(DockOp::SetActiveTab {
                                    tabs: tabs_node,
                                    active: tab_index,
                                }));
                                request_focus_panel = Some(panel_key.clone());
                                invalidate_layout = true;
                                dock.dock_tab_context_menu = Some(DockTabContextMenu {
                                    window: self.window,
                                    tabs: tabs_node,
                                    panel: panel_key,
                                    position: *position,
                                });
                                dock.hover = None;
                                invalidate_paint = true;
                                open_dock_tab_menu = Some(*position);
                                handled = true;
                            }
                        }

                        if !handled {
                            if let Some(hit) = hit_test_active_viewport_panel(
                                &dock.graph,
                                &dock.panels,
                                &layout,
                                *position,
                            ) {
                                if *button == fret_core::MouseButton::Left
                                    || *button == fret_core::MouseButton::Right
                                    || *button == fret_core::MouseButton::Middle
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
                                        if *button == fret_core::MouseButton::Right
                                            && hit.viewport.context_menu_enabled
                                        {
                                            dock.viewport_context_menu = Some(e);
                                        }
                                        pending_effects.push(Effect::ViewportInput(e));
                                        pending_redraws.push(self.window);
                                    }

                                    self.viewport_capture = Some(ViewportCaptureState {
                                        hit,
                                        button: *button,
                                        start: *position,
                                        moved: false,
                                        open_context_menu_on_up: *button
                                            == fret_core::MouseButton::Right,
                                    });
                                    request_pointer_capture = Some(Some(cx.node));
                                }
                            }
                        }
                    }
                    fret_core::PointerEvent::Move {
                        position,
                        buttons,
                        modifiers,
                    } => {
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
                            let (_chrome, dock_bounds) = dock_space_regions(self.last_bounds);
                            let layout = compute_layout_map(&dock.graph, root, dock_bounds);
                            if let Some((left, right)) =
                                split_children_two(&dock.graph, divider.split).and_then(|(a, b)| {
                                    Some((layout.get(&a).copied()?, layout.get(&b).copied()?))
                                })
                            {
                                if let Some(f0) = compute_split_fraction(
                                    divider.axis,
                                    divider.bounds,
                                    left,
                                    right,
                                    *position,
                                ) {
                                    dock.graph.update_split_two(divider.split, f0);
                                    divider.fraction = f0;
                                    self.divider_drag = Some(divider);
                                    cx.invalidate(cx.node, crate::widget::Invalidation::Layout);
                                    cx.invalidate(cx.node, crate::widget::Invalidation::Paint);
                                }
                            }
                            return;
                        }

                        if let Some(capture) = self.viewport_capture.as_mut() {
                            if capture.open_context_menu_on_up
                                && !capture.moved
                                && capture.button == fret_core::MouseButton::Right
                                && buttons.right
                            {
                                let dx = position.x.0 - capture.start.x.0;
                                let dy = position.y.0 - capture.start.y.0;
                                let dist2 = dx * dx + dy * dy;
                                if dist2 > 16.0 {
                                    capture.moved = true;
                                }
                            }

                            let hit = capture.hit.clone();
                            dock.viewport_hover = Some(ViewportHover {
                                window: self.window,
                                panel: hit.panel.clone(),
                                position: *position,
                            });
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

                                let next_hover = hit.as_ref().map(|hit| ViewportHover {
                                    window: self.window,
                                    panel: hit.panel.clone(),
                                    position: *position,
                                });
                                if dock.viewport_hover != next_hover {
                                    dock.viewport_hover = next_hover;
                                    pending_redraws.push(self.window);
                                } else if next_hover.is_some() {
                                    dock.viewport_hover = next_hover;
                                    pending_redraws.push(self.window);
                                }

                                if let Some(hit) = hit {
                                    if let Some(e) = viewport_input_from_hit(
                                        self.window,
                                        hit,
                                        *position,
                                        ViewportInputKind::PointerMove {
                                            buttons: *buttons,
                                            modifiers: *modifiers,
                                        },
                                    ) {
                                        pending_effects.push(Effect::ViewportInput(e));
                                    }
                                }
                            } else if dock
                                .viewport_hover
                                .as_ref()
                                .is_some_and(|h| h.window == self.window)
                            {
                                dock.viewport_hover = None;
                                pending_redraws.push(self.window);
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

                        if !scrolled_tabs {
                            if let Some(hit) = hit_test_active_viewport_panel(
                                &dock.graph,
                                &dock.panels,
                                &layout,
                                *position,
                            ) {
                                if let Some(e) = viewport_input_from_hit(
                                    self.window,
                                    hit,
                                    *position,
                                    ViewportInputKind::Wheel {
                                        delta: *delta,
                                        modifiers: *modifiers,
                                    },
                                ) {
                                    pending_effects.push(Effect::ViewportInput(e));
                                    pending_redraws.push(self.window);
                                }
                            }
                        }
                    }
                    fret_core::PointerEvent::Up {
                        position,
                        button,
                        modifiers,
                    } => {
                        let mut handled = false;
                        if *button == fret_core::MouseButton::Left {
                            if let Some((tabs_node, tab_index, panel_key)) =
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
                                if *button == fret_core::MouseButton::Right {
                                    if capture.hit.viewport.context_menu_enabled {
                                        dock.viewport_context_menu = Some(e);
                                    }
                                }
                                pending_effects.push(Effect::ViewportInput(e));
                                pending_redraws.push(self.window);

                                if capture.open_context_menu_on_up
                                    && !capture.moved
                                    && *button == fret_core::MouseButton::Right
                                    && capture.hit.viewport.context_menu_enabled
                                {
                                    open_viewport_menu = Some((*position, e));
                                }

                                dock.hover = None;
                                dock.viewport_hover = None;
                                request_pointer_capture = Some(None);
                                invalidate_paint = true;
                            }

                            if !released_capture && *button == fret_core::MouseButton::Left {
                                if let Some(divider) = self.divider_drag.take() {
                                    pending_effects.push(Effect::Dock(
                                        DockOp::SetSplitFractionTwo {
                                            split: divider.split,
                                            first_fraction: divider.fraction,
                                        },
                                    ));
                                    invalidate_layout = true;
                                }
                            }

                            if !released_capture
                                && *button == fret_core::MouseButton::Left
                                && dock_drag.is_some()
                            {
                                let drag = dock_drag.unwrap();

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
                                            pending_effects.push(Effect::Dock(
                                                DockOp::RequestFloatPanelToNewWindow {
                                                    source_window: drag.source_window,
                                                    panel: drag.panel.clone(),
                                                    anchor: Some(fret_core::WindowAnchor {
                                                        window: self.window,
                                                        position: *position,
                                                    }),
                                                },
                                            ));
                                            invalidate_layout = true;
                                        }
                                        None => {
                                            if float_zone(self.last_bounds).contains(*position) {
                                                pending_effects.push(Effect::Dock(
                                                    DockOp::RequestFloatPanelToNewWindow {
                                                        source_window: drag.source_window,
                                                        panel: drag.panel.clone(),
                                                        anchor: Some(fret_core::WindowAnchor {
                                                            window: self.window,
                                                            position: *position,
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
                                    ) {
                                        if let Some(e) = viewport_input_from_hit(
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
                                    if !dragging && dist2 > 16.0 {
                                        dragging = true;
                                    }
                                } else if !dragging {
                                    dragging = true;
                                }

                                update_drag = Some((position, dragging));

                                if dragging {
                                    let bounds = self.last_bounds;
                                    if float_zone(bounds).contains(position) {
                                        dock.hover = Some(DockDropTarget::Float {
                                            window: self.window,
                                        });
                                    } else if bounds.contains(position) {
                                        let layout = compute_layout_map(&dock.graph, root, bounds);
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
                                if drag.source_window != self.window && !dragging {
                                    dragging = true;
                                }

                                if dragging {
                                    let bounds = self.last_bounds;
                                    if float_zone(bounds).contains(position) {
                                        dock.hover = Some(DockDropTarget::Float {
                                            window: self.window,
                                        });
                                    } else if bounds.contains(position) {
                                        let layout = compute_layout_map(&dock.graph, root, bounds);
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
                                            pending_effects.push(Effect::Dock(
                                                DockOp::RequestFloatPanelToNewWindow {
                                                    source_window: drag.source_window,
                                                    panel: drag.panel.clone(),
                                                    anchor: Some(fret_core::WindowAnchor {
                                                        window: self.window,
                                                        position,
                                                    }),
                                                },
                                            ));
                                            invalidate_layout = true;
                                        }
                                        None => {
                                            if float_zone(self.last_bounds).contains(position) {
                                                pending_effects.push(Effect::Dock(
                                                    DockOp::RequestFloatPanelToNewWindow {
                                                        source_window: drag.source_window,
                                                        panel: drag.panel.clone(),
                                                        anchor: Some(fret_core::WindowAnchor {
                                                            window: self.window,
                                                            position,
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

        if request_focus.is_none() {
            if let Some(panel) = request_focus_panel {
                let panel_nodes = self.panel_nodes(cx.app);
                request_focus = panel_nodes.get(&panel).copied();
            }
        }

        if let Some((start, panel)) = begin_drag {
            cx.app.begin_cross_window_drag_with_kind(
                DragKind::DockPanel,
                self.window,
                start,
                DockPanelDragPayload { panel },
            );
        }

        if let Some(request) = request_pointer_capture {
            match request {
                Some(node) => cx.capture_pointer(node),
                None => cx.release_pointer_capture(),
            }
        }

        if let Some((position, dragging)) = update_drag {
            if let Some(drag) = cx.app.drag_mut() {
                if drag.payload::<DockPanelDragPayload>().is_some() {
                    drag.position = position;
                    drag.dragging = dragging;
                }
            }
        }

        if end_dock_drag {
            if cx
                .app
                .drag()
                .and_then(|d| d.payload::<DockPanelDragPayload>())
                .is_some()
            {
                cx.app.cancel_drag();
            }
        }

        if let Some(position) = open_dock_tab_menu {
            let Some(window) = cx.window else {
                return;
            };

            cx.request_focus(cx.node);

            let inv_ctx = InputContext {
                platform: cx.input_ctx.platform,
                ui_has_modal: cx.input_ctx.ui_has_modal,
                focus_is_text_input: cx.input_ctx.focus_is_text_input,
            };

            let menu = Menu {
                title: Arc::from("Dock"),
                items: vec![
                    MenuItem::Command {
                        command: CommandId::from("dock.tab.float"),
                        when: None,
                    },
                    MenuItem::Separator,
                    MenuItem::Command {
                        command: CommandId::from("dock.tab.move_left"),
                        when: None,
                    },
                    MenuItem::Command {
                        command: CommandId::from("dock.tab.move_right"),
                        when: None,
                    },
                    MenuItem::Separator,
                    MenuItem::Command {
                        command: CommandId::from("dock.tab.close"),
                        when: None,
                    },
                ],
            };

            cx.app
                .with_global_mut(ContextMenuService::default, |service, _app| {
                    service.set_request(
                        window,
                        ContextMenuRequest {
                            position,
                            menu,
                            input_ctx: inv_ctx,
                            menu_bar: None,
                        },
                    );
                });
            cx.dispatch_command(CommandId::from("context_menu.open"));
            cx.request_redraw();
            cx.stop_propagation();
            return;
        }

        if let Some((position, _viewport_event)) = open_viewport_menu {
            let Some(window) = cx.window else {
                return;
            };

            cx.request_focus(cx.node);

            let inv_ctx = InputContext {
                platform: cx.input_ctx.platform,
                ui_has_modal: cx.input_ctx.ui_has_modal,
                focus_is_text_input: cx.input_ctx.focus_is_text_input,
            };

            let menu = Menu {
                title: Arc::from("Viewport"),
                items: vec![
                    MenuItem::Command {
                        command: CommandId::from("viewport.copy_uv"),
                        when: None,
                    },
                    MenuItem::Command {
                        command: CommandId::from("viewport.copy_target_px"),
                        when: None,
                    },
                ],
            };

            cx.app
                .with_global_mut(ContextMenuService::default, |service, _app| {
                    service.set_request(
                        window,
                        ContextMenuRequest {
                            position,
                            menu,
                            input_ctx: inv_ctx,
                            menu_bar: None,
                        },
                    );
                });
            cx.dispatch_command(CommandId::from("context_menu.open"));
            cx.request_redraw();
            cx.stop_propagation();
            return;
        }

        if let Some(node) = request_focus {
            cx.request_focus(node);
        }
        if invalidate_layout {
            cx.invalidate(cx.node, crate::widget::Invalidation::Layout);
        }
        if invalidate_paint {
            cx.invalidate(cx.node, crate::widget::Invalidation::Paint);
        }

        for window in pending_redraws {
            cx.app.request_redraw(window);
        }
        for effect in pending_effects {
            cx.app.push_effect(effect);
        }
    }

    fn command(&mut self, cx: &mut crate::widget::CommandCx<'_>, command: &CommandId) -> bool {
        match command.as_str() {
            "dock.tab.float" => {
                let Some(dock) = cx.app.global_mut::<DockManager>() else {
                    return false;
                };
                let Some(ctx) = dock.dock_tab_context_menu.take() else {
                    return false;
                };
                cx.app
                    .push_effect(Effect::Dock(DockOp::RequestFloatPanelToNewWindow {
                        source_window: ctx.window,
                        panel: ctx.panel,
                        anchor: Some(WindowAnchor {
                            window: ctx.window,
                            position: ctx.position,
                        }),
                    }));
                cx.stop_propagation();
                true
            }
            "dock.tab.move_left" => {
                let Some(dock) = cx.app.global_mut::<DockManager>() else {
                    return false;
                };
                let Some(ctx) = dock.dock_tab_context_menu.take() else {
                    return false;
                };
                let Some(DockNode::Tabs { tabs, .. }) = dock.graph.node(ctx.tabs) else {
                    return false;
                };
                let source_index = tabs.iter().position(|p| *p == ctx.panel);
                let Some(source_index) = source_index else {
                    return false;
                };
                if source_index == 0 {
                    cx.stop_propagation();
                    return true;
                }
                cx.app.push_effect(Effect::Dock(DockOp::MovePanel {
                    source_window: ctx.window,
                    panel: ctx.panel,
                    target_window: ctx.window,
                    target_tabs: ctx.tabs,
                    zone: DropZone::Center,
                    insert_index: Some(source_index.saturating_sub(1)),
                }));
                cx.stop_propagation();
                true
            }
            "dock.tab.move_right" => {
                let Some(dock) = cx.app.global_mut::<DockManager>() else {
                    return false;
                };
                let Some(ctx) = dock.dock_tab_context_menu.take() else {
                    return false;
                };
                let Some(DockNode::Tabs { tabs, .. }) = dock.graph.node(ctx.tabs) else {
                    return false;
                };
                let source_index = tabs.iter().position(|p| *p == ctx.panel);
                let Some(source_index) = source_index else {
                    return false;
                };
                if source_index + 1 >= tabs.len() {
                    cx.stop_propagation();
                    return true;
                }
                cx.app.push_effect(Effect::Dock(DockOp::MovePanel {
                    source_window: ctx.window,
                    panel: ctx.panel,
                    target_window: ctx.window,
                    target_tabs: ctx.tabs,
                    zone: DropZone::Center,
                    insert_index: Some(source_index + 2),
                }));
                cx.stop_propagation();
                true
            }
            "dock.tab.close" => {
                let Some(dock) = cx.app.global_mut::<DockManager>() else {
                    return false;
                };
                let Some(ctx) = dock.dock_tab_context_menu.take() else {
                    return false;
                };
                cx.app.push_effect(Effect::Dock(DockOp::ClosePanel {
                    window: ctx.window,
                    panel: ctx.panel,
                }));
                cx.stop_propagation();
                true
            }
            "viewport.copy_uv" => {
                let Some(dock) = cx.app.global::<DockManager>() else {
                    return false;
                };
                let Some(e) = dock.viewport_context_menu else {
                    return false;
                };
                cx.app.push_effect(Effect::ClipboardSetText {
                    text: format!("{:.6}, {:.6}", e.uv.0, e.uv.1),
                });
                cx.stop_propagation();
                true
            }
            "viewport.copy_target_px" => {
                let Some(dock) = cx.app.global::<DockManager>() else {
                    return false;
                };
                let Some(e) = dock.viewport_context_menu else {
                    return false;
                };
                cx.app.push_effect(Effect::ClipboardSetText {
                    text: format!("{}, {}", e.target_px.0, e.target_px.1),
                });
                cx.stop_propagation();
                true
            }
            _ => false,
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        self.last_bounds = cx.bounds;
        let hidden = hidden_bounds(Size::new(Px(0.0), Px(0.0)));

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

        let theme = cx.theme().snapshot();
        let scale_factor = cx.scale_factor;
        if let Some(dock) = cx.app.global::<DockManager>() {
            self.rebuild_tab_titles(cx.text, theme, scale_factor, dock, &layout);

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

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
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

        if let Some(dock) = cx.app.global_mut::<DockManager>() {
            dock.clear_viewport_layout_for_window(self.window);
            for (&node_id, &rect) in layout.iter() {
                let (_tab_bar, content) = split_tab_bar(rect);
                let target = {
                    let mut out: Option<RenderTargetId> = None;
                    if let Some(DockNode::Tabs { tabs, active }) = dock.graph.node(node_id) {
                        if let Some(panel_key) = tabs.get(*active) {
                            if let Some(panel) = dock.panel(panel_key) {
                                if let Some(vp) = panel.viewport {
                                    out = Some(vp.target);
                                }
                            }
                        }
                    }
                    out
                };
                if let Some(target) = target {
                    dock.set_viewport_content_rect(self.window, target, content);
                }
            }
        }
        if let Some(dock) = cx.app.global::<DockManager>() {
            paint_dock(
                cx.theme().snapshot(),
                dock,
                self.window,
                &layout,
                &self.tab_titles,
                self.hovered_tab,
                self.hovered_tab_close,
                self.pressed_tab_close.as_ref().map(|(n, i, _)| (*n, *i)),
                &self.tab_scroll,
                self.tab_close_glyph,
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
            paint_split_handles(&dock.graph, &layout, cx.scene);
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
    graph.compute_layout(root, bounds, &mut layout);
    layout
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

fn paint_dock(
    theme: crate::ThemeSnapshot,
    dock: &DockManager,
    window: fret_core::AppWindowId,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    tab_titles: &HashMap<PanelKey, PreparedTabTitle>,
    hovered_tab: Option<(DockNodeId, usize)>,
    hovered_tab_close: bool,
    pressed_tab_close: Option<(DockNodeId, usize)>,
    tab_scroll: &HashMap<DockNodeId, Px>,
    tab_close_glyph: Option<PreparedTabTitle>,
    scene: &mut Scene,
) {
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
                if let Some(h) = dock.viewport_hover.as_ref() {
                    if h.window == window && active_panel.is_some_and(|p| p == &h.panel) {
                        paint_viewport_crosshair(theme, draw_rect, h.position, scene);
                    }
                }
                if let Some(overlay) = dock.viewport_overlays.get(&(window, vp.target)) {
                    paint_viewport_overlay(theme, draw_rect, *overlay, scene);
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

fn paint_viewport_crosshair(
    theme: crate::ThemeSnapshot,
    content: Rect,
    position: Point,
    scene: &mut Scene,
) {
    if !content.contains(position) {
        return;
    }

    let thickness = Px(1.5);
    let len = Px(12.0);
    let x = position.x;
    let y = position.y;

    let h = Rect {
        origin: Point::new(Px(x.0 - len.0), Px(y.0 - thickness.0 * 0.5)),
        size: Size::new(Px(len.0 * 2.0), thickness),
    };
    let v = Rect {
        origin: Point::new(Px(x.0 - thickness.0 * 0.5), Px(y.0 - len.0)),
        size: Size::new(thickness, Px(len.0 * 2.0)),
    };

    let color = Color {
        a: 0.65,
        ..theme.colors.text_primary
    };

    for rect in [h, v] {
        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(5),
            rect,
            background: color,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });
    }
}

fn paint_viewport_overlay(
    theme: crate::ThemeSnapshot,
    content: Rect,
    overlay: ViewportOverlay,
    scene: &mut Scene,
) {
    if let Some(sel) = overlay.selection_rect {
        paint_viewport_selection_rect(content, sel, scene);
    }
    if let Some(gizmo) = overlay.gizmo {
        paint_viewport_gizmo(theme, content, gizmo, scene);
    }
    if let Some(gizmo) = overlay.rotate_gizmo {
        paint_viewport_rotate_gizmo(theme, content, gizmo, scene);
    }
    if let Some(m) = overlay.marquee {
        paint_viewport_marquee(theme, content, m, scene);
    }
    if let Some(line) = overlay.drag_line {
        paint_viewport_drag_line(content, line, scene);
    }
    if let Some(marker) = overlay.marker {
        paint_viewport_marker(content, marker, scene);
    }
}

fn paint_viewport_gizmo(
    theme: crate::ThemeSnapshot,
    content: Rect,
    gizmo: ViewportGizmo,
    scene: &mut Scene,
) {
    let (u, v) = gizmo.center_uv;
    let x = content.origin.x.0 + content.size.width.0 * u;
    let y = content.origin.y.0 + content.size.height.0 * v;

    let len = gizmo.axis_len_px;
    let highlight = gizmo.highlight;
    let t = Px(2.5);
    let x_t = if highlight == Some(ViewportGizmoPart::X) {
        Px(4.0)
    } else {
        t
    };
    let y_t = if highlight == Some(ViewportGizmoPart::Y) {
        Px(4.0)
    } else {
        t
    };

    let x_axis = Rect::new(Point::new(Px(x), Px(y - x_t.0 * 0.5)), Size::new(len, x_t));
    let y_axis = Rect::new(
        Point::new(Px(x - y_t.0 * 0.5), Px(y - len.0)),
        Size::new(y_t, len),
    );

    let x_axis_alpha = if highlight == Some(ViewportGizmoPart::X) {
        1.0
    } else {
        0.85
    };
    let y_axis_alpha = if highlight == Some(ViewportGizmoPart::Y) {
        1.0
    } else {
        0.85
    };
    let x_color = Color {
        a: x_axis_alpha,
        ..theme.colors.viewport_gizmo_x
    };
    let y_color = Color {
        a: y_axis_alpha,
        ..theme.colors.viewport_gizmo_y
    };

    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(6),
        rect: x_axis,
        background: x_color,
        border: Edges::all(Px(0.0)),
        border_color: Color::TRANSPARENT,
        corner_radii: fret_core::Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(6),
        rect: y_axis,
        background: y_color,
        border: Edges::all(Px(0.0)),
        border_color: Color::TRANSPARENT,
        corner_radii: fret_core::Corners::all(Px(0.0)),
    });

    let handle = Px(10.0);
    let handle_highlight = highlight == Some(ViewportGizmoPart::Handle);
    let handle_border = if handle_highlight { Px(2.5) } else { Px(1.5) };
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(7),
        rect: Rect::new(
            Point::new(Px(x - handle.0 * 0.5), Px(y - handle.0 * 0.5)),
            Size::new(handle, handle),
        ),
        background: Color {
            a: 0.85,
            ..theme.colors.viewport_gizmo_handle_background
        },
        border: Edges::all(handle_border),
        border_color: Color {
            a: 0.90,
            ..theme.colors.viewport_gizmo_handle_border
        },
        corner_radii: fret_core::Corners::all(Px(2.0)),
    });
}

fn paint_viewport_rotate_gizmo(
    theme: crate::ThemeSnapshot,
    content: Rect,
    gizmo: ViewportRotateGizmo,
    scene: &mut Scene,
) {
    let (u, v) = gizmo.center_uv;
    let x = content.origin.x.0 + content.size.width.0 * u;
    let y = content.origin.y.0 + content.size.height.0 * v;

    let r = gizmo.radius_px;
    let t = if gizmo.highlight { Px(3.0) } else { Px(2.0) };
    let a = if gizmo.highlight { 0.95 } else { 0.75 };
    let color = Color {
        a,
        ..theme.colors.viewport_rotate_gizmo
    };

    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(6),
        rect: Rect::new(
            Point::new(Px(x - r.0), Px(y - r.0)),
            Size::new(Px(r.0 * 2.0), Px(r.0 * 2.0)),
        ),
        background: Color::TRANSPARENT,
        border: Edges::all(t),
        border_color: color,
        corner_radii: fret_core::Corners::all(r),
    });
}

fn paint_viewport_selection_rect(content: Rect, rect: ViewportSelectionRect, scene: &mut Scene) {
    let (u0, v0) = rect.min_uv;
    let (u1, v1) = rect.max_uv;
    let left = content.origin.x.0 + content.size.width.0 * u0.min(u1);
    let right = content.origin.x.0 + content.size.width.0 * u0.max(u1);
    let top = content.origin.y.0 + content.size.height.0 * v0.min(v1);
    let bottom = content.origin.y.0 + content.size.height.0 * v0.max(v1);

    let inner = Rect::new(
        Point::new(Px(left), Px(top)),
        Size::new(Px((right - left).max(0.0)), Px((bottom - top).max(0.0))),
    );
    if inner.size.width.0 <= 0.0 || inner.size.height.0 <= 0.0 {
        return;
    }

    let t = Px(2.0);
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(4),
        rect: inner,
        background: rect.fill,
        border: Edges::all(Px(0.0)),
        border_color: Color::TRANSPARENT,
        corner_radii: fret_core::Corners::all(Px(0.0)),
    });

    let top_rect = Rect::new(inner.origin, Size::new(inner.size.width, t));
    let bottom_rect = Rect::new(
        Point::new(
            inner.origin.x,
            Px(inner.origin.y.0 + inner.size.height.0 - t.0),
        ),
        Size::new(inner.size.width, t),
    );
    let left_rect = Rect::new(inner.origin, Size::new(t, inner.size.height));
    let right_rect = Rect::new(
        Point::new(
            Px(inner.origin.x.0 + inner.size.width.0 - t.0),
            inner.origin.y,
        ),
        Size::new(t, inner.size.height),
    );
    for r in [top_rect, bottom_rect, left_rect, right_rect] {
        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(5),
            rect: r,
            background: rect.stroke,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });
    }
}

fn paint_viewport_marker(content: Rect, marker: ViewportMarker, scene: &mut Scene) {
    let (u, v) = marker.uv;
    let x = content.origin.x.0 + content.size.width.0 * u;
    let y = content.origin.y.0 + content.size.height.0 * v;

    let t = Px(2.0);
    let len = Px(10.0);
    let color = marker.color;
    let shadow = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.35,
    };

    let h_shadow = Rect::new(
        Point::new(Px(x - len.0), Px(y - t.0 * 0.5 + 1.0)),
        Size::new(Px(len.0 * 2.0), t),
    );
    let v_shadow = Rect::new(
        Point::new(Px(x - t.0 * 0.5 + 1.0), Px(y - len.0)),
        Size::new(t, Px(len.0 * 2.0)),
    );
    for rect in [h_shadow, v_shadow] {
        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(10),
            rect,
            background: shadow,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });
    }

    let h = Rect::new(
        Point::new(Px(x - len.0), Px(y - t.0 * 0.5)),
        Size::new(Px(len.0 * 2.0), t),
    );
    let v = Rect::new(
        Point::new(Px(x - t.0 * 0.5), Px(y - len.0)),
        Size::new(t, Px(len.0 * 2.0)),
    );
    for rect in [h, v] {
        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(11),
            rect,
            background: color,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });
    }

    let p = Px(7.0);
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(12),
        rect: Rect::new(
            Point::new(Px(x - p.0 * 0.5), Px(y - p.0 * 0.5)),
            Size::new(p, p),
        ),
        background: Color {
            r: color.r,
            g: color.g,
            b: color.b,
            a: (color.a * 0.25).min(1.0),
        },
        border: Edges::all(Px(1.5)),
        border_color: color,
        corner_radii: fret_core::Corners::all(Px(2.0)),
    });
}

fn paint_viewport_marquee(
    theme: crate::ThemeSnapshot,
    content: Rect,
    marquee: ViewportMarquee,
    scene: &mut Scene,
) {
    let (au, av) = marquee.a_uv;
    let (bu, bv) = marquee.b_uv;
    let x0 = content.origin.x.0 + content.size.width.0 * au;
    let y0 = content.origin.y.0 + content.size.height.0 * av;
    let x1 = content.origin.x.0 + content.size.width.0 * bu;
    let y1 = content.origin.y.0 + content.size.height.0 * bv;

    let left = x0.min(x1);
    let right = x0.max(x1);
    let top = y0.min(y1);
    let bottom = y0.max(y1);

    let rect = Rect::new(
        Point::new(Px(left), Px(top)),
        Size::new(Px((right - left).max(0.0)), Px((bottom - top).max(0.0))),
    );
    // Render even for very thin drags (so users still see feedback); only skip true clicks.
    if rect.size.width.0 <= 1.0 && rect.size.height.0 <= 1.0 {
        return;
    }

    let fill = Color {
        a: 0.14,
        ..theme.colors.accent
    };
    let stroke = Color {
        a: 0.85,
        ..theme.colors.accent
    };
    let t = Px(1.5);

    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(6),
        rect,
        background: fill,
        border: Edges::all(Px(0.0)),
        border_color: Color::TRANSPARENT,
        corner_radii: fret_core::Corners::all(Px(0.0)),
    });

    let top_rect = Rect::new(rect.origin, Size::new(rect.size.width, t));
    let bottom_rect = Rect::new(
        Point::new(
            rect.origin.x,
            Px(rect.origin.y.0 + rect.size.height.0 - t.0),
        ),
        Size::new(rect.size.width, t),
    );
    let left_rect = Rect::new(rect.origin, Size::new(t, rect.size.height));
    let right_rect = Rect::new(
        Point::new(Px(rect.origin.x.0 + rect.size.width.0 - t.0), rect.origin.y),
        Size::new(t, rect.size.height),
    );

    for r in [top_rect, bottom_rect, left_rect, right_rect] {
        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(7),
            rect: r,
            background: stroke,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });
    }
}

fn paint_viewport_drag_line(content: Rect, line: ViewportDragLine, scene: &mut Scene) {
    let (au, av) = line.a_uv;
    let (bu, bv) = line.b_uv;
    let x0 = content.origin.x.0 + content.size.width.0 * au;
    let y0 = content.origin.y.0 + content.size.height.0 * av;
    let x1 = content.origin.x.0 + content.size.width.0 * bu;
    let y1 = content.origin.y.0 + content.size.height.0 * bv;

    let color = line.color;
    let t = Px(1.5);

    let h = Rect::new(
        Point::new(Px(x0.min(x1)), Px(y0 - t.0 * 0.5)),
        Size::new(Px((x1 - x0).abs().max(0.0)), t),
    );
    let v = Rect::new(
        Point::new(Px(x1 - t.0 * 0.5), Px(y0.min(y1))),
        Size::new(t, Px((y1 - y0).abs().max(0.0))),
    );

    for rect in [h, v] {
        if rect.size.width.0 <= 0.0 || rect.size.height.0 <= 0.0 {
            continue;
        }
        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(8),
            rect,
            background: color,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });
    }

    let p = Px(6.0);
    for (x, y) in [(x0, y0), (x1, y1)] {
        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(9),
            rect: Rect::new(
                Point::new(Px(x - p.0 * 0.5), Px(y - p.0 * 0.5)),
                Size::new(p, p),
            ),
            background: color,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(2.0)),
        });
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
    open_context_menu_on_up: bool,
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
    let base = (min_dim * 0.20).clamp(12.0, 64.0);
    let cap = (min_dim * 0.33).clamp(8.0, 64.0);
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

fn tab_close_rect(theme: crate::ThemeSnapshot, tab_rect: Rect) -> Rect {
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
    theme: crate::ThemeSnapshot,
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
    let thickness = Px(6.0);

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
        let Some(_right) = layout.get(&children[1]).copied() else {
            continue;
        };

        let handle = match axis {
            fret_core::Axis::Horizontal => {
                let x = left.origin.x.0 + left.size.width.0 - thickness.0 * 0.5;
                Rect {
                    origin: Point::new(Px(x), bounds.origin.y),
                    size: Size::new(thickness, bounds.size.height),
                }
            }
            fret_core::Axis::Vertical => {
                let y = left.origin.y.0 + left.size.height.0 - thickness.0 * 0.5;
                Rect {
                    origin: Point::new(bounds.origin.x, Px(y)),
                    size: Size::new(bounds.size.width, thickness),
                }
            }
        };

        if handle.contains(position) {
            let total = fractions.iter().take(2).sum::<f32>();
            let total = if total <= 0.0 { 1.0 } else { total };
            let f0 = fractions.get(0).copied().unwrap_or(0.5) / total;
            return Some(DividerDragState {
                split: node,
                axis: *axis,
                bounds,
                fraction: f0,
            });
        }
    }

    None
}

fn compute_split_fraction(
    axis: fret_core::Axis,
    bounds: Rect,
    _first: Rect,
    _second: Rect,
    position: Point,
) -> Option<f32> {
    let min_px = 120.0;
    match axis {
        fret_core::Axis::Horizontal => {
            let w = bounds.size.width.0;
            if w <= min_px * 2.0 {
                return None;
            }
            let x = (position.x.0 - bounds.origin.x.0).clamp(min_px, w - min_px);
            Some(x / w)
        }
        fret_core::Axis::Vertical => {
            let h = bounds.size.height.0;
            if h <= min_px * 2.0 {
                return None;
            }
            let y = (position.y.0 - bounds.origin.y.0).clamp(min_px, h - min_px);
            Some(y / h)
        }
    }
}

fn paint_split_handles(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    scene: &mut Scene,
) {
    let thickness = Px(4.0);
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

        let rect = match axis {
            fret_core::Axis::Horizontal => Rect {
                origin: Point::new(
                    Px(first.origin.x.0 + first.size.width.0 - thickness.0 * 0.5),
                    bounds.origin.y,
                ),
                size: Size::new(thickness, bounds.size.height),
            },
            fret_core::Axis::Vertical => Rect {
                origin: Point::new(
                    bounds.origin.x,
                    Px(first.origin.y.0 + first.size.height.0 - thickness.0 * 0.5),
                ),
                size: Size::new(bounds.size.width, thickness),
            },
        };

        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(9_000),
            rect,
            background: Color {
                r: 0.06,
                g: 0.06,
                b: 0.07,
                a: 1.0,
            },
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });
    }
}

fn paint_drop_overlay(
    theme: crate::ThemeSnapshot,
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
            let zone = float_zone(bounds);
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(10_000),
                rect: zone,
                background: Color {
                    a: 0.35,
                    ..theme.colors.accent
                },
                border: Edges::all(Px(2.0)),
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
    theme: crate::ThemeSnapshot,
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

    let cx = rect.origin.x.0 + rect.size.width.0 * 0.5;
    let cy = rect.origin.y.0 + rect.size.height.0 * 0.5;

    let size = Px(26.0);
    let gap = Px(8.0);
    let step = Px(size.0 + gap.0);

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

    for (zone, dx, dy) in [
        (DropZone::Center, 0.0, 0.0),
        (DropZone::Left, -(step.0), 0.0),
        (DropZone::Right, step.0, 0.0),
        (DropZone::Top, 0.0, -(step.0)),
        (DropZone::Bottom, 0.0, step.0),
    ] {
        let is_active = zone == target.zone;
        let bg = if is_active { active_bg } else { inactive_bg };
        let stroke = if is_active {
            active_border
        } else {
            inactive_border
        };

        scene.push(SceneOp::Quad {
            order,
            rect: Rect::new(
                Point::new(Px(cx + dx - size.0 * 0.5), Px(cy + dy - size.0 * 0.5)),
                Size::new(size, size),
            ),
            background: bg,
            border,
            border_color: stroke,
            corner_radii,
        });
    }
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
    use fret_app::App;
    use fret_core::{
        AppWindowId, Event, InternalDragEvent, InternalDragKind, Scene, SceneOp, TextConstraints,
        TextMetrics, TextService, TextStyle,
    };

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

    #[test]
    fn dock_space_paints_empty_state_when_no_window_root() {
        let mut ui = crate::UiTree::new();
        ui.set_window(AppWindowId::default());

        let root = ui.create_node(DockSpace::new(AppWindowId::default()));
        ui.set_root(root);

        let mut app = App::new();
        let mut text = FakeTextService::default();

        let size = Size::new(Px(800.0), Px(600.0));
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);

        let _ = ui.layout(&mut app, &mut text, root, size, 1.0);
        let mut scene = Scene::default();
        ui.paint(&mut app, &mut text, root, bounds, &mut scene, 1.0);

        assert!(
            scene
                .ops
                .iter()
                .any(|op| matches!(op, SceneOp::Quad { .. }))
        );
        assert!(
            scene
                .ops
                .iter()
                .any(|op| matches!(op, SceneOp::Text { .. }))
        );
    }

    #[test]
    fn dock_space_clears_hover_on_drop_without_drag_session() {
        let window = AppWindowId::default();

        let mut ui = crate::UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(DockSpace::new(window));
        ui.set_root(root);

        let mut app = App::new();
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

        let mut text = FakeTextService::default();
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
}
