// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::hit_test::{hit_test_split_handle, hit_test_tab, tab_scroll_for_node};
use super::layout::{
    active_panel_content_bounds, compute_layout_map, dock_space_regions, float_zone, hidden_bounds,
    split_tab_bar,
};
use super::manager::DockManager;
use super::paint::{
    PaintDockParams, paint_dock, paint_drop_hints, paint_drop_overlay, paint_split_handles,
};
use super::prelude_core::*;
use super::prelude_runtime::*;
use super::prelude_ui::*;
use super::services::{
    DockFocusRequestService, DockPanelContentService, DockViewportOverlayHooksService,
};
use super::split_stabilize::{
    SplitSizeLock, apply_same_axis_locks, compute_same_axis_locks_for_split_drag,
};
use super::tab_bar_geometry::TabBarGeometry;
use super::tab_bar_geometry::dock_tab_width_for_title;
use super::tab_overflow::{
    TabOverflowMenuState, overflow_menu_max_scroll, overflow_menu_row_at_pos,
    overflow_menu_row_count, overflow_menu_row_height, tab_overflow_button_rect,
    tab_overflow_menu_rect, tab_strip_rect_with_overflow_button,
};
use super::viewport::{
    ViewportCaptureState, hit_test_active_viewport_panel, viewport_input_from_hit,
    viewport_input_from_hit_clamped,
};
use crate::invalidation::DockInvalidationService;
use fret_ui::retained_bridge::resizable_panel_group as resizable;

const DOCK_FLOATING_BORDER: Px = Px(1.0);
const DOCK_FLOATING_TITLE_H: Px = Px(22.0);
const DOCK_FLOATING_CLOSE_SIZE: Px = Px(14.0);

pub struct DockSpace {
    pub window: fret_core::AppWindowId,
    semantics_test_id: Option<&'static str>,
    last_bounds: Rect,
    prepaint_wants_animation_frames: bool,
    dock_drop_resolve_diagnostics: Option<(
        fret_runtime::FrameId,
        fret_runtime::DockDropResolveDiagnostics,
    )>,
    divider_drag: Option<DividerDragSession>,
    floating_drag: Option<FloatingDragState>,
    pending_dock_drags: HashMap<fret_core::PointerId, PendingDockDrag>,
    pending_dock_tabs_drags: HashMap<fret_core::PointerId, PendingDockTabsDrag>,
    hovered_floating_close: Option<DockNodeId>,
    pressed_floating_close: Option<DockNodeId>,
    hovered_floating_title_bar: Option<DockNodeId>,
    panel_content: HashMap<PanelKey, NodeId>,
    panel_last_sizes: HashMap<PanelKey, Size>,
    viewport_capture: HashMap<fret_core::PointerId, ViewportCaptureState>,
    tab_titles: HashMap<PanelKey, PreparedTabTitle>,
    empty_state: Option<PreparedTabTitle>,
    hovered_tab: Option<(DockNodeId, usize)>,
    hovered_tab_close: bool,
    hovered_tab_overflow_button: Option<DockNodeId>,
    pressed_tab_close: Option<(DockNodeId, usize, PanelKey)>,
    tab_scroll: HashMap<DockNodeId, Px>,
    tab_drag_auto_scroll_last_frame: HashMap<DockNodeId, fret_runtime::FrameId>,
    tab_overflow_menu: Option<TabOverflowMenuState>,
    tab_widths: HashMap<DockNodeId, Arc<[Px]>>,
    tab_close_glyph: Option<PreparedTabTitle>,
    tab_overflow_glyph: Option<PreparedTabTitle>,
    float_zone_glyph: Option<PreparedTabTitle>,
    float_zone_tooltip: Option<PreparedTabTitle>,
    tab_text_style: TextStyle,
    tab_close_style: TextStyle,
    empty_state_style: TextStyle,
    float_zone_style: TextStyle,
    float_zone_tooltip_style: TextStyle,
    last_empty_state_scale_factor: Option<f32>,
    last_empty_state_theme_revision: Option<u64>,
    last_float_zone_scale_factor: Option<f32>,
    last_float_zone_theme_revision: Option<u64>,
    last_float_zone_tooltip_scale_factor: Option<f32>,
    last_float_zone_tooltip_theme_revision: Option<u64>,
    last_tab_text_scale_factor: Option<f32>,
    last_theme_revision: Option<u64>,
    last_active_tabs: Option<DockNodeId>,
    hovered_float_zone: bool,
}

#[derive(Debug, Clone, Copy)]
struct FloatingDragState {
    pointer_id: fret_core::PointerId,
    floating: DockNodeId,
    grab_offset: Point,
    start_rect: Rect,
    start: Point,
    start_tick: fret_runtime::TickId,
    activated: bool,
    last_debug_frame: Option<fret_runtime::FrameId>,
}

#[derive(Debug, Clone)]
struct PendingDockDrag {
    start: Point,
    panel: PanelKey,
    grab_offset: Point,
    start_tick: fret_runtime::TickId,
}

#[derive(Debug, Clone)]
struct PendingDockTabsDrag {
    start: Point,
    tabs: DockNodeId,
    grab_offset: Point,
    start_tick: fret_runtime::TickId,
}

#[derive(Debug, Clone)]
struct DividerDragSession {
    handle: DividerDragState,
    layout_root: DockNodeId,
    layout_bounds: Rect,
    locks: Vec<SplitSizeLock>,
}

#[derive(Debug, Clone, Copy)]
struct FloatingChrome {
    outer: Rect,
    title_bar: Rect,
    close_button: Rect,
    inner: Rect,
}

impl DockSpace {
    pub fn new(window: fret_core::AppWindowId) -> Self {
        Self {
            window,
            semantics_test_id: None,
            last_bounds: Rect::default(),
            prepaint_wants_animation_frames: false,
            dock_drop_resolve_diagnostics: None,
            divider_drag: None,
            floating_drag: None,
            pending_dock_drags: HashMap::new(),
            pending_dock_tabs_drags: HashMap::new(),
            hovered_floating_close: None,
            pressed_floating_close: None,
            hovered_floating_title_bar: None,
            panel_content: HashMap::new(),
            panel_last_sizes: HashMap::new(),
            viewport_capture: HashMap::new(),
            tab_titles: HashMap::new(),
            empty_state: None,
            hovered_tab: None,
            hovered_tab_close: false,
            hovered_tab_overflow_button: None,
            pressed_tab_close: None,
            tab_scroll: HashMap::new(),
            tab_drag_auto_scroll_last_frame: HashMap::new(),
            tab_overflow_menu: None,
            tab_widths: HashMap::new(),
            tab_close_glyph: None,
            tab_overflow_glyph: None,
            float_zone_glyph: None,
            float_zone_tooltip: None,
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
            float_zone_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(11.0),
                ..Default::default()
            },
            float_zone_tooltip_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(12.0),
                ..Default::default()
            },
            last_empty_state_scale_factor: None,
            last_empty_state_theme_revision: None,
            last_float_zone_scale_factor: None,
            last_float_zone_theme_revision: None,
            last_float_zone_tooltip_scale_factor: None,
            last_float_zone_tooltip_theme_revision: None,
            last_tab_text_scale_factor: None,
            last_theme_revision: None,
            last_active_tabs: None,
            hovered_float_zone: false,
        }
    }

    pub fn with_semantics_test_id(mut self, id: &'static str) -> Self {
        self.semantics_test_id = Some(id);
        self
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

    fn floating_chrome(outer: Rect) -> FloatingChrome {
        let border = DOCK_FLOATING_BORDER.0.max(0.0);
        let title_h = DOCK_FLOATING_TITLE_H.0.max(0.0);

        let inner_w = (outer.size.width.0 - border * 2.0).max(0.0);
        let inner_h = (outer.size.height.0 - border * 2.0 - title_h).max(0.0);

        let title_bar = Rect::new(
            Point::new(Px(outer.origin.x.0 + border), Px(outer.origin.y.0 + border)),
            Size::new(Px(inner_w), Px(title_h)),
        );

        let inner = Rect::new(
            Point::new(
                Px(outer.origin.x.0 + border),
                Px(outer.origin.y.0 + border + title_h),
            ),
            Size::new(Px(inner_w), Px(inner_h)),
        );

        let close_size = DOCK_FLOATING_CLOSE_SIZE.0.max(0.0);
        let close_pad = border.max(4.0);
        let close_button = Rect::new(
            Point::new(
                Px(title_bar.origin.x.0 + (title_bar.size.width.0 - close_pad - close_size)),
                Px(title_bar.origin.y.0 + (title_bar.size.height.0 - close_size) * 0.5),
            ),
            Size::new(Px(close_size), Px(close_size)),
        );

        FloatingChrome {
            outer,
            title_bar,
            close_button,
            inner,
        }
    }

    fn clamp_rect_to_bounds(rect: Rect, bounds: Rect) -> Rect {
        let mut out = rect;
        if bounds.size.width.0 > 0.0 && bounds.size.height.0 > 0.0 {
            let min_x = bounds.origin.x.0;
            let min_y = bounds.origin.y.0;
            let max_x = bounds.origin.x.0 + (bounds.size.width.0 - out.size.width.0).max(0.0);
            let max_y = bounds.origin.y.0 + (bounds.size.height.0 - out.size.height.0).max(0.0);
            out.origin.x = Px(out.origin.x.0.clamp(min_x, max_x.max(min_x)));
            out.origin.y = Px(out.origin.y.0.clamp(min_y, max_y.max(min_y)));
        }
        out
    }

    fn default_floating_rect_for_panel(
        &self,
        panel: &PanelKey,
        cursor: Point,
        tab_grab_offset: Point,
        window_bounds: Rect,
    ) -> Rect {
        let content = self
            .panel_last_sizes
            .get(panel)
            .copied()
            .unwrap_or(Size::new(Px(360.0), Px(240.0)));

        let inner_w = content.width.0.max(160.0);
        let inner_h = (content.height.0 + DOCK_TAB_H.0).max(120.0);

        let border = DOCK_FLOATING_BORDER.0.max(0.0);
        let title_h = DOCK_FLOATING_TITLE_H.0.max(0.0);
        let outer_w = inner_w + border * 2.0;
        let outer_h = inner_h + border * 2.0 + title_h;

        let inner_origin = Point::new(
            Px(cursor.x.0 - tab_grab_offset.x.0),
            Px(cursor.y.0 - tab_grab_offset.y.0),
        );
        let outer_origin = Point::new(
            Px(inner_origin.x.0 - border),
            Px(inner_origin.y.0 - border - title_h),
        );

        Self::clamp_rect_to_bounds(
            Rect::new(outer_origin, Size::new(Px(outer_w), Px(outer_h))),
            window_bounds,
        )
    }

    fn rebuild_tab_titles(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        theme: fret_ui::ThemeSnapshot,
        scale_factor: f32,
        dock: &DockManager,
        layout: &std::collections::HashMap<DockNodeId, Rect>,
        drag_panel: Option<&PanelKey>,
    ) {
        self.tab_text_style.size = theme.metric_required("font.size");
        self.tab_close_style.size = theme.metric_required("font.size");
        self.empty_state_style.size = theme.metric_required("font.size");

        let mut visible_set: HashSet<PanelKey> = HashSet::new();
        for &node_id in layout.keys() {
            let Some(DockNode::Tabs { tabs, .. }) = dock.graph.node(node_id) else {
                continue;
            };
            for panel in tabs {
                visible_set.insert(panel.clone());
            }
        }
        if let Some(panel) = drag_panel {
            visible_set.insert(panel.clone());
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
        if let Some(glyph) = self.tab_overflow_glyph.take() {
            services.text().release(glyph.blob);
        }

        let pad_x = theme.metric_required("metric.padding.md");
        let reserve = Px(DOCK_TAB_CLOSE_SIZE.0 + DOCK_TAB_CLOSE_GAP.0);
        let inner_max_w = Px((DOCK_TAB_MAX_W.0 - pad_x.0 * 2.0 - reserve.0).max(0.0));
        let constraints = TextConstraints {
            max_width: Some(inner_max_w),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor,
        };

        let (close_blob, close_metrics) = services.text().prepare_str(
            "×",
            &self.tab_close_style,
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

        let (more_blob, more_metrics) = services.text().prepare_str(
            "⋯",
            &self.tab_close_style,
            TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor,
            },
        );
        self.tab_overflow_glyph = Some(PreparedTabTitle {
            blob: more_blob,
            metrics: more_metrics,
            title_hash: 0,
        });

        for panel in visible_set {
            let title = dock
                .panel(&panel)
                .map(|p| p.title.as_str())
                .unwrap_or(panel.kind.0.as_str());
            let title_hash = hash_title(title);
            let (blob, metrics) =
                services
                    .text()
                    .prepare_str(title, &self.tab_text_style, constraints);
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

    fn tab_bar_geometry_for_node(
        &self,
        theme: fret_ui::ThemeSnapshot,
        tabs: DockNodeId,
        tab_bar: Rect,
        tab_count: usize,
    ) -> (TabBarGeometry, bool) {
        let strip_candidate = tab_strip_rect_with_overflow_button(theme, tab_bar);
        let geom_candidate = self
            .tab_widths
            .get(&tabs)
            .filter(|w| w.len() == tab_count)
            .map(|w| TabBarGeometry::variable(strip_candidate, w.clone()))
            .unwrap_or_else(|| TabBarGeometry::fixed(strip_candidate, tab_count));
        let overflow = geom_candidate.max_scroll().0 > 0.0;
        if overflow {
            (geom_candidate, true)
        } else {
            let geom = self
                .tab_widths
                .get(&tabs)
                .filter(|w| w.len() == tab_count)
                .map(|w| TabBarGeometry::variable(tab_bar, w.clone()))
                .unwrap_or_else(|| TabBarGeometry::fixed(tab_bar, tab_count));
            (geom, false)
        }
    }

    fn max_tab_scroll(
        &self,
        theme: fret_ui::ThemeSnapshot,
        tabs: DockNodeId,
        tab_bar: Rect,
        tab_count: usize,
    ) -> Px {
        self.tab_bar_geometry_for_node(theme, tabs, tab_bar, tab_count)
            .0
            .max_scroll()
    }

    fn clamp_and_ensure_active_visible(
        &mut self,
        theme: fret_ui::ThemeSnapshot,
        tabs: DockNodeId,
        tab_bar: Rect,
        tab_count: usize,
        active: usize,
    ) {
        if tab_count == 0 {
            self.tab_scroll.remove(&tabs);
            return;
        }

        let (geom, _overflow) = self.tab_bar_geometry_for_node(theme, tabs, tab_bar, tab_count);
        let max_scroll = geom.max_scroll();
        let mut scroll = self.tab_scroll_for(tabs);

        if max_scroll.0 <= 0.0 {
            self.tab_scroll.remove(&tabs);
            return;
        }

        scroll = geom.ensure_tab_visible(scroll, active.min(tab_count.saturating_sub(1)));
        self.set_tab_scroll_for(tabs, scroll);
    }

    fn apply_tab_bar_drag_auto_scroll(
        &mut self,
        theme: fret_ui::ThemeSnapshot,
        graph: &DockGraph,
        hover: &mut HoverTarget,
        tab_bar: Rect,
        tab_count: usize,
        font_size: Px,
        position: Point,
        frame_id: fret_runtime::FrameId,
    ) -> bool {
        if tab_count == 0
            || hover.zone != DropZone::Center
            || hover.insert_index.is_none()
            || hover.outer
        {
            return false;
        }
        let tabs = hover.tabs;
        let (geom, _overflow) = self.tab_bar_geometry_for_node(theme, tabs, tab_bar, tab_count);
        if !tab_bar.contains(position) {
            return false;
        }

        let Some(DockNode::Tabs { .. }) = graph.node(tabs) else {
            return false;
        };

        let max_scroll = geom.max_scroll();
        if max_scroll.0 <= 0.0 {
            return false;
        }

        if self
            .tab_drag_auto_scroll_last_frame
            .get(&tabs)
            .is_some_and(|f| *f == frame_id)
        {
            return false;
        }
        self.tab_drag_auto_scroll_last_frame.insert(tabs, frame_id);

        let left_dist = position.x.0 - tab_bar.origin.x.0;
        let right_dist = tab_bar.origin.x.0 + tab_bar.size.width.0 - position.x.0;
        let edge = Px(((tab_bar.size.height.0 * 0.6).max(font_size.0 * 1.25)).clamp(12.0, 28.0));

        let (dir, t) = if left_dist >= 0.0 && left_dist < edge.0 {
            (-1.0, 1.0 - left_dist / edge.0)
        } else if right_dist >= 0.0 && right_dist < edge.0 {
            (1.0, 1.0 - right_dist / edge.0)
        } else {
            return false;
        };
        let t = t.clamp(0.0, 1.0);

        let base = (font_size.0 * 0.9).clamp(8.0, 22.0);
        let step = base * (0.20 + 0.80 * t);

        let prev_scroll = self.tab_scroll_for(tabs);
        let next_scroll = Px((prev_scroll.0 + dir * step).clamp(0.0, max_scroll.0));
        if (next_scroll.0 - prev_scroll.0).abs() < 0.01 {
            return false;
        }

        self.set_tab_scroll_for(tabs, next_scroll);

        hover.insert_index = Some(geom.compute_insert_index(position, next_scroll));
        true
    }

    fn rebuild_empty_state(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        theme: fret_ui::ThemeSnapshot,
        scale_factor: f32,
        max_width: Px,
    ) {
        self.empty_state_style.size = theme.metric_required("font.size");
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
        let (blob, metrics) = services.text().prepare_str(
            "No panels in this window.\nUse File → Layout → Reset Layout.",
            &self.empty_state_style,
            constraints,
        );
        self.empty_state = Some(PreparedTabTitle {
            blob,
            metrics,
            title_hash: 0,
        });
    }

    fn rebuild_float_zone_glyph(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        theme: fret_ui::ThemeSnapshot,
        scale_factor: f32,
    ) {
        self.float_zone_style.size = theme.metric_required("font.size");
        if self.last_float_zone_theme_revision == Some(theme.revision)
            && self.last_float_zone_scale_factor == Some(scale_factor)
        {
            return;
        }
        self.last_float_zone_theme_revision = Some(theme.revision);
        self.last_float_zone_scale_factor = Some(scale_factor);

        if let Some(prev) = self.float_zone_glyph.take() {
            services.text().release(prev.blob);
        }

        let constraints = TextConstraints {
            max_width: Some(Px(64.0)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor,
        };
        let (blob, metrics) = services
            .text()
            .prepare_str("F", &self.float_zone_style, constraints);
        self.float_zone_glyph = Some(PreparedTabTitle {
            blob,
            metrics,
            title_hash: 0,
        });
    }

    fn rebuild_float_zone_tooltip(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        theme: fret_ui::ThemeSnapshot,
        scale_factor: f32,
        msg: &str,
    ) {
        self.float_zone_tooltip_style.size = theme.metric_required("font.size");

        let hash_title = |s: &str| -> u64 {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            s.hash(&mut hasher);
            hasher.finish()
        };
        let msg_hash = hash_title(msg);

        if self.last_float_zone_tooltip_theme_revision == Some(theme.revision)
            && self.last_float_zone_tooltip_scale_factor == Some(scale_factor)
            && self
                .float_zone_tooltip
                .as_ref()
                .is_some_and(|t| t.title_hash == msg_hash)
        {
            return;
        }
        self.last_float_zone_tooltip_theme_revision = Some(theme.revision);
        self.last_float_zone_tooltip_scale_factor = Some(scale_factor);

        if let Some(prev) = self.float_zone_tooltip.take() {
            services.text().release(prev.blob);
        }

        let constraints = TextConstraints {
            max_width: Some(Px(280.0)),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor,
        };
        let (blob, metrics) =
            services
                .text()
                .prepare_str(msg, &self.float_zone_tooltip_style, constraints);

        self.float_zone_tooltip = Some(PreparedTabTitle {
            blob,
            metrics,
            title_hash: msg_hash,
        });
    }

    fn paint_float_zone_hint(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        theme: fret_ui::ThemeSnapshot,
        scale_factor: f32,
        dock_bounds: Rect,
        tear_off_possible: bool,
        scene: &mut fret_core::Scene,
    ) {
        let rect = float_zone(dock_bounds);
        self.rebuild_float_zone_glyph(services, theme, scale_factor);

        let border = theme.color_required("border");
        let card = theme.color_required("card");
        let fg = theme.color_required("muted-foreground");

        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(20),
            rect,
            background: card,
            border: Edges::all(Px(1.0)),
            border_color: border,
            corner_radii: fret_core::Corners::all(Px(6.0)),
        });

        let Some(glyph) = self.float_zone_glyph else {
            return;
        };

        let x = rect.origin.x.0 + (rect.size.width.0 - glyph.metrics.size.width.0) * 0.5;
        let y = rect.origin.y.0
            + (rect.size.height.0 - glyph.metrics.size.height.0) * 0.5
            + glyph.metrics.baseline.0;

        scene.push(SceneOp::Text {
            order: fret_core::DrawOrder(21),
            origin: Point::new(Px(x), Px(y)),
            text: glyph.blob,
            color: fg,
        });

        if !self.hovered_float_zone {
            return;
        }

        let msg = if tear_off_possible {
            "Click: float active tabs (in-window).\nDrag tab outside the window: tear-off (OS window)."
        } else {
            "Click: float active tabs (in-window).\nTear-off (OS window) is disabled on this platform/session."
        };

        self.rebuild_float_zone_tooltip(services, theme, scale_factor, msg);

        let Some(tooltip) = self.float_zone_tooltip else {
            return;
        };

        let pad = theme.metric_required("metric.padding.sm").0.max(4.0);
        let border_px = Px(1.0);
        let gap = Px(8.0);

        let size = Size::new(
            Px((tooltip.metrics.size.width.0 + pad * 2.0).max(0.0)),
            Px((tooltip.metrics.size.height.0 + pad * 2.0).max(0.0)),
        );

        let desired = Rect::new(
            Point::new(
                Px(rect.origin.x.0 + rect.size.width.0 + gap.0),
                rect.origin.y,
            ),
            size,
        );
        let tooltip_rect = Self::clamp_rect_to_bounds(desired, dock_bounds);

        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(22),
            rect: tooltip_rect,
            background: theme.color_required("popover"),
            border: Edges::all(border_px),
            border_color: border,
            corner_radii: fret_core::Corners::all(Px(8.0)),
        });

        let text_origin = Point::new(
            Px(tooltip_rect.origin.x.0 + pad),
            Px(tooltip_rect.origin.y.0 + pad + tooltip.metrics.baseline.0),
        );
        scene.push(SceneOp::Text {
            order: fret_core::DrawOrder(23),
            origin: text_origin,
            text: tooltip.blob,
            color: theme.color_required("popover-foreground"),
        });
    }

    fn find_first_tabs(graph: &DockGraph, node: DockNodeId) -> Option<DockNodeId> {
        let Some(n) = graph.node(node) else {
            return None;
        };
        match n {
            DockNode::Tabs { tabs, .. } => (!tabs.is_empty()).then_some(node),
            DockNode::Split { children, .. } => children
                .iter()
                .copied()
                .find_map(|child| Self::find_first_tabs(graph, child)),
            DockNode::Floating { child } => Self::find_first_tabs(graph, *child),
        }
    }

    fn graph_contains_node(graph: &DockGraph, root: DockNodeId, needle: DockNodeId) -> bool {
        if root == needle {
            return true;
        }
        let Some(n) = graph.node(root) else {
            return false;
        };
        match n {
            DockNode::Tabs { .. } => false,
            DockNode::Split { children, .. } => children
                .iter()
                .copied()
                .any(|child| Self::graph_contains_node(graph, child, needle)),
            DockNode::Floating { child } => Self::graph_contains_node(graph, *child, needle),
        }
    }

    fn find_floating_container_for_tabs(
        &self,
        graph: &DockGraph,
        tabs: DockNodeId,
    ) -> Option<DockNodeId> {
        graph
            .floating_windows(self.window)
            .iter()
            .find_map(|w| Self::graph_contains_node(graph, w.floating, tabs).then_some(w.floating))
    }

    fn float_zone_click_op(
        &mut self,
        graph: &DockGraph,
        root: DockNodeId,
        dock_bounds: Rect,
        window_bounds: Rect,
    ) -> Option<DockOp> {
        let tabs = self
            .last_active_tabs
            .filter(|tabs_node| {
                graph.node(*tabs_node).is_some_and(|n| match n {
                    DockNode::Tabs { tabs, .. } => !tabs.is_empty(),
                    _ => false,
                })
            })
            .or_else(|| Self::find_first_tabs(graph, root))?;

        if let Some(floating) = self.find_floating_container_for_tabs(graph, tabs) {
            return Some(DockOp::RaiseFloating {
                window: self.window,
                floating,
            });
        }

        let panel = graph.node(tabs).and_then(|n| match n {
            DockNode::Tabs { tabs, active } => tabs.get(*active).cloned(),
            _ => None,
        })?;

        let cursor = Point::new(
            Px(dock_bounds.origin.x.0 + dock_bounds.size.width.0 * 0.5),
            Px(dock_bounds.origin.y.0 + dock_bounds.size.height.0 * 0.25),
        );
        let rect = self.default_floating_rect_for_panel(
            &panel,
            cursor,
            Point::new(Px(0.0), Px(0.0)),
            window_bounds,
        );

        Some(DockOp::FloatTabsInWindow {
            source_window: self.window,
            source_tabs: tabs,
            target_window: self.window,
            rect,
        })
    }

    fn paint_empty_state<H: UiHost>(&mut self, cx: &mut PaintCx<'_, H>) {
        let theme = cx.theme().snapshot();
        cx.scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(0),
            rect: cx.bounds,
            background: theme.color_required("card"),
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });

        let pad = theme.metric_required("metric.padding.md").0.max(0.0);
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
            color: theme.color_required("muted-foreground"),
        });
    }
}

impl<H: UiHost> Widget<H> for DockSpace {
    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Panel);
        if let Some(id) = self.semantics_test_id {
            cx.set_test_id(id);
        }
    }

    fn prepaint(&mut self, cx: &mut PrepaintCx<'_, H>) {
        // Keep the dock host "alive" as a stable internal drag route target.
        //
        // This must be refreshed during paint/layout, not only during event handling, because
        // cross-window drag routing needs a reliable per-window anchor even when hit-testing is
        // over unrelated UI (ADR 0072) and even when no events have fired this frame.
        fret_ui::internal_drag::set_route(
            cx.app,
            self.window,
            fret_runtime::DRAG_KIND_DOCK_PANEL,
            cx.node,
        );
        fret_ui::internal_drag::set_route(
            cx.app,
            self.window,
            fret_runtime::DRAG_KIND_DOCK_TABS,
            cx.node,
        );
        fret_ui::internal_drag::set_route(
            cx.app,
            self.window,
            fret_runtime::DRAG_KIND_DOCK_TABS,
            cx.node,
        );

        let is_dock_dragging = cx.app.any_drag_session(|d| {
            (d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                || d.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
                && (d.source_window == self.window || d.current_window == self.window)
                && d.dragging
        });
        let wants_animation_frames = is_dock_dragging
            || self.divider_drag.is_some()
            || self.floating_drag.is_some()
            || !self.viewport_capture.is_empty();

        // If the node is a view-cache root and paint caching is active, a "no-op" frame can
        // replay the previous paint ops and skip `Widget::paint()`. This makes it easy to miss
        // frame-driven UI (dragging, capture, resize).
        //
        // Keep paint invalidated while the chrome is active so cached replay cannot suppress
        // per-frame indicator updates. On transitions, request a redraw to ensure we get at least
        // one paint pass to establish/tear down the frame loop.
        if wants_animation_frames {
            cx.invalidate_self(Invalidation::Paint);
            if !self.prepaint_wants_animation_frames {
                cx.request_redraw();
            }
        } else if self.prepaint_wants_animation_frames {
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
        }
        self.prepaint_wants_animation_frames = wants_animation_frames;
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &fret_core::Event) {
        let theme = cx.theme().snapshot();
        let font_size = theme.metric_required("font.size");

        let mut pending_effects: Vec<Effect> = Vec::new();
        let mut pending_redraws: Vec<fret_core::AppWindowId> = Vec::new();
        let mut invalidate_paint = false;
        let mut invalidate_layout = false;
        let mut request_focus: Option<NodeId> = None;
        let mut request_focus_panel: Option<PanelKey> = None;
        let mut request_pointer_capture: Option<Option<NodeId>> = None;
        let mut request_cursor: Option<fret_core::CursorIcon> = None;
        let mut stop_propagation = false;

        #[derive(Clone)]
        struct DockPanelDragSnapshot {
            source_window: fret_core::AppWindowId,
            start: Point,
            dragging: bool,
            panel: PanelKey,
            grab_offset: Point,
            start_tick: fret_runtime::TickId,
            tear_off_requested: bool,
            tear_off_oob_start_frame: Option<fret_runtime::FrameId>,
            dock_previews_enabled: bool,
        }

        #[derive(Clone)]
        struct DockTabsDragSnapshot {
            source_window: fret_core::AppWindowId,
            start: Point,
            dragging: bool,
            source_tabs: DockNodeId,
            tabs: Vec<PanelKey>,
            active: usize,
            grab_offset: Point,
            start_tick: fret_runtime::TickId,
            dock_previews_enabled: bool,
        }

        #[derive(Clone)]
        enum DockDragSnapshot {
            Panel(DockPanelDragSnapshot),
            Tabs(DockTabsDragSnapshot),
        }

        fn dock_drop_target(
            graph: &DockGraph,
            root: DockNodeId,
            layout: &HashMap<DockNodeId, Rect>,
            tab_scroll: &HashMap<DockNodeId, Px>,
            tab_widths: &HashMap<DockNodeId, Arc<[Px]>>,
            hint_font_size_inner: Px,
            hint_font_size_outer: Px,
            position: Point,
            mut candidates: Option<&mut Vec<fret_runtime::DockDropCandidateRectDiagnostics>>,
        ) -> Option<(HoverTarget, fret_runtime::DockDropResolveSource)> {
            fn leaf_tabs_node_at_pos(
                graph: &DockGraph,
                layout: &HashMap<DockNodeId, Rect>,
                position: Point,
            ) -> Option<(DockNodeId, Rect, usize)> {
                let mut best: Option<(DockNodeId, Rect, usize, f32)> = None;
                for (&node, &rect) in layout.iter() {
                    let Some(DockNode::Tabs { tabs, .. }) = graph.node(node) else {
                        continue;
                    };
                    if tabs.is_empty() || !rect.contains(position) {
                        continue;
                    }
                    let area = rect.size.width.0 * rect.size.height.0;
                    match best {
                        None => best = Some((node, rect, tabs.len(), area)),
                        Some((_best_node, _best_rect, _best_len, best_area)) => {
                            if area < best_area {
                                best = Some((node, rect, tabs.len(), area));
                            }
                        }
                    }
                }
                best.map(|(node, rect, len, _)| (node, rect, len))
            }

            let leaf = leaf_tabs_node_at_pos(graph, layout, position);
            if let Some((tabs_node, rect, tab_count)) = leaf {
                if let Some(candidates) = candidates.as_deref_mut() {
                    candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
                        kind: fret_runtime::DockDropCandidateRectKind::LeafTabsRect,
                        zone: None,
                        rect,
                    });
                }
                let (tab_bar, _content) = split_tab_bar(rect);
                if tab_bar.contains(position) {
                    if let Some(candidates) = candidates.as_deref_mut() {
                        candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
                            kind: fret_runtime::DockDropCandidateRectKind::TabBarRect,
                            zone: None,
                            rect: tab_bar,
                        });
                    }
                    let scroll = tab_scroll_for_node(tab_scroll, tabs_node);
                    return Some((
                        HoverTarget {
                            tabs: tabs_node,
                            root,
                            leaf_tabs: tabs_node,
                            zone: DropZone::Center,
                            insert_index: Some({
                                let geom = tab_widths
                                    .get(&tabs_node)
                                    .filter(|w| w.len() == tab_count)
                                    .map(|w| TabBarGeometry::variable(tab_bar, w.clone()))
                                    .unwrap_or_else(|| TabBarGeometry::fixed(tab_bar, tab_count));
                                geom.compute_insert_index(position, scroll)
                            }),
                            outer: false,
                            explicit: false,
                        },
                        fret_runtime::DockDropResolveSource::TabBar,
                    ));
                }
            }

            // Reduce preview flicker: only show/allow docking previews when hovering over a
            // tab bar or one of the hint-pad rectangles (inner/outer).
            if let Some(&root_rect) = layout.get(&root)
                && let Some((leaf_tabs, _leaf_rect, _leaf_tab_count)) = leaf
                && root != leaf_tabs
                && let Some(zone) = super::layout::dock_hint_pick_zone(
                    root_rect,
                    hint_font_size_outer,
                    true,
                    position,
                )
                && zone != DropZone::Center
            {
                if let Some(candidates) = candidates.as_deref_mut() {
                    candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
                        kind: fret_runtime::DockDropCandidateRectKind::RootRect,
                        zone: None,
                        rect: root_rect,
                    });
                    for (z, r) in super::layout::dock_hint_rects_with_font(
                        root_rect,
                        hint_font_size_outer,
                        true,
                    ) {
                        candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
                            kind: fret_runtime::DockDropCandidateRectKind::OuterHintRect,
                            zone: Some(z),
                            rect: r,
                        });
                    }
                }
                return Some((
                    HoverTarget {
                        tabs: root,
                        root,
                        leaf_tabs,
                        zone,
                        insert_index: None,
                        outer: true,
                        explicit: true,
                    },
                    fret_runtime::DockDropResolveSource::OuterHintRect,
                ));
            }

            if let Some((tabs_node, rect, _tab_count)) = leaf
                && let Some(zone) =
                    super::layout::dock_hint_pick_zone(rect, hint_font_size_inner, false, position)
            {
                if let Some(candidates) = candidates.as_deref_mut() {
                    for (z, r) in
                        super::layout::dock_hint_rects_with_font(rect, hint_font_size_inner, false)
                    {
                        candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
                            kind: fret_runtime::DockDropCandidateRectKind::InnerHintRect,
                            zone: Some(z),
                            rect: r,
                        });
                    }
                }
                return Some((
                    HoverTarget {
                        tabs: tabs_node,
                        root,
                        leaf_tabs: tabs_node,
                        zone,
                        insert_index: None,
                        outer: false,
                        explicit: true,
                    },
                    fret_runtime::DockDropResolveSource::InnerHintRect,
                ));
            }

            None
        }

        fn is_outside_bounds_with_margin(bounds: Rect, position: Point, margin: Px) -> bool {
            position.x.0 < bounds.origin.x.0 - margin.0
                || position.y.0 < bounds.origin.y.0 - margin.0
                || position.x.0 > bounds.origin.x.0 + bounds.size.width.0 + margin.0
                || position.y.0 > bounds.origin.y.0 + bounds.size.height.0 + margin.0
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum FloatingHitKind {
            Close,
            TitleBar,
            Body,
        }

        fn hit_test_floating(
            graph: &DockGraph,
            window: fret_core::AppWindowId,
            position: Point,
        ) -> Option<(DockNodeId, FloatingChrome, FloatingHitKind)> {
            for floating in graph.floating_windows(window).iter().rev() {
                let chrome = DockSpace::floating_chrome(floating.rect);
                if !chrome.outer.contains(position) {
                    continue;
                }
                if chrome.close_button.contains(position) {
                    return Some((floating.floating, chrome, FloatingHitKind::Close));
                }
                if chrome.title_bar.contains(position) {
                    return Some((floating.floating, chrome, FloatingHitKind::TitleBar));
                }
                return Some((floating.floating, chrome, FloatingHitKind::Body));
            }
            None
        }

        fn layout_context_for_position(
            graph: &DockGraph,
            window: fret_core::AppWindowId,
            root: DockNodeId,
            dock_bounds: Rect,
            position: Point,
        ) -> (DockNodeId, Rect) {
            if let Some((floating, chrome, _)) = hit_test_floating(graph, window, position)
                && chrome.inner.contains(position)
            {
                return (floating, chrome.inner);
            }
            (root, dock_bounds)
        }

        fn compute_dock_drop_target(
            graph: &DockGraph,
            window: fret_core::AppWindowId,
            root: DockNodeId,
            dock_bounds: Rect,
            window_bounds: Rect,
            tab_scroll: &HashMap<DockNodeId, Px>,
            tab_widths: &HashMap<DockNodeId, Arc<[Px]>>,
            hint_font_size_inner: Px,
            hint_font_size_outer: Px,
            split_handle_gap: Px,
            split_handle_hit_thickness: Px,
            position: Point,
            mut candidates: Option<&mut Vec<fret_runtime::DockDropCandidateRectDiagnostics>>,
        ) -> (Option<DockDropTarget>, fret_runtime::DockDropResolveSource) {
            fn clamp_point_inside_rect(rect: Rect, point: Point) -> Point {
                const EPS: f32 = 0.001;
                let x0 = rect.origin.x.0;
                let y0 = rect.origin.y.0;
                let x1 = x0 + rect.size.width.0;
                let y1 = y0 + rect.size.height.0;

                let max_x = if x1 > x0 { (x1 - EPS).max(x0) } else { x0 };
                let max_y = if y1 > y0 { (y1 - EPS).max(y0) } else { y0 };

                Point::new(
                    Px(point.x.0.clamp(x0, max_x)),
                    Px(point.y.0.clamp(y0, max_y)),
                )
            }

            if let Some(candidates) = candidates.as_deref_mut() {
                candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
                    kind: fret_runtime::DockDropCandidateRectKind::WindowBounds,
                    zone: None,
                    rect: window_bounds,
                });
                candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
                    kind: fret_runtime::DockDropCandidateRectKind::DockBounds,
                    zone: None,
                    rect: dock_bounds,
                });
                candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
                    kind: fret_runtime::DockDropCandidateRectKind::FloatZone,
                    zone: None,
                    rect: float_zone(dock_bounds),
                });
            }
            if !window_bounds.contains(position) {
                return (
                    Some(DockDropTarget::Float { window }),
                    fret_runtime::DockDropResolveSource::OutsideWindow,
                );
            }
            if float_zone(dock_bounds).contains(position) {
                return (
                    Some(DockDropTarget::Float { window }),
                    fret_runtime::DockDropResolveSource::FloatZone,
                );
            }

            if let Some((floating, chrome, FloatingHitKind::TitleBar)) =
                hit_test_floating(graph, window, position)
            {
                let layout_bounds = chrome.inner;
                let layout = compute_layout_map(
                    graph,
                    floating,
                    layout_bounds,
                    split_handle_gap,
                    split_handle_hit_thickness,
                );
                let center = Point::new(
                    Px(layout_bounds.origin.x.0 + layout_bounds.size.width.0 * 0.5),
                    Px(layout_bounds.origin.y.0 + layout_bounds.size.height.0 * 0.5),
                );
                let mut best: Option<(DockNodeId, f32)> = None;
                for (&node_id, &rect) in layout.iter() {
                    if !rect.contains(center) {
                        continue;
                    }
                    let Some(DockNode::Tabs { tabs, .. }) = graph.node(node_id) else {
                        continue;
                    };
                    if tabs.is_empty() {
                        continue;
                    }
                    let area = rect.size.width.0 * rect.size.height.0;
                    match best {
                        None => best = Some((node_id, area)),
                        Some((_best_node, best_area)) => {
                            if area < best_area {
                                best = Some((node_id, area));
                            }
                        }
                    }
                }

                if let Some((leaf_tabs, _area)) = best {
                    return (
                        Some(DockDropTarget::Dock(HoverTarget {
                            tabs: leaf_tabs,
                            root: floating,
                            leaf_tabs,
                            zone: DropZone::Center,
                            insert_index: None,
                            outer: false,
                            explicit: false,
                        })),
                        fret_runtime::DockDropResolveSource::FloatingTitleBar,
                    );
                }
                return (None, fret_runtime::DockDropResolveSource::None);
            }

            let (layout_root, layout_bounds, effective_position) =
                match hit_test_floating(graph, window, position) {
                    None | Some((_, _, FloatingHitKind::Close)) => {
                        let (layout_root, layout_bounds) =
                            layout_context_for_position(graph, window, root, dock_bounds, position);
                        (layout_root, layout_bounds, position)
                    }
                    Some((floating, chrome, FloatingHitKind::TitleBar)) => {
                        let projected = Point::new(
                            Px(chrome.inner.origin.x.0 + chrome.inner.size.width.0 * 0.5),
                            Px(chrome.inner.origin.y.0 + chrome.inner.size.height.0 * 0.5),
                        );
                        (
                            floating,
                            chrome.inner,
                            clamp_point_inside_rect(chrome.inner, projected),
                        )
                    }
                    Some((floating, chrome, FloatingHitKind::Body)) => (
                        floating,
                        chrome.inner,
                        clamp_point_inside_rect(chrome.inner, position),
                    ),
                };

            if !layout_bounds.contains(effective_position) {
                if let Some(candidates) = candidates.as_deref_mut() {
                    candidates.push(fret_runtime::DockDropCandidateRectDiagnostics {
                        kind: fret_runtime::DockDropCandidateRectKind::LayoutBounds,
                        zone: None,
                        rect: layout_bounds,
                    });
                }
                return (None, fret_runtime::DockDropResolveSource::LayoutBoundsMiss);
            }

            let layout = compute_layout_map(
                graph,
                layout_root,
                layout_bounds,
                split_handle_gap,
                split_handle_hit_thickness,
            );
            dock_drop_target(
                graph,
                layout_root,
                &layout,
                tab_scroll,
                tab_widths,
                hint_font_size_inner,
                hint_font_size_outer,
                effective_position,
                candidates,
            )
            .map(|(target, source)| (Some(DockDropTarget::Dock(target)), source))
            .unwrap_or((None, fret_runtime::DockDropResolveSource::None))
        }

        fn resolve_dock_drop_target(
            prev_hover: Option<DockDropTarget>,
            invert_docking: bool,
            window: fret_core::AppWindowId,
            graph: &DockGraph,
            root: DockNodeId,
            dock_bounds: Rect,
            window_bounds: Rect,
            tab_scroll: &HashMap<DockNodeId, Px>,
            tab_widths: &HashMap<DockNodeId, Arc<[Px]>>,
            hint_font_size_inner: Px,
            hint_font_size_outer: Px,
            split_handle_gap: Px,
            split_handle_hit_thickness: Px,
            position: Point,
            candidates: Option<&mut Vec<fret_runtime::DockDropCandidateRectDiagnostics>>,
        ) -> (Option<DockDropTarget>, fret_runtime::DockDropResolveSource) {
            if invert_docking {
                return (
                    Some(DockDropTarget::Float { window }),
                    fret_runtime::DockDropResolveSource::InvertDocking,
                );
            }
            if let Some(prev_hover) = prev_hover {
                return (
                    Some(prev_hover),
                    fret_runtime::DockDropResolveSource::LatchedPreviousHover,
                );
            }

            compute_dock_drop_target(
                graph,
                window,
                root,
                dock_bounds,
                window_bounds,
                tab_scroll,
                tab_widths,
                hint_font_size_inner,
                hint_font_size_outer,
                split_handle_gap,
                split_handle_hit_thickness,
                position,
                candidates,
            )
        }

        fn dock_drop_target_diagnostics(
            target: Option<&DockDropTarget>,
        ) -> Option<fret_runtime::DockDropTargetDiagnostics> {
            match target {
                Some(DockDropTarget::Dock(t)) => Some(fret_runtime::DockDropTargetDiagnostics {
                    layout_root: t.root,
                    tabs: t.tabs,
                    zone: t.zone,
                    insert_index: t.insert_index,
                    outer: t.outer,
                }),
                _ => None,
            }
        }

        fn compute_dock_drop_resolve_diagnostics(
            pointer_id: fret_core::PointerId,
            position: Point,
            window_bounds: Rect,
            dock_bounds: Rect,
            source: fret_runtime::DockDropResolveSource,
            target: Option<&DockDropTarget>,
            candidates: Vec<fret_runtime::DockDropCandidateRectDiagnostics>,
        ) -> fret_runtime::DockDropResolveDiagnostics {
            fret_runtime::DockDropResolveDiagnostics {
                pointer_id,
                position,
                window_bounds,
                dock_bounds,
                source,
                resolved: dock_drop_target_diagnostics(target),
                candidates,
            }
        }

        fn resolve_dock_drop_intent_panel<F>(
            target: Option<DockDropTarget>,
            drag: &DockPanelDragSnapshot,
            target_window: fret_core::AppWindowId,
            window_bounds: Rect,
            position: Point,
            allow_tear_off: bool,
            mark_drag_tear_off_requested: bool,
            default_floating_rect_for_panel: F,
        ) -> DockDropIntent
        where
            F: FnOnce(&PanelKey, Point, Point, Rect) -> Rect,
        {
            match target {
                Some(DockDropTarget::Dock(target)) => DockDropIntent::MovePanel {
                    source_window: drag.source_window,
                    panel: drag.panel.clone(),
                    target_window,
                    target_tabs: target.tabs,
                    zone: target.zone,
                    insert_index: target.insert_index,
                },
                Some(DockDropTarget::Float { .. }) => {
                    // Tear-off (new OS window) is only triggered by leaving the window bounds.
                    //
                    // `float_zone(...)` is a Fret-specific affordance to force in-window floating;
                    // it must never request a new OS window.
                    let wants_tear_off = allow_tear_off && !window_bounds.contains(position);
                    if wants_tear_off {
                        if drag.tear_off_requested || mark_drag_tear_off_requested {
                            DockDropIntent::None
                        } else {
                            DockDropIntent::RequestFloatPanelToNewWindow {
                                source_window: drag.source_window,
                                panel: drag.panel.clone(),
                                anchor: Some(fret_core::WindowAnchor {
                                    window: target_window,
                                    position: drag.grab_offset,
                                }),
                            }
                        }
                    } else {
                        let rect = default_floating_rect_for_panel(
                            &drag.panel,
                            position,
                            drag.grab_offset,
                            window_bounds,
                        );
                        DockDropIntent::FloatPanelInWindow {
                            source_window: drag.source_window,
                            panel: drag.panel.clone(),
                            target_window,
                            rect,
                        }
                    }
                }
                None => DockDropIntent::None,
            }
        }

        fn resolve_dock_drop_intent_tabs<F>(
            target: Option<DockDropTarget>,
            drag: &DockTabsDragSnapshot,
            target_window: fret_core::AppWindowId,
            window_bounds: Rect,
            position: Point,
            default_floating_rect_for_panel: F,
        ) -> DockDropIntent
        where
            F: FnOnce(&PanelKey, Point, Point, Rect) -> Rect,
        {
            match target {
                Some(DockDropTarget::Dock(target)) => DockDropIntent::MoveTabs {
                    source_window: drag.source_window,
                    source_tabs: drag.source_tabs,
                    target_window,
                    target_tabs: target.tabs,
                    zone: target.zone,
                    insert_index: target.insert_index,
                },
                Some(DockDropTarget::Float { .. }) => {
                    let panel = drag
                        .tabs
                        .get(drag.active)
                        .or_else(|| drag.tabs.first())
                        .cloned();
                    let Some(panel) = panel else {
                        return DockDropIntent::None;
                    };
                    let rect = default_floating_rect_for_panel(
                        &panel,
                        position,
                        drag.grab_offset,
                        window_bounds,
                    );
                    DockDropIntent::FloatTabsInWindow {
                        source_window: drag.source_window,
                        source_tabs: drag.source_tabs,
                        target_window,
                        rect,
                    }
                }
                None => DockDropIntent::None,
            }
        }

        fn apply_dock_drop_intent(
            intent: DockDropIntent,
            pending_effects: &mut Vec<Effect>,
            invalidate_layout: &mut bool,
        ) {
            match intent {
                DockDropIntent::None => {}
                DockDropIntent::MovePanel {
                    source_window,
                    panel,
                    target_window,
                    target_tabs,
                    zone,
                    insert_index,
                } => {
                    pending_effects.push(Effect::Dock(DockOp::MovePanel {
                        source_window,
                        panel,
                        target_window,
                        target_tabs,
                        zone,
                        insert_index,
                    }));
                    *invalidate_layout = true;
                }
                DockDropIntent::MoveTabs {
                    source_window,
                    source_tabs,
                    target_window,
                    target_tabs,
                    zone,
                    insert_index,
                } => {
                    pending_effects.push(Effect::Dock(DockOp::MoveTabs {
                        source_window,
                        source_tabs,
                        target_window,
                        target_tabs,
                        zone,
                        insert_index,
                    }));
                    *invalidate_layout = true;
                }
                DockDropIntent::FloatPanelInWindow {
                    source_window,
                    panel,
                    target_window,
                    rect,
                } => {
                    pending_effects.push(Effect::Dock(DockOp::FloatPanelInWindow {
                        source_window,
                        panel,
                        target_window,
                        rect,
                    }));
                    *invalidate_layout = true;
                }
                DockDropIntent::FloatTabsInWindow {
                    source_window,
                    source_tabs,
                    target_window,
                    rect,
                } => {
                    pending_effects.push(Effect::Dock(DockOp::FloatTabsInWindow {
                        source_window,
                        source_tabs,
                        target_window,
                        rect,
                    }));
                    *invalidate_layout = true;
                }
                DockDropIntent::RequestFloatPanelToNewWindow {
                    source_window,
                    panel,
                    anchor,
                } => {
                    pending_effects.push(Effect::Dock(DockOp::RequestFloatPanelToNewWindow {
                        source_window,
                        panel,
                        anchor,
                    }));
                    *invalidate_layout = true;
                }
            }
        }

        let pointer_id: fret_core::PointerId = match event {
            fret_core::Event::Pointer(fret_core::PointerEvent::Move { pointer_id, .. })
            | fret_core::Event::Pointer(fret_core::PointerEvent::Down { pointer_id, .. })
            | fret_core::Event::Pointer(fret_core::PointerEvent::Up { pointer_id, .. })
            | fret_core::Event::Pointer(fret_core::PointerEvent::Wheel { pointer_id, .. })
            | fret_core::Event::Pointer(fret_core::PointerEvent::PinchGesture {
                pointer_id, ..
            }) => *pointer_id,
            fret_core::Event::PointerCancel(e) => e.pointer_id,
            fret_core::Event::InternalDrag(e) => e.pointer_id,
            _ => fret_core::PointerId(0),
        };

        let dock_drag_affects_window = cx.app.any_drag_session(|d| {
            (d.kind == DRAG_KIND_DOCK_PANEL || d.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
                && (d.source_window == self.window || d.current_window == self.window)
        });
        let dock_drag = cx.app.drag(pointer_id).and_then(|d| {
            if let Some(p) = d.payload::<DockPanelDragPayload>() {
                return Some(DockDragSnapshot::Panel(DockPanelDragSnapshot {
                    source_window: d.source_window,
                    start: d.start_position,
                    dragging: d.dragging,
                    panel: p.panel.clone(),
                    grab_offset: p.grab_offset,
                    start_tick: p.start_tick,
                    tear_off_requested: p.tear_off_requested,
                    tear_off_oob_start_frame: p.tear_off_oob_start_frame,
                    dock_previews_enabled: p.dock_previews_enabled,
                }));
            }
            d.payload::<DockTabsDragPayload>().map(|p| {
                DockDragSnapshot::Tabs(DockTabsDragSnapshot {
                    source_window: d.source_window,
                    start: d.start_position,
                    dragging: d.dragging,
                    source_tabs: p.source_tabs,
                    tabs: p.tabs.clone(),
                    active: p.active,
                    grab_offset: p.grab_offset,
                    start_tick: p.start_tick,
                    dock_previews_enabled: p.dock_previews_enabled,
                })
            })
        });
        let has_pending_dock_drag = self.pending_dock_drags.contains_key(&pointer_id);
        let has_pending_dock_tabs_drag = self.pending_dock_tabs_drags.contains_key(&pointer_id);
        // While a dock drag session exists (even before it crosses the drag threshold), we must
        // not forward pointer moves/wheel to embedded viewports in this window. Docking owns the
        // interaction until the session ends (ADR 0072).
        let allow_viewport_hover = !dock_drag_affects_window
            && dock_drag.is_none()
            && !has_pending_dock_drag
            && !has_pending_dock_tabs_drag
            && cx.app.drag(pointer_id).is_none_or(|d| !d.dragging);
        let docking_interaction_settings = cx
            .app
            .global::<fret_runtime::DockingInteractionSettings>()
            .copied()
            .unwrap_or_default();
        let hint_font_size_inner = Px((font_size.0
            * docking_interaction_settings.dock_hint_scale_inner.max(0.0))
        .max(0.0));
        let hint_font_size_outer = Px((font_size.0
            * docking_interaction_settings.dock_hint_scale_outer.max(0.0))
        .max(0.0));
        let split_handle_gap = docking_interaction_settings.split_handle_gap;
        let split_handle_hit_thickness = docking_interaction_settings.split_handle_hit_thickness;
        let window_bounds = cx
            .app
            .global::<WindowMetricsService>()
            .and_then(|svc| svc.inner_bounds(self.window))
            .unwrap_or(self.last_bounds);
        let pixels_per_point = cx
            .app
            .global::<WindowMetricsService>()
            .and_then(|svc| svc.scale_factor(self.window))
            .unwrap_or(1.0);
        let (_chrome, dock_bounds) = dock_space_regions(self.last_bounds);
        let now_tick = cx.app.tick_id();
        let now_frame = cx.app.frame_id();
        let diagnostics_enabled = cx
            .app
            .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
            .is_some();

        let mut start_dock_drag: Option<(Point, DockPanelDragPayload, Point)> = None;
        let mut start_dock_tabs_drag: Option<(Point, DockTabsDragPayload, Point)> = None;
        let mut update_drag: Option<(Point, bool)> = None;
        let mut end_dock_drag = false;
        let mut mark_drag_tear_off_requested = false;
        let mut set_drag_tear_off_oob_start_frame: Option<Option<fret_runtime::FrameId>> = None;

        fret_ui::internal_drag::set_route(
            cx.app,
            self.window,
            fret_runtime::DRAG_KIND_DOCK_PANEL,
            cx.node,
        );
        let dock_space_node = cx.node;
        let allow_tear_off = cx.input_ctx.caps.ui.window_tear_off
            && cx.input_ctx.caps.ui.multi_window
            && cx.input_ctx.caps.ui.window_hover_detection
                != fret_runtime::WindowHoverDetectionQuality::None;
        let window_input_arbitration = cx.input_ctx.window_arbitration();
        let pointer_occlusion = cx.input_ctx.window_pointer_occlusion();
        let foreign_capture_active = window_input_arbitration.is_some_and(|snapshot| {
            if !snapshot.pointer_capture_active {
                return false;
            }

            let Some(dock_root) = cx.layer_root else {
                // If the dock space's layer cannot be determined, be conservative: treat
                // captures as foreign so we don't start competing interactions.
                return true;
            };

            if snapshot.pointer_capture_multiple_roots {
                return true;
            }

            snapshot
                .pointer_capture_root
                .is_none_or(|root| root != dock_root)
        });

        if cx.app.global::<DockManager>().is_none() {
            return;
        }

        cx.app
            .with_global_mut_untracked(DockManager::default, |dock, _app| {
                dock.register_dock_space_node(self.window, dock_space_node);
                let Some(root) = dock.graph.window_root(self.window) else {
                    return;
                };

                match event {
                    fret_core::Event::Pointer(p) => match p {
                        fret_core::PointerEvent::Down {
                            position,
                            button,
                            modifiers,
                            click_count,
                            pointer_type,
                            ..
                        } => {
                            let pointer_occlusion_blocks_mouse = matches!(
                                pointer_occlusion,
                                fret_runtime::WindowPointerOcclusion::BlockMouse
                                    | fret_runtime::WindowPointerOcclusion::BlockMouseExceptScroll
                            );
                            if pointer_occlusion_blocks_mouse
                                && *pointer_type == fret_core::PointerType::Mouse
                            {
                                return;
                            }

                            // When a different UI layer owns a pointer capture (typically an
                            // editor interaction), avoid starting competing dock/viewport capture
                            // sessions from other pointers.
                            if foreign_capture_active && cx.captured != Some(dock_space_node) {
                                return;
                            }
                            // Arbitration: while a dock drag session is active (or viewport capture is
                            // active), we do not allow starting competing capture sessions from a
                            // secondary button press. The active session owns the interaction.
                            if dock_drag_affects_window
                                || has_pending_dock_drag
                                || has_pending_dock_tabs_drag
                                || self.divider_drag.is_some()
                                || self.floating_drag.is_some()
                            {
                                return;
                            }
                            if !self.viewport_capture.is_empty() {
                                if let Some(capture) = self.viewport_capture.values().next() {
                                    if docking_interaction_settings
                                        .suppress_context_menu_during_viewport_capture
                                        && *button == fret_core::MouseButton::Right
                                        && capture.button != fret_core::MouseButton::Right
                                    {
                                        stop_propagation = true;
                                    }
                                }
                                return;
                            }

                            let (_chrome, dock_bounds) = dock_space_regions(self.last_bounds);
                            let mut handled = false;
                            let float_zone_rect = float_zone(dock_bounds);

                            if *button == fret_core::MouseButton::Left && dock_bounds.contains(*position)
                            {
                                if float_zone_rect.contains(*position) {
                                    if let Some(op) = self.float_zone_click_op(
                                        &dock.graph,
                                        root,
                                        dock_bounds,
                                        self.last_bounds,
                                    ) {
                                        pending_effects.push(Effect::Dock(op));
                                        invalidate_layout = true;
                                        invalidate_paint = true;
                                        pending_redraws.push(self.window);
                                        dock.hover = None;
                                        stop_propagation = true;
                                        handled = true;
                                    }
                                }

                                let mut layout_all = compute_layout_map(
                                    &dock.graph,
                                    root,
                                    dock_bounds,
                                    split_handle_gap,
                                    split_handle_hit_thickness,
                                );
                                for floating in dock.graph.floating_windows(self.window) {
                                    let chrome = Self::floating_chrome(floating.rect);
                                    let floating_layout = compute_layout_map(
                                        &dock.graph,
                                        floating.floating,
                                        chrome.inner,
                                        split_handle_gap,
                                        split_handle_hit_thickness,
                                    );
                                    for (k, v) in floating_layout {
                                        layout_all.insert(k, v);
                                    }
                                }

                                if let Some(menu) = self.tab_overflow_menu {
                                    let mut keep_open = true;

                                    let tabs_rect = layout_all.get(&menu.tabs).copied();
                                    let node = dock.graph.node(menu.tabs);
                                    if let (Some(tabs_rect), Some(DockNode::Tabs { tabs, .. })) =
                                        (tabs_rect, node)
                                    {
                                        let (tab_bar, _content) = split_tab_bar(tabs_rect);
                                        let button_rect = tab_overflow_button_rect(theme, tab_bar);
                                        let menu_rect =
                                            tab_overflow_menu_rect(theme, tab_bar, tabs.len());

                                        if menu_rect.contains(*position) {
                                            let row = overflow_menu_row_at_pos(
                                                menu_rect,
                                                tab_bar,
                                                tabs.len(),
                                                menu.scroll,
                                                *position,
                                            );
                                            if let Some(ix) = row {
                                                self.last_active_tabs = Some(menu.tabs);
                                                pending_effects.push(Effect::Dock(
                                                    DockOp::SetActiveTab {
                                                        tabs: menu.tabs,
                                                        active: ix,
                                                    },
                                                ));
                                                if let Some(panel) = tabs.get(ix) {
                                                    request_focus_panel = Some(panel.clone());
                                                }
                                                invalidate_layout = true;
                                                invalidate_paint = true;
                                                pending_redraws.push(self.window);
                                                keep_open = false;
                                            }
                                            handled = true;
                                        } else if button_rect.contains(*position) {
                                            // Toggle: clicking the overflow button closes the menu.
                                            keep_open = false;
                                            invalidate_paint = true;
                                            pending_redraws.push(self.window);
                                            handled = true;
                                        } else {
                                            // Click outside closes the menu, but does not swallow the click.
                                            if keep_open {
                                                keep_open = false;
                                                invalidate_paint = true;
                                                pending_redraws.push(self.window);
                                            }
                                        }
                                    } else {
                                        keep_open = false;
                                    }

                                    if keep_open {
                                        // Keep the menu state stable on unrelated clicks.
                                        self.tab_overflow_menu = Some(menu);
                                    } else {
                                        self.tab_overflow_menu = None;
                                    }
                                }

                                if !handled {
                                    // Open the overflow menu by clicking the overflow button on any overflowing tab bar.
                                    for (&node_id, &rect) in layout_all.iter() {
                                        let Some(DockNode::Tabs { tabs, active }) =
                                            dock.graph.node(node_id)
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
                                        let max_scroll =
                                            self.max_tab_scroll(theme, node_id, tab_bar, tabs.len());
                                        if max_scroll.0 <= 0.0 {
                                            continue;
                                        }
                                        let button_rect = tab_overflow_button_rect(theme, tab_bar);
                                        if !button_rect.contains(*position) {
                                            continue;
                                        }

                                        let row_h = overflow_menu_row_height(tab_bar).0;
                                        let visible = overflow_menu_row_count(tabs.len()) as f32;
                                        let active_y = *active as f32 * row_h;
                                        let min_scroll = active_y - (visible - 1.0) * row_h;
                                        let max_scroll_menu =
                                            overflow_menu_max_scroll(tab_bar, tabs.len());
                                        let scroll =
                                            Px(min_scroll.clamp(0.0, max_scroll_menu.0.max(0.0)));

                                        self.tab_overflow_menu = Some(TabOverflowMenuState {
                                            tabs: node_id,
                                            scroll,
                                            hovered: None,
                                        });
                                        invalidate_paint = true;
                                        pending_redraws.push(self.window);
                                        handled = true;
                                        break;
                                    }
                                }
                            }

                            if *button == fret_core::MouseButton::Left
                                && let Some((floating, _chrome, kind)) =
                                    hit_test_floating(&dock.graph, self.window, *position)
                            {
                                pending_effects.push(Effect::Dock(DockOp::RaiseFloating {
                                    window: self.window,
                                    floating,
                                }));
                                invalidate_paint = true;
                                pending_redraws.push(self.window);

                                match kind {
                                    FloatingHitKind::Close => {
                                        self.hovered_floating_close = Some(floating);
                                        self.pressed_floating_close = Some(floating);
                                        request_pointer_capture = Some(Some(dock_space_node));
                                        handled = true;
                                    }
                                    FloatingHitKind::TitleBar => {
                                        if let Some(entry) = dock
                                            .graph
                                            .floating_windows(self.window)
                                            .iter()
                                            .find(|w| w.floating == floating)
                                        {
                                            if std::env::var_os("FRET_DOCK_FLOAT_MOVE_DEBUG")
                                                .is_some_and(|v| !v.is_empty())
                                            {
                                                tracing::info!(
                                                    window = ?self.window,
                                                    pointer_id = ?pointer_id,
                                                    floating = ?floating,
                                                    start_rect = ?entry.rect,
                                                    start = ?position,
                                                    "floating drag start (title bar)"
                                                );
                                            }
                                            self.floating_drag = Some(FloatingDragState {
                                                pointer_id,
                                                floating,
                                                grab_offset: Point::new(
                                                    Px(position.x.0 - entry.rect.origin.x.0),
                                                    Px(position.y.0 - entry.rect.origin.y.0),
                                                ),
                                                start_rect: entry.rect,
                                                start: *position,
                                                start_tick: now_tick,
                                                activated: true,
                                                last_debug_frame: None,
                                            });
                                            request_pointer_capture = Some(Some(dock_space_node));
                                            request_cursor = Some(fret_core::CursorIcon::Default);
                                            handled = true;
                                        }
                                    }
                                    FloatingHitKind::Body => {}
                                }
                            }

                            let (layout_root, layout_bounds) = layout_context_for_position(
                                &dock.graph,
                                self.window,
                                root,
                                dock_bounds,
                                *position,
                            );
                            let layout = compute_layout_map(
                                &dock.graph,
                                layout_root,
                                layout_bounds,
                                split_handle_gap,
                                split_handle_hit_thickness,
                            );
                            if *button == fret_core::MouseButton::Left {
                                if !handled
                                    && let Some(handle) =
                                        hit_test_split_handle(
                                            &dock.graph,
                                            &layout,
                                            split_handle_gap,
                                            split_handle_hit_thickness,
                                            *position,
                                        )
                                {
                                    let locks = compute_same_axis_locks_for_split_drag(
                                        &dock.graph,
                                        &layout,
                                        handle.split,
                                        handle.axis,
                                        handle.handle_ix,
                                    );
                                    self.divider_drag = Some(DividerDragSession {
                                        handle,
                                        layout_root,
                                        layout_bounds,
                                        locks,
                                    });
                                    request_pointer_capture = Some(Some(dock_space_node));
                                    request_cursor = Some(match handle.axis {
                                        fret_core::Axis::Horizontal => {
                                            fret_core::CursorIcon::ColResize
                                        }
                                        fret_core::Axis::Vertical => {
                                            fret_core::CursorIcon::RowResize
                                        }
                                    });
                                    invalidate_paint = true;
                                    pending_redraws.push(self.window);
                                    handled = true;
                                }
                                if !handled
                                    && let Some((tabs_node, tab_index, panel_key, close)) =
                                        hit_test_tab(
                                            &dock.graph,
                                            &layout,
                                            &self.tab_scroll,
                                            &self.tab_widths,
                                            theme,
                                            *position,
                                        )
                                {
                                    if close {
                                        self.pressed_tab_close =
                                            Some((tabs_node, tab_index, panel_key.clone()));
                                        request_pointer_capture = Some(Some(dock_space_node));
                                        dock.hover = None;
                                        invalidate_paint = true;
                                        pending_redraws.push(self.window);
                                        handled = true;
                                    } else {
                                        self.last_active_tabs = Some(tabs_node);
                                        pending_effects.push(Effect::Dock(DockOp::SetActiveTab {
                                            tabs: tabs_node,
                                            active: tab_index,
                                        }));
                                        request_focus_panel = Some(panel_key.clone());
                                        invalidate_layout = true;
                                        // For tear-off, we want the tab itself to stay under the
                                        // cursor after it becomes index 0 in its own floating window
                                        // after the panel is extracted. Our `DockFloating` windows
                                        // render only a `DockSpace` starting at (0,0), so the correct
                                        // anchor is the *tab-local* grab offset (not the window-local
                                        // cursor position).
                                        let tab_rect = layout
                                            .get(&tabs_node)
                                            .copied()
                                            .and_then(|rect| {
                                                let (bar, _content) = split_tab_bar(rect);
                                                let tab_count = match dock.graph.node(tabs_node) {
                                                    Some(DockNode::Tabs { tabs, .. }) => tabs.len(),
                                                    _ => 0,
                                                };
                                                (tab_index < tab_count).then(|| {
                                                    let scroll = self.tab_scroll_for(tabs_node);
                                                    let (geom, _overflow) = self
                                                        .tab_bar_geometry_for_node(
                                                            theme,
                                                            tabs_node,
                                                            bar,
                                                            tab_count,
                                                        );
                                                    geom.tab_rect(tab_index, scroll)
                                                })
                                            })
                                            .unwrap_or_else(|| {
                                                Rect::new(*position, Size::default())
                                            });
                                        let tab_local = Point::new(
                                            Px((position.x.0 - tab_rect.origin.x.0).max(0.0)),
                                            Px((position.y.0 - tab_rect.origin.y.0).max(0.0)),
                                        );
                                        // If this tab is already in an in-window floating container, prefer moving
                                        // the floating window itself by dragging the tab (imgui/egui parity).
                                        // Hold Alt to force the dock drag behavior (tear-off / docking previews).
                                        if layout_root != root
                                            && !modifiers.alt
                                            && let Some(entry) = dock
                                                .graph
                                                .floating_windows(self.window)
                                                .iter()
                                                .find(|w| w.floating == layout_root)
                                        {
                                            if std::env::var_os("FRET_DOCK_FLOAT_MOVE_DEBUG")
                                                .is_some_and(|v| !v.is_empty())
                                            {
                                                tracing::info!(
                                                    window = ?self.window,
                                                    pointer_id = ?pointer_id,
                                                    floating = ?layout_root,
                                                    start_rect = ?entry.rect,
                                                    start = ?position,
                                                    "floating drag start (tab)"
                                                );
                                            }
                                            self.floating_drag = Some(FloatingDragState {
                                                pointer_id,
                                                floating: layout_root,
                                                grab_offset: Point::new(
                                                    Px(position.x.0 - entry.rect.origin.x.0),
                                                    Px(position.y.0 - entry.rect.origin.y.0),
                                                ),
                                                start_rect: entry.rect,
                                                start: *position,
                                                start_tick: now_tick,
                                                activated: false,
                                                last_debug_frame: None,
                                            });
                                            request_pointer_capture = Some(Some(dock_space_node));
                                            request_cursor =
                                                Some(fret_core::CursorIcon::Default);
                                            dock.hover = None;
                                            invalidate_paint = true;
                                            handled = true;
                                        } else {
                                            self.pending_dock_drags.insert(
                                                pointer_id,
                                                PendingDockDrag {
                                                    start: *position,
                                                    panel: panel_key,
                                                    grab_offset: tab_local,
                                                    start_tick: now_tick,
                                                },
                                            );
                                            request_pointer_capture = Some(Some(dock_space_node));
                                            dock.hover = None;
                                            invalidate_paint = true;
                                            handled = true;
                                        }
                                    }
                                }
                            }

                            if !handled
                                && let Some((tabs_node, tabs_rect, _tab_count)) = {
                                    let mut best: Option<(DockNodeId, Rect, usize, f32)> = None;
                                    for (&node, &rect) in layout.iter() {
                                        let Some(DockNode::Tabs { tabs, .. }) =
                                            dock.graph.node(node)
                                        else {
                                            continue;
                                        };
                                        if tabs.is_empty() || !rect.contains(*position) {
                                            continue;
                                        }
                                        let area = rect.size.width.0 * rect.size.height.0;
                                        match best {
                                            None => best = Some((node, rect, tabs.len(), area)),
                                            Some((_best_node, _best_rect, _best_len, best_area)) => {
                                                if area < best_area {
                                                    best = Some((node, rect, tabs.len(), area));
                                                }
                                            }
                                        }
                                    }
                                    best.map(|(node, rect, len, _)| (node, rect, len))
                                }
                            {
                                let (tab_bar, _content) = split_tab_bar(tabs_rect);
                                if tab_bar.contains(*position)
                                    && !tab_overflow_button_rect(theme, tab_bar)
                                        .contains(*position)
                                {
                                    // If the tab bar belongs to an in-window floating container, prefer moving the
                                    // floating window itself over starting a "tabs group" dock drag.
                                    if let Some(floating) = self
                                        .find_floating_container_for_tabs(&dock.graph, tabs_node)
                                        && let Some(entry) = dock
                                            .graph
                                            .floating_windows(self.window)
                                            .iter()
                                            .find(|w| w.floating == floating)
                                    {
                                        if std::env::var_os("FRET_DOCK_FLOAT_MOVE_DEBUG")
                                            .is_some_and(|v| !v.is_empty())
                                        {
                                            tracing::info!(
                                                window = ?self.window,
                                                pointer_id = ?pointer_id,
                                                floating = ?floating,
                                                start_rect = ?entry.rect,
                                                start = ?position,
                                                "floating drag start (tab bar)"
                                            );
                                        }
                                        self.floating_drag = Some(FloatingDragState {
                                            pointer_id,
                                            floating,
                                            grab_offset: Point::new(
                                                Px(position.x.0 - entry.rect.origin.x.0),
                                                Px(position.y.0 - entry.rect.origin.y.0),
                                            ),
                                            start_rect: entry.rect,
                                            start: *position,
                                            start_tick: now_tick,
                                            activated: false,
                                            last_debug_frame: None,
                                        });
                                        request_pointer_capture = Some(Some(dock_space_node));
                                        dock.hover = None;
                                        invalidate_paint = true;
                                        pending_redraws.push(self.window);
                                        handled = true;
                                    } else {
                                        let tab_local = Point::new(
                                            Px((position.x.0 - tab_bar.origin.x.0).max(0.0)),
                                            Px((position.y.0 - tab_bar.origin.y.0).max(0.0)),
                                        );
                                        self.pending_dock_tabs_drags.insert(
                                            pointer_id,
                                            PendingDockTabsDrag {
                                                start: *position,
                                                tabs: tabs_node,
                                                grab_offset: tab_local,
                                                start_tick: now_tick,
                                            },
                                        );
                                        request_pointer_capture = Some(Some(dock_space_node));
                                        dock.hover = None;
                                        invalidate_paint = true;
                                        pending_redraws.push(self.window);
                                        handled = true;
                                    }
                                }
                            }

                            if !handled
                                && *button == fret_core::MouseButton::Right
                                && let Some((tabs_node, tab_index, panel_key, _close)) =
                                    hit_test_tab(
                                        &dock.graph,
                                        &layout,
                                        &self.tab_scroll,
                                        &self.tab_widths,
                                        theme,
                                        *position,
                                    )
                            {
                                self.last_active_tabs = Some(tabs_node);
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
                                    pixels_per_point,
                                    pointer_id,
                                    *pointer_type,
                                    *position,
                                    ViewportInputKind::PointerDown {
                                        button: *button,
                                        modifiers: *modifiers,
                                        click_count: *click_count,
                                    },
                                ) {
                                    pending_effects.push(Effect::ViewportInput(e));
                                    pending_redraws.push(self.window);
                                }

                                self.viewport_capture.insert(
                                    pointer_id,
                                    ViewportCaptureState {
                                        pointer_id,
                                        hit,
                                        button: *button,
                                        start: *position,
                                        last: *position,
                                        moved: false,
                                    },
                                );
                                request_pointer_capture = Some(Some(dock_space_node));
                            }
                        }
                        fret_core::PointerEvent::Move {
                            position,
                            buttons,
                            modifiers,
                            pointer_type,
                            ..
                        } => {
                            if !self.viewport_capture.is_empty()
                                && !self.viewport_capture.contains_key(&pointer_id)
                            {
                                return;
                            }

                            let float_zone_hovered_now = *pointer_type == fret_core::PointerType::Mouse
                                && !buttons.left
                                && !buttons.right
                                && !buttons.middle
                                && dock_bounds.contains(*position)
                                && float_zone(dock_bounds).contains(*position);
                            if float_zone_hovered_now != self.hovered_float_zone {
                                self.hovered_float_zone = float_zone_hovered_now;
                                invalidate_paint = true;
                                pending_redraws.push(self.window);
                            }
                            if !buttons.left
                                && self
                                    .floating_drag
                                    .as_ref()
                                    .is_some_and(|d| d.pointer_id == pointer_id)
                            {
                                // Defensive: if we missed the corresponding `Up`, don't keep
                                // holding the drag session forever.
                                self.floating_drag = None;
                            }
                            if let Some(drag) = self.floating_drag.as_mut() {
                                if drag.pointer_id == pointer_id && buttons.left {
                                    if !drag.activated {
                                        let activation =
                                            fret_dnd::ActivationConstraint::Distance {
                                                px: docking_interaction_settings
                                                    .tab_drag_threshold
                                                    .0,
                                            };
                                        let should_activate = activation.is_satisfied(
                                            drag.start_tick.0,
                                            now_tick.0,
                                            drag.start,
                                            *position,
                                        );
                                        if should_activate {
                                            drag.activated = true;
                                            if std::env::var_os("FRET_DOCK_FLOAT_MOVE_DEBUG")
                                                .is_some_and(|v| !v.is_empty())
                                            {
                                                tracing::info!(
                                                    window = ?self.window,
                                                    pointer_id = ?pointer_id,
                                                    floating = ?drag.floating,
                                                    "floating drag activated"
                                                );
                                            }
                                        }
                                    }
                                    if drag.activated {
                                        let desired = Rect::new(
                                            Point::new(
                                                Px(position.x.0 - drag.grab_offset.x.0),
                                                Px(position.y.0 - drag.grab_offset.y.0),
                                            ),
                                            drag.start_rect.size,
                                        );
                                        let rect =
                                            Self::clamp_rect_to_bounds(desired, window_bounds);
                                        pending_effects
                                            .push(Effect::Dock(DockOp::SetFloatingRect {
                                                window: self.window,
                                                floating: drag.floating,
                                                rect,
                                            }));
                                        if std::env::var_os("FRET_DOCK_FLOAT_MOVE_DEBUG")
                                            .is_some_and(|v| !v.is_empty())
                                            && drag.last_debug_frame != Some(now_frame)
                                        {
                                            drag.last_debug_frame = Some(now_frame);
                                            tracing::info!(
                                                window = ?self.window,
                                                pointer_id = ?pointer_id,
                                                floating = ?drag.floating,
                                                desired = ?desired,
                                                rect = ?rect,
                                                bounds = ?window_bounds,
                                                "floating drag move"
                                            );
                                        }
                                        invalidate_layout = true;
                                        invalidate_paint = true;
                                        pending_redraws.push(self.window);
                                    }
                                    request_cursor = Some(fret_core::CursorIcon::Default);
                                    stop_propagation = true;
                                }
                            } else if has_pending_dock_drag {
                                if !buttons.left {
                                    self.pending_dock_drags.remove(&pointer_id);
                                    request_pointer_capture = Some(None);
                                } else {
                                    let activation = fret_dnd::ActivationConstraint::Distance {
                                        px: docking_interaction_settings.tab_drag_threshold.0,
                                    };
                                    let should_activate = self
                                        .pending_dock_drags
                                        .get(&pointer_id)
                                        .is_some_and(|pending| {
                                            activation.is_satisfied(
                                                pending.start_tick.0,
                                                now_tick.0,
                                                pending.start,
                                                *position,
                                            )
                                        });
                                    if should_activate
                                        && let Some(pending) =
                                            self.pending_dock_drags.remove(&pointer_id)
                                    {
                                        let wants_dock_previews = docking_interaction_settings
                                            .drag_inversion
                                            .wants_dock_previews(*modifiers);
                                        if std::env::var_os("FRET_DOCK_DRAG_DEBUG")
                                            .is_some_and(|v| !v.is_empty())
                                        {
                                            tracing::info!(
                                                window = ?self.window,
                                                pointer_id = ?pointer_id,
                                                modifiers = ?modifiers,
                                                wants_dock_previews = wants_dock_previews,
                                                tab_drag_threshold = docking_interaction_settings
                                                    .tab_drag_threshold
                                                    .0,
                                                dock_hint_scale_inner =
                                                    docking_interaction_settings.dock_hint_scale_inner,
                                                dock_hint_scale_outer =
                                                    docking_interaction_settings.dock_hint_scale_outer,
                                                "dock drag activated"
                                            );
                                        }
                                        start_dock_drag = Some((
                                            pending.start,
                                            DockPanelDragPayload {
                                                panel: pending.panel,
                                                grab_offset: pending.grab_offset,
                                                start_tick: pending.start_tick,
                                                tear_off_requested: false,
                                                tear_off_oob_start_frame: None,
                                                dock_previews_enabled: wants_dock_previews,
                                            },
                                            *position,
                                        ));
                                        request_pointer_capture = Some(None);
                                    }
                                }

                                dock.hover = None;
                                invalidate_paint = true;
                                pending_redraws.push(self.window);
                            } else if has_pending_dock_tabs_drag {
                                if !buttons.left {
                                    self.pending_dock_tabs_drags.remove(&pointer_id);
                                    request_pointer_capture = Some(None);
                                } else {
                                    let activation = fret_dnd::ActivationConstraint::Distance {
                                        px: docking_interaction_settings.tab_drag_threshold.0,
                                    };
                                    let should_activate = self
                                        .pending_dock_tabs_drags
                                        .get(&pointer_id)
                                        .is_some_and(|pending| {
                                            activation.is_satisfied(
                                                pending.start_tick.0,
                                                now_tick.0,
                                                pending.start,
                                                *position,
                                            )
                                        });
                                    if should_activate
                                        && let Some(pending) =
                                            self.pending_dock_tabs_drags.remove(&pointer_id)
                                    {
                                        let wants_dock_previews = docking_interaction_settings
                                            .drag_inversion
                                            .wants_dock_previews(*modifiers);
                                        let (tabs, active) = match dock.graph.node(pending.tabs) {
                                            Some(DockNode::Tabs { tabs, active }) => {
                                                (tabs.clone(), *active)
                                            }
                                            _ => (Vec::new(), 0),
                                        };
                                        start_dock_tabs_drag = Some((
                                            pending.start,
                                            DockTabsDragPayload {
                                                source_tabs: pending.tabs,
                                                tabs,
                                                active,
                                                grab_offset: pending.grab_offset,
                                                start_tick: pending.start_tick,
                                                dock_previews_enabled: wants_dock_previews,
                                            },
                                            *position,
                                        ));
                                        request_pointer_capture = Some(None);
                                    }
                                }

                                dock.hover = None;
                                invalidate_paint = true;
                                pending_redraws.push(self.window);
                            } else {
                                let hovered_floating =
                                    hit_test_floating(&dock.graph, self.window, *position);
                                let hovered_title =
                                    hovered_floating.and_then(|(floating, _chrome, kind)| {
                                        (kind == FloatingHitKind::TitleBar).then_some(floating)
                                    });
                                let hovered_close =
                                    hovered_floating.and_then(|(floating, _chrome, kind)| {
                                        if kind == FloatingHitKind::Close {
                                            Some(floating)
                                        } else {
                                            None
                                        }
                                    });
                                if hovered_close != self.hovered_floating_close {
                                    self.hovered_floating_close = hovered_close;
                                    invalidate_paint = true;
                                    pending_redraws.push(self.window);
                                }
                                if hovered_title != self.hovered_floating_title_bar {
                                    self.hovered_floating_title_bar = hovered_title;
                                    invalidate_paint = true;
                                    pending_redraws.push(self.window);
                                }
                                if request_cursor.is_none()
                                    && let Some((_floating, _chrome, kind)) = hovered_floating
                                {
                                    match kind {
                                        FloatingHitKind::Close => {
                                            request_cursor = Some(fret_core::CursorIcon::Pointer);
                                        }
                                        FloatingHitKind::TitleBar => {
                                            request_cursor = Some(fret_core::CursorIcon::Default);
                                        }
                                        FloatingHitKind::Body => {}
                                    }
                                }

                                if self.viewport_capture.is_empty()
                                    && self.divider_drag.is_none()
                                    && dock_drag.is_none()
                                {
                                    let (_chrome, dock_bounds) =
                                        dock_space_regions(self.last_bounds);
                                    let (layout_root, layout_bounds) = layout_context_for_position(
                                        &dock.graph,
                                        self.window,
                                        root,
                                        dock_bounds,
                                        *position,
                                    );
                                    let layout = compute_layout_map(
                                        &dock.graph,
                                        layout_root,
                                        layout_bounds,
                                        split_handle_gap,
                                        split_handle_hit_thickness,
                                    );
                                    if let Some(handle) =
                                        hit_test_split_handle(
                                            &dock.graph,
                                            &layout,
                                            split_handle_gap,
                                            split_handle_hit_thickness,
                                            *position,
                                        )
                                    {
                                        request_cursor = Some(match handle.axis {
                                            fret_core::Axis::Horizontal => {
                                                fret_core::CursorIcon::ColResize
                                            }
                                            fret_core::Axis::Vertical => {
                                                fret_core::CursorIcon::RowResize
                                            }
                                        });
                                    }
                                }

                                if self.viewport_capture.is_empty()
                                    && self.divider_drag.is_none()
                                    && dock_drag.is_none()
                                {
                                    let (_chrome, dock_bounds) =
                                        dock_space_regions(self.last_bounds);
                                    let (layout_root, layout_bounds) = layout_context_for_position(
                                        &dock.graph,
                                        self.window,
                                        root,
                                        dock_bounds,
                                        *position,
                                    );
                                    let layout = compute_layout_map(
                                        &dock.graph,
                                        layout_root,
                                        layout_bounds,
                                        split_handle_gap,
                                        split_handle_hit_thickness,
                                    );

                                    let hovered = hit_test_tab(
                                        &dock.graph,
                                        &layout,
                                        &self.tab_scroll,
                                        &self.tab_widths,
                                        theme,
                                        *position,
                                    )
                                    .map(|(node, idx, _panel, close)| (node, idx, close));

                                    let next_tab = hovered.map(|(node, idx, _close)| (node, idx));
                                    let next_close = hovered
                                        .map(|(_node, _idx, close)| close)
                                        .unwrap_or(false);
                                    if next_tab != self.hovered_tab
                                        || next_close != self.hovered_tab_close
                                    {
                                        self.hovered_tab = next_tab;
                                        self.hovered_tab_close = next_close;
                                        invalidate_paint = true;
                                        pending_redraws.push(self.window);
                                    }

                                    let mut next_overflow_button: Option<DockNodeId> = None;
                                    for (&node_id, &rect) in layout.iter() {
                                        let Some(DockNode::Tabs { tabs, .. }) =
                                            dock.graph.node(node_id)
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
                                        let (_geom, overflow) = self.tab_bar_geometry_for_node(
                                            theme,
                                            node_id,
                                            tab_bar,
                                            tabs.len(),
                                        );
                                        if !overflow {
                                            continue;
                                        }
                                        if tab_overflow_button_rect(theme, tab_bar)
                                            .contains(*position)
                                        {
                                            next_overflow_button = Some(node_id);
                                            request_cursor =
                                                Some(fret_core::CursorIcon::Pointer);
                                            break;
                                        }
                                    }
                                    if next_overflow_button != self.hovered_tab_overflow_button {
                                        self.hovered_tab_overflow_button = next_overflow_button;
                                        invalidate_paint = true;
                                        pending_redraws.push(self.window);
                                    }

                                    if let Some(mut menu) = self.tab_overflow_menu {
                                        let mut close_menu = false;
                                        if let Some(&tabs_rect) = layout.get(&menu.tabs) {
                                            if let Some(DockNode::Tabs { tabs, .. }) =
                                                dock.graph.node(menu.tabs)
                                            {
                                                let (tab_bar, _content) = split_tab_bar(tabs_rect);
                                                let menu_rect = tab_overflow_menu_rect(
                                                    theme,
                                                    tab_bar,
                                                    tabs.len(),
                                                );
                                                let next_hovered = if menu_rect.contains(*position)
                                                {
                                                    overflow_menu_row_at_pos(
                                                        menu_rect,
                                                        tab_bar,
                                                        tabs.len(),
                                                        menu.scroll,
                                                        *position,
                                                    )
                                                } else {
                                                    None
                                                };
                                                if next_hovered != menu.hovered {
                                                    menu.hovered = next_hovered;
                                                    invalidate_paint = true;
                                                    pending_redraws.push(self.window);
                                                }
                                                if menu_rect.contains(*position) {
                                                    request_cursor =
                                                        Some(fret_core::CursorIcon::Pointer);
                                                }
                                            } else {
                                                close_menu = true;
                                            }
                                        } else if menu.hovered.take().is_some() {
                                            invalidate_paint = true;
                                            pending_redraws.push(self.window);
                                        }

                                        if close_menu {
                                            self.tab_overflow_menu = None;
                                            invalidate_paint = true;
                                            pending_redraws.push(self.window);
                                        } else {
                                            self.tab_overflow_menu = Some(menu);
                                        }
                                    }
                                } else {
                                    if self.hovered_tab.is_some()
                                        || self.hovered_tab_close
                                        || self.hovered_tab_overflow_button.is_some()
                                    {
                                        self.hovered_tab = None;
                                        self.hovered_tab_close = false;
                                        self.hovered_tab_overflow_button = None;
                                        invalidate_paint = true;
                                        pending_redraws.push(self.window);
                                    }
                                }

                                if let Some(divider) = self.divider_drag.as_ref() {
                                    request_cursor = Some(match divider.handle.axis {
                                        fret_core::Axis::Horizontal => {
                                            fret_core::CursorIcon::ColResize
                                        }
                                        fret_core::Axis::Vertical => {
                                            fret_core::CursorIcon::RowResize
                                        }
                                    });
                                    let Some((children_len, fractions_now)) = dock
                                        .graph
                                        .node(divider.handle.split)
                                        .and_then(|n| match n {
                                            DockNode::Split {
                                                children,
                                                fractions,
                                                ..
                                            } => Some((children.len(), fractions.clone())),
                                            _ => None,
                                        })
                                    else {
                                        return;
                                    };
                                    if let Some(next) = resizable::drag_update_fractions(
                                        divider.handle.axis,
                                        divider.handle.bounds,
                                        children_len,
                                        &fractions_now,
                                        divider.handle.handle_ix,
                                        split_handle_gap,
                                        split_handle_hit_thickness,
                                        &[],
                                        divider.handle.grab_offset,
                                        *position,
                                    ) {
                                        dock.graph
                                            .update_split_fractions(divider.handle.split, next);
                                        apply_same_axis_locks(
                                            &mut dock.graph,
                                            divider.layout_root,
                                            divider.layout_bounds,
                                            divider.handle.axis,
                                            split_handle_gap,
                                            split_handle_hit_thickness,
                                            &divider.locks,
                                        );
                                        invalidate_layout = true;
                                        invalidate_paint = true;
                                    }
                                }

                                if let Some(capture) = self.viewport_capture.get_mut(&pointer_id) {
                                    capture.last = *position;
                                    if !capture.moved
                                        && capture.button == fret_core::MouseButton::Right
                                    {
                                        let dx = position.x.0 - capture.start.x.0;
                                        let dy = position.y.0 - capture.start.y.0;
                                        let dist2 = dx * dx + dy * dy;
                                        let threshold = docking_interaction_settings
                                            .viewport_context_menu_drag_threshold
                                            .0
                                            .max(0.0);
                                        if dist2 >= threshold * threshold {
                                            capture.moved = true;
                                        }
                                    }

                                    let hit = capture.hit.clone();
                                    let e = viewport_input_from_hit_clamped(
                                        self.window,
                                        hit,
                                        pixels_per_point,
                                        pointer_id,
                                        *pointer_type,
                                        *position,
                                        ViewportInputKind::PointerMove {
                                            buttons: *buttons,
                                            modifiers: *modifiers,
                                        },
                                    );
                                    pending_effects.push(Effect::ViewportInput(e));
                                    pending_redraws.push(self.window);
                                } else {
                                    let (_chrome, dock_bounds) =
                                        dock_space_regions(self.last_bounds);
                                    let (layout_root, layout_bounds) = layout_context_for_position(
                                        &dock.graph,
                                        self.window,
                                        root,
                                        dock_bounds,
                                        *position,
                                    );
                                    let pointer_occlusion_blocks_mouse = matches!(
                                        pointer_occlusion,
                                        fret_runtime::WindowPointerOcclusion::BlockMouse
                                            | fret_runtime::WindowPointerOcclusion::BlockMouseExceptScroll
                                    );
                                    if allow_viewport_hover
                                        && !(pointer_occlusion_blocks_mouse
                                            && *pointer_type == fret_core::PointerType::Mouse)
                                        && layout_bounds.contains(*position)
                                    {
                                        let layout = compute_layout_map(
                                            &dock.graph,
                                            layout_root,
                                            layout_bounds,
                                            split_handle_gap,
                                            split_handle_hit_thickness,
                                        );
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
                                                pixels_per_point,
                                                pointer_id,
                                                *pointer_type,
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
                            }

                            // Dock/tab dragging uses `Event::InternalDrag` so it can work across
                            // windows without relying on pointer-event broadcasting. Drag session
                            // creation is delayed until activation crosses the drag threshold.
                        }
                        fret_core::PointerEvent::Wheel {
                            position,
                            delta,
                            modifiers,
                            pointer_type,
                            ..
                        } => {
                            let bounds = self.last_bounds;
                            if !bounds.contains(*position) {
                                return;
                            }
                            if dock_drag.is_some() || has_pending_dock_drag || has_pending_dock_tabs_drag {
                                return;
                            }
                            if pointer_occlusion == fret_runtime::WindowPointerOcclusion::BlockMouse
                                && *pointer_type == fret_core::PointerType::Mouse
                            {
                                return;
                            }
                            let (_chrome, dock_bounds) = dock_space_regions(self.last_bounds);
                            let (layout_root, layout_bounds) = layout_context_for_position(
                                &dock.graph,
                                self.window,
                                root,
                                dock_bounds,
                                *position,
                            );
                            let layout = compute_layout_map(
                                &dock.graph,
                                layout_root,
                                layout_bounds,
                                split_handle_gap,
                                split_handle_hit_thickness,
                            );

                            if let Some(mut menu) = self.tab_overflow_menu
                                && let Some(&tabs_rect) = layout.get(&menu.tabs)
                                && let Some(DockNode::Tabs { tabs, .. }) = dock.graph.node(menu.tabs)
                            {
                                let (tab_bar, _content) = split_tab_bar(tabs_rect);
                                let menu_rect =
                                    tab_overflow_menu_rect(theme, tab_bar, tabs.len());
                                if menu_rect.contains(*position) {
                                    let max_scroll = overflow_menu_max_scroll(tab_bar, tabs.len());
                                    let wheel = delta.x.0 + delta.y.0;
                                    let next =
                                        Px((menu.scroll.0 - wheel).clamp(0.0, max_scroll.0));
                                    if (next.0 - menu.scroll.0).abs() >= 0.01 {
                                        menu.scroll = next;
                                        menu.hovered = overflow_menu_row_at_pos(
                                            menu_rect,
                                            tab_bar,
                                            tabs.len(),
                                            menu.scroll,
                                            *position,
                                        );
                                        self.tab_overflow_menu = Some(menu);
                                        invalidate_paint = true;
                                        pending_redraws.push(self.window);
                                    }
                                    stop_propagation = true;
                                    return;
                                }
                            }
                            let mut scrolled_tabs = false;
                            for (&node_id, &rect) in &layout {
                                let Some(DockNode::Tabs { tabs, active }) =
                                    dock.graph.node(node_id)
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
                                    theme,
                                    node_id,
                                    tab_bar,
                                    tabs.len(),
                                    *active,
                                );

                                let max_scroll =
                                    self.max_tab_scroll(theme, node_id, tab_bar, tabs.len());
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
                                    pixels_per_point,
                                    pointer_id,
                                    *pointer_type,
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
                        fret_core::PointerEvent::PinchGesture { .. } => {}
                        fret_core::PointerEvent::Up {
                            position,
                            button,
                            modifiers,
                            is_click,
                            click_count,
                            pointer_type,
                            ..
                        } => {
                            if docking_interaction_settings
                                .suppress_context_menu_during_viewport_capture
                                && *button == fret_core::MouseButton::Right
                                && self
                                    .viewport_capture
                                    .values()
                                    .next()
                                    .is_some_and(|c| c.button != fret_core::MouseButton::Right)
                            {
                                stop_propagation = true;
                                return;
                            }

                            let mut handled = false;
                            if *button == fret_core::MouseButton::Left && has_pending_dock_drag {
                                self.pending_dock_drags.remove(&pointer_id);
                                request_pointer_capture = Some(None);
                                dock.hover = None;
                                invalidate_paint = true;
                                pending_redraws.push(self.window);
                                handled = true;
                            }
                            if *button == fret_core::MouseButton::Left && has_pending_dock_tabs_drag {
                                self.pending_dock_tabs_drags.remove(&pointer_id);
                                request_pointer_capture = Some(None);
                                dock.hover = None;
                                invalidate_paint = true;
                                pending_redraws.push(self.window);
                                handled = true;
                            }
                            if *button == fret_core::MouseButton::Left
                                && self.floating_drag.take().is_some()
                            {
                                request_pointer_capture = Some(None);
                                invalidate_paint = true;
                                pending_redraws.push(self.window);
                                handled = true;
                            }

                            if *button == fret_core::MouseButton::Left
                                && let Some(floating) = self.pressed_floating_close.take()
                            {
                                request_pointer_capture = Some(None);
                                self.hovered_floating_close = None;

                                let clicked =
                                    hit_test_floating(&dock.graph, self.window, *position)
                                        .is_some_and(|(f, _chrome, kind)| {
                                            f == floating && kind == FloatingHitKind::Close
                                        });
                                if clicked
                                    && let Some(target_tabs) =
                                        dock.graph.first_tabs_in_window(self.window)
                                {
                                    pending_effects.push(Effect::Dock(DockOp::MergeFloatingInto {
                                        window: self.window,
                                        floating,
                                        target_tabs,
                                    }));
                                    invalidate_layout = true;
                                }

                                invalidate_paint = true;
                                pending_redraws.push(self.window);
                                handled = true;
                            }

                            if *button == fret_core::MouseButton::Left
                                && let Some(divider) = self.divider_drag.take()
                            {
                                let mut seen: std::collections::HashSet<DockNodeId> =
                                    std::collections::HashSet::new();
                                let mut updates: Vec<fret_core::SplitFractionsUpdate> = Vec::new();
                                for split in std::iter::once(divider.handle.split)
                                    .chain(divider.locks.iter().map(|l| l.split))
                                {
                                    if !seen.insert(split) {
                                        continue;
                                    }
                                    if let Some(DockNode::Split {
                                        children,
                                        fractions,
                                        ..
                                    }) = dock.graph.node(split)
                                        && children.len() >= 2
                                        && children.len() == fractions.len()
                                    {
                                        updates.push(fret_core::SplitFractionsUpdate {
                                            split,
                                            fractions: fractions.clone(),
                                        });
                                    }
                                }

                                if !updates.is_empty() {
                                    pending_effects.push(Effect::Dock(
                                        DockOp::SetSplitFractionsMany { updates },
                                    ));
                                }

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
                                let mut layout = compute_layout_map(
                                    &dock.graph,
                                    root,
                                    dock_bounds,
                                    split_handle_gap,
                                    split_handle_hit_thickness,
                                );
                                if !layout.contains_key(&tabs_node) {
                                    for floating in dock.graph.floating_windows(self.window) {
                                        let chrome = Self::floating_chrome(floating.rect);
                                        let l = compute_layout_map(
                                            &dock.graph,
                                            floating.floating,
                                            chrome.inner,
                                            split_handle_gap,
                                            split_handle_hit_thickness,
                                        );
                                        if l.contains_key(&tabs_node) {
                                            layout = l;
                                            break;
                                        }
                                    }
                                }
                                let clicked = hit_test_tab(
                                    &dock.graph,
                                    &layout,
                                    &self.tab_scroll,
                                    &self.tab_widths,
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
                                    .get(&pointer_id)
                                    .is_some_and(|c| c.button == *button);
                                if released_capture {
                                    let capture =
                                        self.viewport_capture.remove(&pointer_id).unwrap();
                                    let e = viewport_input_from_hit_clamped(
                                        self.window,
                                        capture.hit.clone(),
                                        pixels_per_point,
                                        pointer_id,
                                        *pointer_type,
                                        *position,
                                        ViewportInputKind::PointerUp {
                                            button: *button,
                                            modifiers: *modifiers,
                                            is_click: *is_click,
                                            click_count: *click_count,
                                        },
                                    );
                                    pending_effects.push(Effect::ViewportInput(e));
                                    pending_redraws.push(self.window);

                                    dock.hover = None;
                                    request_pointer_capture = Some(None);
                                    invalidate_paint = true;
                                    if docking_interaction_settings
                                        .suppress_context_menu_during_viewport_capture
                                        && capture.button == fret_core::MouseButton::Right
                                        && capture.moved
                                    {
                                        stop_propagation = true;
                                    }
                                }

                                if !released_capture
                                    && *button == fret_core::MouseButton::Left
                                    && let Some(drag) = dock_drag.as_ref()
                                {
                                    let dragging = match drag {
                                        DockDragSnapshot::Panel(d) => d.dragging,
                                        DockDragSnapshot::Tabs(d) => d.dragging,
                                    };
                                    if dragging {
                                        let target = dock.hover.clone().or_else(|| {
                                            (!window_bounds.contains(*position)
                                                || float_zone(dock_bounds).contains(*position))
                                            .then_some(DockDropTarget::Float {
                                                window: self.window,
                                            })
                                        });

                                        let intent = match drag {
                                            DockDragSnapshot::Panel(drag) => {
                                                resolve_dock_drop_intent_panel(
                                                    target,
                                                    drag,
                                                    self.window,
                                                    window_bounds,
                                                    *position,
                                                    allow_tear_off,
                                                    mark_drag_tear_off_requested,
                                                    |panel, position, grab_offset, window_bounds| {
                                                        self.default_floating_rect_for_panel(
                                                            panel,
                                                            position,
                                                            grab_offset,
                                                            window_bounds,
                                                        )
                                                    },
                                                )
                                            }
                                            DockDragSnapshot::Tabs(drag) => resolve_dock_drop_intent_tabs(
                                                target,
                                                drag,
                                                self.window,
                                                window_bounds,
                                                *position,
                                                |panel, position, grab_offset, window_bounds| {
                                                    self.default_floating_rect_for_panel(
                                                        panel,
                                                        position,
                                                        grab_offset,
                                                        window_bounds,
                                                    )
                                                },
                                            ),
                                        };

                                        if matches!(
                                            intent,
                                            DockDropIntent::RequestFloatPanelToNewWindow { .. }
                                        ) {
                                            mark_drag_tear_off_requested = true;
                                        }

                                        apply_dock_drop_intent(
                                            intent,
                                            &mut pending_effects,
                                            &mut invalidate_layout,
                                        );
                                    }

                                    dock.hover = None;
                                    end_dock_drag = true;
                                    invalidate_paint = true;
                                } else if !released_capture {
                                    let (_chrome, dock_bounds) =
                                        dock_space_regions(self.last_bounds);
                                    if dock_bounds.contains(*position) {
                                        let layout = compute_layout_map(
                                            &dock.graph,
                                            root,
                                            dock_bounds,
                                            split_handle_gap,
                                            split_handle_hit_thickness,
                                        );
                                        if let Some(hit) = hit_test_active_viewport_panel(
                                            &dock.graph,
                                            &dock.panels,
                                            &layout,
                                            *position,
                                        ) && let Some(e) = viewport_input_from_hit(
                                            self.window,
                                            hit,
                                            pixels_per_point,
                                            pointer_id,
                                            *pointer_type,
                                            *position,
                                            ViewportInputKind::PointerUp {
                                                button: *button,
                                                modifiers: *modifiers,
                                                is_click: *is_click,
                                                click_count: *click_count,
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
                            fret_core::InternalDragKind::Enter
                            | fret_core::InternalDragKind::Over => {
                                if let Some(drag) = dock_drag.as_ref() {
                                    let prev_hover = dock.hover.clone();
                                    let (invert_docking, source_window, start_tick, start_pos) =
                                        match drag {
                                            DockDragSnapshot::Panel(d) => (
                                                !d.dock_previews_enabled,
                                                d.source_window,
                                                d.start_tick,
                                                d.start,
                                            ),
                                            DockDragSnapshot::Tabs(d) => (
                                                !d.dock_previews_enabled,
                                                d.source_window,
                                                d.start_tick,
                                                d.start,
                                            ),
                                        };
                                    let mut dragging = match drag {
                                        DockDragSnapshot::Panel(d) => d.dragging,
                                        DockDragSnapshot::Tabs(d) => d.dragging,
                                    };

                                    if source_window == self.window {
                                        // Use the default drag threshold (~6 logical px).
                                        let activation = fret_dnd::ActivationConstraint::Distance {
                                            px: docking_interaction_settings.tab_drag_threshold.0,
                                        };
                                        if !dragging
                                            && activation.is_satisfied(
                                                start_tick.0,
                                                now_tick.0,
                                                start_pos,
                                                position,
                                            )
                                        {
                                            dragging = true;
                                        }
                                    } else if !dragging {
                                        dragging = true;
                                    }

                                    update_drag = Some((position, dragging));

                                    if dragging {
                                        let requested_tear_off = match drag {
                                            DockDragSnapshot::Panel(drag) => {
                                                let margin = Px(10.0);
                                                let oob = is_outside_bounds_with_margin(
                                                    window_bounds,
                                                    position,
                                                    margin,
                                                );
                                                if allow_tear_off && drag.source_window == self.window
                                                {
                                                    match (oob, drag.tear_off_oob_start_frame) {
                                                        (true, None) => {
                                                            set_drag_tear_off_oob_start_frame =
                                                                Some(Some(now_frame));
                                                        }
                                                        (false, Some(_)) => {
                                                            set_drag_tear_off_oob_start_frame =
                                                                Some(None);
                                                        }
                                                        _ => {}
                                                    }
                                                }

                                                let stable_oob = oob
                                                    && drag
                                                        .tear_off_oob_start_frame
                                                        .is_some_and(|f| f != now_frame);
                                                let requested_tear_off = allow_tear_off
                                                    && drag.source_window == self.window
                                                    && stable_oob
                                                    && !drag.tear_off_requested
                                                    && !mark_drag_tear_off_requested;

                                                if requested_tear_off {
                                                    mark_drag_tear_off_requested = true;
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

                                                requested_tear_off
                                            }
                                            DockDragSnapshot::Tabs(_) => false,
                                        };

                                        if !requested_tear_off {
                                            let mut candidates =
                                                Vec::<fret_runtime::DockDropCandidateRectDiagnostics>::new();
	                                            let (hover, source) = resolve_dock_drop_target(
	                                                None,
	                                                invert_docking,
	                                                self.window,
	                                                &dock.graph,
	                                                root,
	                                                dock_bounds,
	                                                window_bounds,
	                                                &self.tab_scroll,
	                                                &self.tab_widths,
	                                                hint_font_size_inner,
	                                                hint_font_size_outer,
	                                                split_handle_gap,
	                                                split_handle_hit_thickness,
	                                                position,
	                                                diagnostics_enabled.then_some(&mut candidates),
	                                            );
                                            dock.hover = hover;
                                            if diagnostics_enabled {
                                                self.dock_drop_resolve_diagnostics = Some((
                                                    now_frame,
                                                    compute_dock_drop_resolve_diagnostics(
                                                        e.pointer_id,
                                                        position,
                                                        window_bounds,
                                                        dock_bounds,
                                                        source,
                                                        dock.hover.as_ref(),
                                                        candidates,
                                                    ),
                                                ));
                                            }
                                            if std::env::var_os("FRET_DOCK_DRAG_DEBUG")
                                                .is_some_and(|v| !v.is_empty())
                                                && dock.hover != prev_hover
                                            {
                                                tracing::info!(
                                                    window = ?self.window,
                                                    invert_docking = invert_docking,
                                                    source = ?source,
                                                    target = ?dock_drop_target_diagnostics(
                                                        dock.hover.as_ref()
                                                    ),
                                                    "dock drag hover changed"
                                                );
                                            }

                                            if let Some(DockDropTarget::Dock(target)) =
                                                dock.hover.as_mut()
                                                && matches!(target.zone, DropZone::Center)
                                                && target.insert_index.is_some()
                                                && !target.outer
                                                && let Some(DockNode::Tabs { tabs, .. }) =
                                                    dock.graph.node(target.tabs)
                                            {
                                                let (layout_root, layout_bounds) =
                                                    layout_context_for_position(
                                                        &dock.graph,
                                                        self.window,
                                                        root,
                                                        dock_bounds,
                                                        position,
                                                    );
                                                let layout = compute_layout_map(
                                                    &dock.graph,
                                                    layout_root,
                                                    layout_bounds,
                                                    split_handle_gap,
                                                    split_handle_hit_thickness,
                                                );
                                                if let Some(&tabs_rect) = layout.get(&target.tabs)
                                                {
                                                    let (tab_bar, _content) =
                                                        split_tab_bar(tabs_rect);
                                                    if self.apply_tab_bar_drag_auto_scroll(
                                                        theme,
                                                        &dock.graph,
                                                        target,
                                                        tab_bar,
                                                        tabs.len(),
                                                        font_size,
                                                        position,
                                                        now_frame,
                                                    ) {
                                                        pending_redraws.push(self.window);
                                                        invalidate_paint = true;
                                                    }
                                                }
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
                                self.dock_drop_resolve_diagnostics = None;
                            }
                            fret_core::InternalDragKind::Drop => {
                                let prev_hover = dock.hover.clone();
                                if let Some(drag) = dock_drag.as_ref() {
                                    let invert_docking = match drag {
                                        DockDragSnapshot::Panel(d) => !d.dock_previews_enabled,
                                        DockDragSnapshot::Tabs(d) => !d.dock_previews_enabled,
                                    };
                                    let mut dragging = match drag {
                                        DockDragSnapshot::Panel(d) => d.dragging,
                                        DockDragSnapshot::Tabs(d) => d.dragging,
                                    };
                                    let source_window = match drag {
                                        DockDragSnapshot::Panel(d) => d.source_window,
                                        DockDragSnapshot::Tabs(d) => d.source_window,
                                    };
                                    if !dragging && source_window != self.window {
                                        dragging = true;
                                    }

                                    if dragging {
                                        let mut candidates =
                                            Vec::<fret_runtime::DockDropCandidateRectDiagnostics>::new();
	                                        let (target, source) = resolve_dock_drop_target(
	                                            prev_hover.clone(),
	                                            invert_docking,
	                                            self.window,
	                                            &dock.graph,
	                                            root,
	                                            dock_bounds,
	                                            window_bounds,
	                                            &self.tab_scroll,
	                                            &self.tab_widths,
	                                            hint_font_size_inner,
	                                            hint_font_size_outer,
	                                            split_handle_gap,
	                                            split_handle_hit_thickness,
	                                            position,
	                                            diagnostics_enabled.then_some(&mut candidates),
	                                        );
                                        if diagnostics_enabled {
                                            self.dock_drop_resolve_diagnostics = Some((
                                                now_frame,
                                                compute_dock_drop_resolve_diagnostics(
                                                    e.pointer_id,
                                                    position,
                                                    window_bounds,
                                                    dock_bounds,
                                                    source,
                                                    target.as_ref(),
                                                    candidates,
                                                ),
                                            ));
                                        }
                                        let intent = match drag {
                                            DockDragSnapshot::Panel(drag) => {
                                                resolve_dock_drop_intent_panel(
                                                    target,
                                                    drag,
                                                    self.window,
                                                    window_bounds,
                                                    position,
                                                    allow_tear_off,
                                                    mark_drag_tear_off_requested,
                                                    |panel, position, grab_offset, window_bounds| {
                                                        self.default_floating_rect_for_panel(
                                                            panel,
                                                            position,
                                                            grab_offset,
                                                            window_bounds,
                                                        )
                                                    },
                                                )
                                            }
                                            DockDragSnapshot::Tabs(drag) => resolve_dock_drop_intent_tabs(
                                                target,
                                                drag,
                                                self.window,
                                                window_bounds,
                                                position,
                                                |panel, position, grab_offset, window_bounds| {
                                                    self.default_floating_rect_for_panel(
                                                        panel,
                                                        position,
                                                        grab_offset,
                                                        window_bounds,
                                                    )
                                                },
                                            ),
                                        };

                                        if matches!(
                                            intent,
                                            DockDropIntent::RequestFloatPanelToNewWindow { .. }
                                        ) {
                                            mark_drag_tear_off_requested = true;
                                        }

                                        apply_dock_drop_intent(
                                            intent,
                                            &mut pending_effects,
                                            &mut invalidate_layout,
                                        );
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
                    fret_core::Event::PointerCancel(e) => {
                        if self.pending_dock_drags.remove(&e.pointer_id).is_some()
                            || self.pending_dock_tabs_drags.remove(&e.pointer_id).is_some()
                        {
                            request_pointer_capture = Some(None);
                            dock.hover = None;
                            invalidate_paint = true;
                            pending_redraws.push(self.window);
                        }
                        if let Some(capture) = self.viewport_capture.remove(&e.pointer_id) {
                            let position = e.position.unwrap_or(capture.last);
                            let evt = viewport_input_from_hit_clamped(
                                self.window,
                                capture.hit,
                                pixels_per_point,
                                e.pointer_id,
                                e.pointer_type,
                                position,
                                ViewportInputKind::PointerCancel {
                                    buttons: e.buttons,
                                    modifiers: e.modifiers,
                                    reason: e.reason,
                                },
                            );
                            pending_effects.push(Effect::ViewportInput(evt));
                            pending_redraws.push(self.window);

                            request_pointer_capture = Some(None);
                            dock.hover = None;
                            invalidate_paint = true;
                            stop_propagation = true;
                        }
                        if dock_drag.is_some() {
                            dock.hover = None;
                            end_dock_drag = true;
                            invalidate_paint = true;
                            pending_redraws.push(self.window);
                        }
                    }
                    _ => {}
                }
            });

        if request_focus.is_none()
            && let Some(panel) = request_focus_panel
        {
            let panel_nodes = self.panel_nodes(cx.app);
            request_focus = panel_nodes.get(&panel).copied();
        }

        if let Some((start, payload, position)) = start_dock_drag {
            cx.app.begin_cross_window_drag_with_kind(
                pointer_id,
                fret_runtime::DRAG_KIND_DOCK_PANEL,
                self.window,
                start,
                payload,
            );
            if let Some(drag) = cx.app.drag_mut(pointer_id) {
                drag.position = position;
                drag.dragging = true;
            }
        }
        if let Some((start, payload, position)) = start_dock_tabs_drag {
            cx.app.begin_cross_window_drag_with_kind(
                pointer_id,
                fret_runtime::DRAG_KIND_DOCK_TABS,
                self.window,
                start,
                payload,
            );
            if let Some(drag) = cx.app.drag_mut(pointer_id) {
                drag.position = position;
                drag.dragging = true;
            }
        }

        if let Some(request) = request_pointer_capture {
            match request {
                Some(node) => cx.capture_pointer(node),
                None => cx.release_pointer_capture(),
            }
        }

        if let Some((position, dragging)) = update_drag
            && let Some(drag) = cx.app.drag_mut(pointer_id)
            && (drag.payload::<DockPanelDragPayload>().is_some()
                || drag.payload::<DockTabsDragPayload>().is_some())
        {
            drag.position = position;
            drag.dragging = dragging;
            if let Some(payload) = drag.payload_mut::<DockPanelDragPayload>() {
                if mark_drag_tear_off_requested {
                    payload.tear_off_requested = true;
                    payload.tear_off_oob_start_frame = None;
                }
                if let Some(next) = set_drag_tear_off_oob_start_frame {
                    payload.tear_off_oob_start_frame = next;
                }
            }
        }

        if end_dock_drag
            && cx.app.drag(pointer_id).is_some_and(|d| {
                d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                    || d.kind == fret_runtime::DRAG_KIND_DOCK_TABS
            })
        {
            cx.app.cancel_drag(pointer_id);
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
        if stop_propagation {
            cx.stop_propagation();
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
        let invalidation_model = DockInvalidationService::model_for_window(cx.app, self.window);
        cx.observe_model(&invalidation_model, Invalidation::Layout);

        self.last_bounds = cx.bounds;
        let docking_interaction_settings = cx
            .app
            .global::<fret_runtime::DockingInteractionSettings>()
            .copied()
            .unwrap_or_default();
        let split_handle_gap = docking_interaction_settings.split_handle_gap;
        let split_handle_hit_thickness = docking_interaction_settings.split_handle_hit_thickness;
        let hidden = hidden_bounds(Size::new(Px(0.0), Px(0.0)));
        let theme = cx.theme().snapshot();

        fret_ui::internal_drag::set_route(
            cx.app,
            self.window,
            fret_runtime::DRAG_KIND_DOCK_PANEL,
            cx.node,
        );
        if cx.app.global::<DockManager>().is_some() {
            cx.app
                .with_global_mut_untracked(DockManager::default, |dock, _app| {
                    dock.register_dock_space_node(self.window, cx.node);
                });
        }

        let Some((active_bounds, layout)) = (|| {
            let dock = cx.app.global::<DockManager>()?;
            let root = dock.graph.window_root(self.window)?;
            let (_chrome, dock_bounds) = dock_space_regions(cx.bounds);
            let mut layout = compute_layout_map(
                &dock.graph,
                root,
                dock_bounds,
                split_handle_gap,
                split_handle_hit_thickness,
            );

            for floating in dock.graph.floating_windows(self.window) {
                let chrome = Self::floating_chrome(floating.rect);
                let floating_layout = compute_layout_map(
                    &dock.graph,
                    floating.floating,
                    chrome.inner,
                    split_handle_gap,
                    split_handle_hit_thickness,
                );
                for (k, v) in floating_layout {
                    layout.insert(k, v);
                }
            }

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
                self.clamp_and_ensure_active_visible(theme, node_id, tab_bar, tabs.len(), *active);
            }
            self.tab_scroll
                .retain(|tabs_node, _| visible_tabs_nodes.contains(tabs_node));
            self.tab_widths
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
            let _ = cx.layout_viewport_root(*node, bounds);
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
        let theme = cx.theme().snapshot();
        let (_chrome, dock_bounds) = dock_space_regions(cx.bounds);
        let font_size = theme.metric_required("font.size");
        // Keep the dock host "alive" as a stable internal drag route target.
        //
        // This must be refreshed during paint/layout, not only during event handling, because
        // cross-window drag routing needs a reliable per-window anchor even when hit-testing is
        // over unrelated UI (ADR 0072) and even when no events have fired this frame.
        fret_ui::internal_drag::set_route(
            cx.app,
            self.window,
            fret_runtime::DRAG_KIND_DOCK_PANEL,
            cx.node,
        );

        // Best-effort diagnostics hook: only record if a diagnostics collector has registered the
        // store (avoids allocating globals in production apps).
        if cx
            .app
            .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
            .is_some()
        {
            let frame_id = cx.app.frame_id();
            let dock_drag_pointer_id = cx.app.find_drag_pointer_id(|d| {
                d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                    && (d.source_window == self.window || d.current_window == self.window)
            });
            let dock_drag = dock_drag_pointer_id.and_then(|pointer_id| {
                let drag = cx.app.drag(pointer_id)?;
                Some(fret_runtime::DockDragDiagnostics {
                    pointer_id,
                    source_window: drag.source_window,
                    current_window: drag.current_window,
                    dragging: drag.dragging,
                    cross_window_hover: drag.cross_window_hover,
                })
            });
            let viewport_capture = self
                .viewport_capture
                .iter()
                .min_by_key(|(pointer_id, _)| pointer_id.0)
                .map(
                    |(_pointer_id, capture)| fret_runtime::ViewportCaptureDiagnostics {
                        pointer_id: capture.pointer_id,
                        target: capture.hit.viewport.target,
                    },
                );
            let dock_drop_resolve = self
                .dock_drop_resolve_diagnostics
                .as_ref()
                .and_then(|(f, d)| (f == &frame_id).then(|| d.clone()));

            cx.app.with_global_mut_untracked(
                fret_runtime::WindowInteractionDiagnosticsStore::default,
                |svc, _app| {
                    svc.record_docking(
                        self.window,
                        frame_id,
                        fret_runtime::DockingInteractionDiagnostics {
                            dock_drag,
                            dock_drop_resolve,
                            viewport_capture,
                        },
                    );
                },
            );
        }
        let overlay_hooks = cx
            .app
            .global::<DockViewportOverlayHooksService>()
            .and_then(|svc| svc.hooks());
        let is_dock_dragging = cx
            .app
            .drag(fret_core::PointerId(0))
            .is_some_and(|d| d.dragging && d.payload::<DockPanelDragPayload>().is_some());
        let wants_animation_frames = is_dock_dragging
            || self.divider_drag.is_some()
            || self.floating_drag.is_some()
            || !self.viewport_capture.is_empty();
        if wants_animation_frames {
            cx.request_animation_frame_paint_only();
        }
        if cx.app.global::<DockManager>().is_none() {
            self.paint_empty_state(cx);
            return;
        }

        let dock_space_node = cx.node;
        let scale_factor = cx.scale_factor;
        let bounds = cx.bounds;
        let app = &mut *cx.app;
        let services = &mut *cx.services;
        let scene = &mut *cx.scene;
        let docking_interaction_settings = app
            .global::<fret_runtime::DockingInteractionSettings>()
            .copied()
            .unwrap_or_default();
        let hint_font_size_inner = Px((font_size.0
            * docking_interaction_settings.dock_hint_scale_inner.max(0.0))
        .max(0.0));
        let hint_font_size_outer = Px((font_size.0
            * docking_interaction_settings.dock_hint_scale_outer.max(0.0))
        .max(0.0));
        let split_handle_gap = docking_interaction_settings.split_handle_gap;
        let split_handle_hit_thickness = docking_interaction_settings.split_handle_hit_thickness;
        let frame_id = app.frame_id();
        let dock_drag_pointer_id = app.find_drag_pointer_id(|d| {
            (d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                || d.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
                && (d.source_window == self.window || d.current_window == self.window)
                && d.dragging
        });
        let dock_drag_panel = dock_drag_pointer_id
            .and_then(|pointer_id| app.drag(pointer_id))
            .and_then(|drag| drag.payload::<DockPanelDragPayload>())
            .map(|payload| payload.panel.clone());
        let dock_drag_pos =
            dock_drag_pointer_id.and_then(|pointer_id| app.drag(pointer_id).map(|d| d.position));
        let dock_drag_source_window = dock_drag_pointer_id
            .and_then(|pointer_id| app.drag(pointer_id).map(|d| d.source_window));
        let dock_drag_source_tabs = dock_drag_pointer_id
            .and_then(|pointer_id| app.drag(pointer_id))
            .and_then(|drag| drag.payload::<DockTabsDragPayload>())
            .map(|payload| payload.source_tabs);

        let caps = app
            .global::<fret_runtime::PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        let tear_off_possible = caps.ui.window_tear_off
            && caps.ui.multi_window
            && caps.ui.window_hover_detection != fret_runtime::WindowHoverDetectionQuality::None;

        let paint_panels = app.with_global_mut_untracked(DockManager::default, |dock, _app| {
            dock.register_dock_space_node(self.window, dock_space_node);
            let Some(root) = dock.graph.window_root(self.window) else {
                return None;
            };

            let root_layout = compute_layout_map(
                &dock.graph,
                root,
                dock_bounds,
                split_handle_gap,
                split_handle_hit_thickness,
            );

            let mut floating_layouts: Vec<(
                fret_core::DockFloatingWindow,
                FloatingChrome,
                HashMap<DockNodeId, Rect>,
            )> = Vec::new();
            let mut layout_all = root_layout.clone();
            for floating in dock.graph.floating_windows(self.window) {
                let chrome = Self::floating_chrome(floating.rect);
                let layout = compute_layout_map(
                    &dock.graph,
                    floating.floating,
                    chrome.inner,
                    split_handle_gap,
                    split_handle_hit_thickness,
                );
                for (k, v) in layout.iter() {
                    layout_all.insert(*k, *v);
                }
                floating_layouts.push((*floating, chrome, layout));
            }

            if let (Some(pos), Some(DockDropTarget::Dock(target))) =
                (dock_drag_pos, dock.hover.as_mut())
                && matches!(target.zone, DropZone::Center)
                && target.insert_index.is_some()
                && !target.outer
                && let Some(DockNode::Tabs { tabs, .. }) = dock.graph.node(target.tabs)
                && let Some(&tabs_rect) = layout_all.get(&target.tabs)
            {
                let (tab_bar, _content) = split_tab_bar(tabs_rect);
                let _ = self.apply_tab_bar_drag_auto_scroll(
                    theme,
                    &dock.graph,
                    target,
                    tab_bar,
                    tabs.len(),
                    font_size,
                    pos,
                    frame_id,
                );
            }

            let hover = dock.hover.clone();

            self.rebuild_tab_titles(
                services,
                theme,
                scale_factor,
                &*dock,
                &layout_all,
                dock_drag_panel.as_ref(),
            );
            let drag_tab_title = dock_drag_panel
                .as_ref()
                .and_then(|panel| self.tab_titles.get(panel).copied());
            let close_glyph_present = self.tab_close_glyph.is_some();
            self.tab_widths.clear();
            for (&node_id, &_rect) in layout_all.iter() {
                let Some(DockNode::Tabs { tabs, .. }) = dock.graph.node(node_id) else {
                    continue;
                };
                if tabs.is_empty() {
                    continue;
                }
                let close_glyph_present = self.tab_close_glyph.is_some();
                let widths: Vec<Px> = tabs
                    .iter()
                    .map(|panel| {
                        let title_width = self
                            .tab_titles
                            .get(panel)
                            .map(|t| t.metrics.size.width)
                            .unwrap_or(Px(0.0));
                        dock_tab_width_for_title(theme, title_width, close_glyph_present)
                    })
                    .collect();
                self.tab_widths.insert(node_id, Arc::from(widths));
            }

            dock.clear_viewport_layout_for_window(self.window);
            for (&node_id, &rect) in layout_all.iter() {
                let (_tab_bar, content) = split_tab_bar(rect);
                let viewport = (|| {
                    let DockNode::Tabs { tabs, active } = dock.graph.node(node_id)?.clone() else {
                        return None;
                    };
                    let panel_key = tabs.get(active)?;
                    let panel = dock.panel(panel_key)?;
                    panel.viewport
                })();
                if let Some(viewport) = viewport {
                    let mapping = ViewportMapping {
                        content_rect: content,
                        target_px_size: viewport.target_px_size,
                        fit: viewport.fit,
                    };
                    dock.set_viewport_layout(
                        self.window,
                        viewport.target,
                        super::DockViewportLayout {
                            content_rect: content,
                            mapping,
                            draw_rect: mapping.map().draw_rect,
                        },
                    );
                }
            }

            paint_dock(
                theme,
                &*dock,
                PaintDockParams {
                    window: self.window,
                    layout: &root_layout,
                    tab_titles: &self.tab_titles,
                    tab_widths: &self.tab_widths,
                    hovered_tab: self.hovered_tab,
                    hovered_tab_close: self.hovered_tab_close,
                    hovered_tab_overflow_button: self.hovered_tab_overflow_button,
                    pressed_tab_close: self.pressed_tab_close.as_ref().map(|(n, i, _)| (*n, *i)),
                    tab_scroll: &self.tab_scroll,
                    tab_close_glyph: self.tab_close_glyph,
                    tab_overflow_glyph: self.tab_overflow_glyph,
                    tab_overflow_menu: self.tab_overflow_menu,
                },
                overlay_hooks.as_deref(),
                scene,
            );

            let mut paint_panels: Vec<(PanelKey, Rect)> =
                active_panel_content_bounds(&dock.graph, &root_layout)
                    .into_iter()
                    .collect();

            for (floating, chrome, layout) in &floating_layouts {
                let border = theme.color_required("border");
                let surface = theme.color_required("background");
                let hover_bg = theme.color_required("accent");
                let fg = theme.color_required("foreground");
                let fg_muted = theme.color_required("muted-foreground");
                let radius_md = theme.metric_required("metric.radius.md");
                let radius_sm = theme.metric_required("metric.radius.sm");

                let border_color = Color { a: 0.85, ..border };
                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(0),
                    rect: chrome.outer,
                    background: surface,
                    border: Edges::all(DOCK_FLOATING_BORDER),
                    border_color,
                    corner_radii: fret_core::Corners::all(Px(radius_md.0.max(6.0))),
                });
                scene.push(SceneOp::Quad {
                    order: fret_core::DrawOrder(1),
                    rect: chrome.title_bar,
                    background: if self.hovered_floating_title_bar == Some(floating.floating) {
                        Color {
                            a: 0.22,
                            ..hover_bg
                        }
                    } else {
                        surface
                    },
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });

                let close_hovered = self.hovered_floating_close == Some(floating.floating);
                let close_pressed = self.pressed_floating_close == Some(floating.floating);
                if close_hovered || close_pressed {
                    scene.push(SceneOp::Quad {
                        order: fret_core::DrawOrder(2),
                        rect: chrome.close_button,
                        background: hover_bg,
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: fret_core::Corners::all(Px(radius_sm.0.max(4.0))),
                    });
                }

                if let Some(glyph) = self.tab_close_glyph {
                    let text_x = Px(chrome.close_button.origin.x.0
                        + (chrome.close_button.size.width.0 - glyph.metrics.size.width.0) * 0.5);
                    let inner_y = chrome.close_button.origin.y.0
                        + ((chrome.close_button.size.height.0 - glyph.metrics.size.height.0) * 0.5);
                    let text_y = Px(inner_y + glyph.metrics.baseline.0);
                    let color = if close_hovered || close_pressed {
                        fg
                    } else {
                        fg_muted
                    };
                    scene.push(SceneOp::Text {
                        order: fret_core::DrawOrder(3),
                        origin: Point::new(text_x, text_y),
                        text: glyph.blob,
                        color,
                    });
                }

                paint_dock(
                    theme,
                    &*dock,
                    PaintDockParams {
                        window: self.window,
                        layout,
                        tab_titles: &self.tab_titles,
                        tab_widths: &self.tab_widths,
                        hovered_tab: self.hovered_tab,
                        hovered_tab_close: self.hovered_tab_close,
                        hovered_tab_overflow_button: self.hovered_tab_overflow_button,
                        pressed_tab_close: self
                            .pressed_tab_close
                            .as_ref()
                            .map(|(n, i, _)| (*n, *i)),
                        tab_scroll: &self.tab_scroll,
                        tab_close_glyph: self.tab_close_glyph,
                        tab_overflow_glyph: self.tab_overflow_glyph,
                        tab_overflow_menu: self.tab_overflow_menu,
                    },
                    overlay_hooks.as_deref(),
                    scene,
                );

                paint_panels.extend(active_panel_content_bounds(&dock.graph, layout));
            }

            paint_split_handles(
                theme,
                &dock.graph,
                &layout_all,
                self.divider_drag.as_ref().map(|d| d.handle.split),
                split_handle_gap,
                split_handle_hit_thickness,
                scale_factor,
                scene,
            );

            self.paint_float_zone_hint(
                services,
                theme,
                scale_factor,
                dock_bounds,
                tear_off_possible,
                scene,
            );

            let drag_source_tabs_for_preview = dock_drag_source_tabs.or_else(|| {
                dock_drag_panel
                    .as_ref()
                    .zip(dock_drag_source_window)
                    .and_then(|(panel, source_window)| {
                        dock.graph
                            .find_panel_in_window(source_window, panel)
                            .map(|(tabs, _active)| tabs)
                    })
            });

            if is_dock_dragging {
                let suppress_hints_for_tab_reorder = hover.as_ref().is_some_and(|hover| {
                    let DockDropTarget::Dock(target) = hover else {
                        return false;
                    };
                    let is_tab_bar_reorder =
                        target.zone == DropZone::Center && target.insert_index.is_some();
                    let is_same_tabs = drag_source_tabs_for_preview
                        .is_some_and(|source_tabs| source_tabs == target.tabs);
                    is_tab_bar_reorder && is_same_tabs && !target.outer
                });

                if !suppress_hints_for_tab_reorder {
                    paint_drop_hints(
                        theme,
                        dock_drag_pos.and_then(|position| {
                            fn clamp_point_inside_rect(rect: Rect, point: Point) -> Point {
                                const EPS: f32 = 0.001;
                                let x0 = rect.origin.x.0;
                                let y0 = rect.origin.y.0;
                                let x1 = x0 + rect.size.width.0;
                                let y1 = y0 + rect.size.height.0;

                                let max_x = if x1 > x0 { (x1 - EPS).max(x0) } else { x0 };
                                let max_y = if y1 > y0 { (y1 - EPS).max(y0) } else { y0 };

                                Point::new(
                                    Px(point.x.0.clamp(x0, max_x)),
                                    Px(point.y.0.clamp(y0, max_y)),
                                )
                            }

                            fn distance2_point_to_rect(point: Point, rect: Rect) -> f32 {
                                let x0 = rect.origin.x.0;
                                let y0 = rect.origin.y.0;
                                let x1 = x0 + rect.size.width.0;
                                let y1 = y0 + rect.size.height.0;

                                let dx = if point.x.0 < x0 {
                                    x0 - point.x.0
                                } else if point.x.0 > x1 {
                                    point.x.0 - x1
                                } else {
                                    0.0
                                };
                                let dy = if point.y.0 < y0 {
                                    y0 - point.y.0
                                } else if point.y.0 > y1 {
                                    point.y.0 - y1
                                } else {
                                    0.0
                                };
                                dx * dx + dy * dy
                            }

                            if float_zone(dock_bounds).contains(position)
                                || !bounds.contains(position)
                            {
                                return None;
                            }

                            let mut layout_root = root;
                            let mut layout_bounds = dock_bounds;
                            let mut layout_ctx = &root_layout;
                            let mut effective_position = position;
                            for (floating, chrome, layout) in floating_layouts.iter().rev() {
                                if chrome.close_button.contains(position) {
                                    continue;
                                }
                                if chrome.outer.contains(position) {
                                    layout_root = floating.floating;
                                    layout_bounds = chrome.inner;
                                    layout_ctx = layout;
                                    effective_position = if chrome.title_bar.contains(position) {
                                        let projected = Point::new(
                                            Px(chrome.inner.origin.x.0
                                                + chrome.inner.size.width.0 * 0.5),
                                            Px(chrome.inner.origin.y.0
                                                + chrome.inner.size.height.0 * 0.5),
                                        );
                                        clamp_point_inside_rect(chrome.inner, projected)
                                    } else {
                                        clamp_point_inside_rect(chrome.inner, position)
                                    };
                                    break;
                                }
                            }
                            if !layout_bounds.contains(effective_position) {
                                return None;
                            }

                            let mut best: Option<(DockNodeId, f32, f32)> = None;
                            for (&node_id, &rect) in layout_ctx.iter() {
                                let Some(DockNode::Tabs { tabs, .. }) = dock.graph.node(node_id)
                                else {
                                    continue;
                                };
                                if tabs.is_empty() {
                                    continue;
                                }
                                let dist2 = distance2_point_to_rect(effective_position, rect);
                                let area = rect.size.width.0 * rect.size.height.0;
                                match best {
                                    None => best = Some((node_id, dist2, area)),
                                    Some((_best_node, best_dist2, best_area)) => {
                                        let better = dist2 < best_dist2
                                            || (dist2 == best_dist2 && area < best_area);
                                        if better {
                                            best = Some((node_id, dist2, area));
                                        }
                                    }
                                }
                            }

                            best.map(|(leaf_tabs, _dist2, _area)| DockDropHints {
                                root: layout_root,
                                leaf_tabs,
                            })
                        }),
                        hover.clone(),
                        hint_font_size_inner,
                        hint_font_size_outer,
                        self.window,
                        bounds,
                        &layout_all,
                        scene,
                    );
                }
            }
            paint_drop_overlay(
                theme,
                hover,
                self.window,
                bounds,
                &dock.graph,
                &layout_all,
                &self.tab_scroll,
                &self.tab_widths,
                drag_source_tabs_for_preview,
                drag_tab_title,
                close_glyph_present,
                scene,
            );

            Some(paint_panels)
        });

        let Some(paint_panels) = paint_panels else {
            self.paint_empty_state(cx);
            return;
        };

        let panel_nodes = self.panel_nodes(cx.app);
        for (panel, rect) in paint_panels {
            let Some(node) = panel_nodes.get(&panel) else {
                let is_viewport_panel = cx
                    .app
                    .global::<DockManager>()
                    .and_then(|dock| dock.panel(&panel))
                    .and_then(|p| p.viewport)
                    .is_some();
                if !is_viewport_panel {
                    tracing::warn!(
                        window = ?self.window,
                        panel_kind = %panel.kind.0,
                        panel_instance = ?panel.instance,
                        "docking panel is active but has no UI node; expected driver to bind DockPanelContentService (or use DockPanelRegistryService)"
                    );
                }
                continue;
            };
            let bounds = cx.child_bounds(*node).unwrap_or(rect);
            cx.paint(*node, bounds);
        }
    }
}
