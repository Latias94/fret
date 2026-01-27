// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::hit_test::{hit_test_split_handle, hit_test_tab, tab_scroll_for_node};
use super::layout::{
    active_panel_content_bounds, compute_layout_map, dock_hint_rects, dock_space_regions,
    drop_zone_rect, float_zone, hidden_bounds, split_tab_bar,
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
use super::viewport::{
    ViewportCaptureState, hit_test_active_viewport_panel, viewport_input_from_hit,
    viewport_input_from_hit_clamped,
};
use crate::invalidation::DockInvalidationService;
use fret_ui::retained_bridge::resizable_panel_group as resizable;
use slotmap::Key as _;

const DOCK_FLOATING_BORDER: Px = Px(1.0);
const DOCK_FLOATING_TITLE_H: Px = Px(22.0);
const DOCK_FLOATING_CLOSE_SIZE: Px = Px(14.0);

pub struct DockSpace {
    pub window: fret_core::AppWindowId,
    semantics_test_id: Option<&'static str>,
    last_bounds: Rect,
    prepaint_wants_animation_frames: bool,
    divider_drag: Option<DividerDragSession>,
    floating_drag: Option<FloatingDragState>,
    pending_dock_drags: HashMap<fret_core::PointerId, PendingDockDrag>,
    hovered_floating_close: Option<DockNodeId>,
    pressed_floating_close: Option<DockNodeId>,
    panel_content: HashMap<PanelKey, NodeId>,
    panel_last_sizes: HashMap<PanelKey, Size>,
    viewport_capture: HashMap<fret_core::PointerId, ViewportCaptureState>,
    tab_titles: HashMap<PanelKey, PreparedTabTitle>,
    empty_state: Option<PreparedTabTitle>,
    hovered_tab: Option<(DockNodeId, usize)>,
    hovered_tab_close: bool,
    pressed_tab_close: Option<(DockNodeId, usize, PanelKey)>,
    tab_scroll: HashMap<DockNodeId, Px>,
    tab_widths: HashMap<DockNodeId, Arc<[Px]>>,
    tab_close_glyph: Option<PreparedTabTitle>,
    tab_text_style: TextStyle,
    tab_close_style: TextStyle,
    empty_state_style: TextStyle,
    last_empty_state_scale_factor: Option<f32>,
    last_empty_state_theme_revision: Option<u64>,
    last_tab_text_scale_factor: Option<f32>,
    last_theme_revision: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
struct FloatingDragState {
    floating: DockNodeId,
    grab_offset: Point,
    start_rect: Rect,
}

#[derive(Debug, Clone)]
struct PendingDockDrag {
    start: Point,
    panel: PanelKey,
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
            divider_drag: None,
            floating_drag: None,
            pending_dock_drags: HashMap::new(),
            hovered_floating_close: None,
            pressed_floating_close: None,
            panel_content: HashMap::new(),
            panel_last_sizes: HashMap::new(),
            viewport_capture: HashMap::new(),
            tab_titles: HashMap::new(),
            empty_state: None,
            hovered_tab: None,
            hovered_tab_close: false,
            pressed_tab_close: None,
            tab_scroll: HashMap::new(),
            tab_widths: HashMap::new(),
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
        tabs: DockNodeId,
        tab_bar: Rect,
        tab_count: usize,
    ) -> TabBarGeometry {
        self.tab_widths
            .get(&tabs)
            .filter(|w| w.len() == tab_count)
            .map(|w| TabBarGeometry::variable(tab_bar, w.clone()))
            .unwrap_or_else(|| TabBarGeometry::fixed(tab_bar, tab_count))
    }

    fn max_tab_scroll(&self, tabs: DockNodeId, tab_bar: Rect, tab_count: usize) -> Px {
        self.tab_bar_geometry_for_node(tabs, tab_bar, tab_count)
            .max_scroll()
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

        let geom = self.tab_bar_geometry_for_node(tabs, tab_bar, tab_count);
        let max_scroll = geom.max_scroll();
        let mut scroll = self.tab_scroll_for(tabs);

        if max_scroll.0 <= 0.0 {
            self.tab_scroll.remove(&tabs);
            return;
        }

        scroll = geom.ensure_tab_visible(scroll, active.min(tab_count.saturating_sub(1)));
        self.set_tab_scroll_for(tabs, scroll);
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

        let is_dock_dragging = cx
            .app
            .drag(fret_core::PointerId(0))
            .is_some_and(|d| d.dragging && d.payload::<DockPanelDragPayload>().is_some());
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
        struct DockDragSnapshot {
            source_window: fret_core::AppWindowId,
            start: Point,
            dragging: bool,
            panel: PanelKey,
            grab_offset: Point,
            start_tick: fret_runtime::TickId,
            tear_off_requested: bool,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum DockDropCandidateKind {
            TabBar,
            Hint(DropZone),
            Edge(DropZone),
            FallbackCenter,
        }

        fn candidate_id(node: DockNodeId, kind: DockDropCandidateKind) -> fret_dnd::DndItemId {
            let node_id = node.data().as_ffi();
            let kind_id = match kind {
                DockDropCandidateKind::TabBar => 1,
                DockDropCandidateKind::Hint(DropZone::Center) => 2,
                DockDropCandidateKind::Hint(DropZone::Left) => 3,
                DockDropCandidateKind::Hint(DropZone::Right) => 4,
                DockDropCandidateKind::Hint(DropZone::Top) => 5,
                DockDropCandidateKind::Hint(DropZone::Bottom) => 6,
                DockDropCandidateKind::Edge(DropZone::Left) => 10,
                DockDropCandidateKind::Edge(DropZone::Right) => 11,
                DockDropCandidateKind::Edge(DropZone::Top) => 12,
                DockDropCandidateKind::Edge(DropZone::Bottom) => 13,
                DockDropCandidateKind::Edge(DropZone::Center) => 14,
                DockDropCandidateKind::FallbackCenter => 255,
            };

            // A small, stable mixing function to avoid relying on std hasher stability.
            let mut x = node_id ^ (kind_id as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
            x ^= x >> 32;
            fret_dnd::DndItemId(x)
        }

        fn dock_drop_target_via_dnd(
            graph: &DockGraph,
            layout: &HashMap<DockNodeId, Rect>,
            tab_scroll: &HashMap<DockNodeId, Px>,
            tab_widths: &HashMap<DockNodeId, Arc<[Px]>>,
            position: Point,
        ) -> Option<HoverTarget> {
            #[derive(Debug, Clone, Copy)]
            enum HoverKind {
                TabBar {
                    tabs: DockNodeId,
                    tab_bar: Rect,
                    scroll: Px,
                    tab_count: usize,
                },
                Zone {
                    tabs: DockNodeId,
                    zone: DropZone,
                },
            }

            let mut nodes: Vec<(DockNodeId, Rect)> = layout.iter().map(|(&n, &r)| (n, r)).collect();
            nodes.sort_by_key(|(node, _)| node.data().as_ffi());

            let mut idx = fret_dnd::RectDroppableIndex::<HoverKind>::default();

            for (node, rect) in nodes {
                let Some(DockNode::Tabs { tabs, .. }) = graph.node(node) else {
                    continue;
                };
                if tabs.is_empty() {
                    continue;
                }

                let (tab_bar, _content) = split_tab_bar(rect);
                let scroll = tab_scroll_for_node(tab_scroll, node);

                let tab_id = candidate_id(node, DockDropCandidateKind::TabBar);
                idx.push_rect(
                    tab_id,
                    tab_bar,
                    30,
                    false,
                    HoverKind::TabBar {
                        tabs: node,
                        tab_bar,
                        scroll,
                        tab_count: tabs.len(),
                    },
                );

                for (zone, hint_rect) in dock_hint_rects(rect) {
                    let hint_id = candidate_id(node, DockDropCandidateKind::Hint(zone));
                    idx.push_rect(
                        hint_id,
                        hint_rect,
                        20,
                        false,
                        HoverKind::Zone { tabs: node, zone },
                    );
                }

                for zone in [
                    DropZone::Left,
                    DropZone::Right,
                    DropZone::Top,
                    DropZone::Bottom,
                ] {
                    let edge_rect = drop_zone_rect(rect, zone);
                    let edge_id = candidate_id(node, DockDropCandidateKind::Edge(zone));
                    idx.push_rect(
                        edge_id,
                        edge_rect,
                        10,
                        false,
                        HoverKind::Zone { tabs: node, zone },
                    );
                }

                let fallback_id = candidate_id(node, DockDropCandidateKind::FallbackCenter);
                idx.push_rect(
                    fallback_id,
                    rect,
                    0,
                    false,
                    HoverKind::Zone {
                        tabs: node,
                        zone: DropZone::Center,
                    },
                );
            }

            let (_id, kind) = idx.pick_pointer_within(position)?;
            match kind {
                HoverKind::TabBar {
                    tabs,
                    tab_bar,
                    scroll,
                    tab_count,
                } => Some(HoverTarget {
                    tabs,
                    zone: DropZone::Center,
                    insert_index: Some({
                        let geom = tab_widths
                            .get(&tabs)
                            .filter(|w| w.len() == tab_count)
                            .map(|w| TabBarGeometry::variable(tab_bar, w.clone()))
                            .unwrap_or_else(|| TabBarGeometry::fixed(tab_bar, tab_count));
                        geom.compute_insert_index(position, scroll)
                    }),
                }),
                HoverKind::Zone { tabs, zone } => Some(HoverTarget {
                    tabs,
                    zone,
                    insert_index: None,
                }),
            }
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
            position: Point,
        ) -> Option<DockDropTarget> {
            if !window_bounds.contains(position) || float_zone(dock_bounds).contains(position) {
                return Some(DockDropTarget::Float { window });
            }

            let (layout_root, layout_bounds) =
                layout_context_for_position(graph, window, root, dock_bounds, position);
            if !layout_bounds.contains(position) {
                return None;
            }

            let layout = compute_layout_map(graph, layout_root, layout_bounds);
            dock_drop_target_via_dnd(graph, &layout, tab_scroll, tab_widths, position)
                .map(DockDropTarget::Dock)
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
            position: Point,
        ) -> Option<DockDropTarget> {
            if invert_docking {
                return Some(DockDropTarget::Float { window });
            }

            prev_hover.or_else(|| {
                compute_dock_drop_target(
                    graph,
                    window,
                    root,
                    dock_bounds,
                    window_bounds,
                    tab_scroll,
                    tab_widths,
                    position,
                )
            })
        }

        fn resolve_dock_drop_intent<F>(
            target: Option<DockDropTarget>,
            drag: &DockDragSnapshot,
            target_window: fret_core::AppWindowId,
            dock_bounds: Rect,
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
                    let wants_tear_off = allow_tear_off
                        && (!window_bounds.contains(position)
                            || float_zone(dock_bounds).contains(position));
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
            d.kind == DRAG_KIND_DOCK_PANEL
                && (d.source_window == self.window || d.current_window == self.window)
        });
        let dock_drag = cx.app.drag(pointer_id).and_then(|d| {
            d.payload::<DockPanelDragPayload>()
                .map(|p| DockDragSnapshot {
                    source_window: d.source_window,
                    start: d.start_position,
                    dragging: d.dragging,
                    panel: p.panel.clone(),
                    grab_offset: p.grab_offset,
                    start_tick: p.start_tick,
                    tear_off_requested: p.tear_off_requested,
                })
        });
        let has_pending_dock_drag = self.pending_dock_drags.contains_key(&pointer_id);
        // While a dock drag session exists (even before it crosses the drag threshold), we must
        // not forward pointer moves/wheel to embedded viewports in this window. Docking owns the
        // interaction until the session ends (ADR 0072).
        let allow_viewport_hover = !dock_drag_affects_window
            && dock_drag.is_none()
            && !has_pending_dock_drag
            && cx.app.drag(pointer_id).is_none_or(|d| !d.dragging);
        let docking_interaction_settings = cx
            .app
            .global::<fret_runtime::DockingInteractionSettings>()
            .copied()
            .unwrap_or_default();
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

        let mut start_dock_drag: Option<(Point, DockPanelDragPayload, Point)> = None;
        let mut update_drag: Option<(Point, bool)> = None;
        let mut end_dock_drag = false;
        let mut mark_drag_tear_off_requested = false;

        fret_ui::internal_drag::set_route(
            cx.app,
            self.window,
            fret_runtime::DRAG_KIND_DOCK_PANEL,
            cx.node,
        );
        let dock_space_node = cx.node;
        let allow_tear_off =
            cx.input_ctx.caps.ui.window_tear_off && cx.input_ctx.caps.ui.multi_window;
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
                                            self.floating_drag = Some(FloatingDragState {
                                                floating,
                                                grab_offset: Point::new(
                                                    Px(position.x.0 - entry.rect.origin.x.0),
                                                    Px(position.y.0 - entry.rect.origin.y.0),
                                                ),
                                                start_rect: entry.rect,
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
                            let layout =
                                compute_layout_map(&dock.graph, layout_root, layout_bounds);
                            if *button == fret_core::MouseButton::Left {
                                if !handled
                                    && let Some(handle) =
                                        hit_test_split_handle(&dock.graph, &layout, *position)
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
                                            .and_then(|rect| {
                                                let (bar, _content) = split_tab_bar(rect);
                                                let tab_count = match dock.graph.node(tabs_node) {
                                                    Some(DockNode::Tabs { tabs, .. }) => tabs.len(),
                                                    _ => 0,
                                                };
                                                (tab_index < tab_count).then(|| {
                                                    let scroll = self.tab_scroll_for(tabs_node);
                                                    self.tab_bar_geometry_for_node(
                                                        tabs_node, bar, tab_count,
                                                    )
                                                    .tab_rect(tab_index, scroll)
                                                })
                                            })
                                            .unwrap_or_else(|| {
                                                Rect::new(*position, Size::default())
                                            });
                                        let tab_local = Point::new(
                                            Px((position.x.0 - tab_rect.origin.x.0).max(0.0)),
                                            Px((position.y.0 - tab_rect.origin.y.0).max(0.0)),
                                        );
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
                            if let Some(drag) = self.floating_drag {
                                let desired = Rect::new(
                                    Point::new(
                                        Px(position.x.0 - drag.grab_offset.x.0),
                                        Px(position.y.0 - drag.grab_offset.y.0),
                                    ),
                                    drag.start_rect.size,
                                );
                                let rect = Self::clamp_rect_to_bounds(desired, window_bounds);
                                pending_effects.push(Effect::Dock(DockOp::SetFloatingRect {
                                    window: self.window,
                                    floating: drag.floating,
                                    rect,
                                }));
                                request_cursor = Some(fret_core::CursorIcon::Default);
                                invalidate_layout = true;
                                invalidate_paint = true;
                                pending_redraws.push(self.window);
                                stop_propagation = true;
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
                                        start_dock_drag = Some((
                                            pending.start,
                                            DockPanelDragPayload {
                                                panel: pending.panel,
                                                grab_offset: pending.grab_offset,
                                                start_tick: pending.start_tick,
                                                tear_off_requested: false,
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
                                    let layout =
                                        compute_layout_map(&dock.graph, layout_root, layout_bounds);
                                    if let Some(handle) =
                                        hit_test_split_handle(&dock.graph, &layout, *position)
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

                                let hovered = if self.viewport_capture.is_empty()
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
                                    let layout =
                                        compute_layout_map(&dock.graph, layout_root, layout_bounds);
                                    hit_test_tab(
                                        &dock.graph,
                                        &layout,
                                        &self.tab_scroll,
                                        &self.tab_widths,
                                        theme,
                                        *position,
                                    )
                                    .map(|(node, idx, _panel, close)| (node, idx, close))
                                } else {
                                    None
                                };
                                let next_tab = hovered.map(|(node, idx, _close)| (node, idx));
                                let next_close =
                                    hovered.map(|(_node, _idx, close)| close).unwrap_or(false);
                                if next_tab != self.hovered_tab
                                    || next_close != self.hovered_tab_close
                                {
                                    self.hovered_tab = next_tab;
                                    self.hovered_tab_close = next_close;
                                    invalidate_paint = true;
                                    pending_redraws.push(self.window);
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
                                        DOCK_SPLIT_HANDLE_GAP,
                                        DOCK_SPLIT_HANDLE_HIT_THICKNESS,
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
                            if dock_drag.is_some() || has_pending_dock_drag {
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
                            let layout =
                                compute_layout_map(&dock.graph, layout_root, layout_bounds);
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
                                    node_id,
                                    tab_bar,
                                    tabs.len(),
                                    *active,
                                );

                                let max_scroll = self.max_tab_scroll(node_id, tab_bar, tabs.len());
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
                                let mut layout = compute_layout_map(&dock.graph, root, dock_bounds);
                                if !layout.contains_key(&tabs_node) {
                                    for floating in dock.graph.floating_windows(self.window) {
                                        let chrome = Self::floating_chrome(floating.rect);
                                        let l = compute_layout_map(
                                            &dock.graph,
                                            floating.floating,
                                            chrome.inner,
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
                                    if drag.dragging {
                                        let target = dock.hover.clone().or_else(|| {
                                            (!window_bounds.contains(*position)
                                                || float_zone(dock_bounds).contains(*position))
                                            .then_some(DockDropTarget::Float {
                                                window: self.window,
                                            })
                                        });

                                        let intent = resolve_dock_drop_intent(
                                            target,
                                            drag,
                                            self.window,
                                            dock_bounds,
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
                                        );

                                        if let DockDropIntent::RequestFloatPanelToNewWindow {
                                            ..
                                        } = intent
                                        {
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
                                        let layout =
                                            compute_layout_map(&dock.graph, root, dock_bounds);
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
                        let wants_dock_previews = docking_interaction_settings
                            .drag_inversion
                            .wants_dock_previews(e.modifiers);
                        let invert_docking = !wants_dock_previews;
                        match e.kind {
                            fret_core::InternalDragKind::Enter
                            | fret_core::InternalDragKind::Over => {
                                if let Some(drag) = dock_drag.as_ref() {
                                    let prev_hover = dock.hover.clone();
                                    let mut dragging = drag.dragging;
                                    if drag.source_window == self.window {
                                        // Match ImGui's default drag threshold (~6 logical px).
                                        let activation = fret_dnd::ActivationConstraint::Distance {
                                            px: docking_interaction_settings.tab_drag_threshold.0,
                                        };
                                        if !dragging
                                            && activation.is_satisfied(
                                                drag.start_tick.0,
                                                now_tick.0,
                                                drag.start,
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
                                        let margin = Px(10.0);
                                        let requested_tear_off = allow_tear_off
                                            && drag.source_window == self.window
                                            && is_outside_bounds_with_margin(
                                                window_bounds,
                                                position,
                                                margin,
                                            )
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

                                        if !requested_tear_off {
                                            dock.hover = resolve_dock_drop_target(
                                                None,
                                                invert_docking,
                                                self.window,
                                                &dock.graph,
                                                root,
                                                dock_bounds,
                                                window_bounds,
                                                &self.tab_scroll,
                                                &self.tab_widths,
                                                position,
                                            );
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
                                        let target = resolve_dock_drop_target(
                                            prev_hover.clone(),
                                            invert_docking,
                                            self.window,
                                            &dock.graph,
                                            root,
                                            dock_bounds,
                                            window_bounds,
                                            &self.tab_scroll,
                                            &self.tab_widths,
                                            position,
                                        );
                                        let intent = resolve_dock_drop_intent(
                                            target,
                                            drag,
                                            self.window,
                                            dock_bounds,
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
                                        );

                                        if let DockDropIntent::RequestFloatPanelToNewWindow {
                                            ..
                                        } = intent
                                        {
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
                        if self.pending_dock_drags.remove(&e.pointer_id).is_some() {
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

        if let Some(request) = request_pointer_capture {
            match request {
                Some(node) => cx.capture_pointer(node),
                None => cx.release_pointer_capture(),
            }
        }

        if let Some((position, dragging)) = update_drag
            && let Some(drag) = cx.app.drag_mut(pointer_id)
            && drag.payload::<DockPanelDragPayload>().is_some()
        {
            drag.position = position;
            drag.dragging = dragging;
            if mark_drag_tear_off_requested {
                if let Some(payload) = drag.payload_mut::<DockPanelDragPayload>() {
                    payload.tear_off_requested = true;
                }
            }
        }

        if end_dock_drag
            && cx
                .app
                .drag(pointer_id)
                .and_then(|d| d.payload::<DockPanelDragPayload>())
                .is_some()
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
        let hidden = hidden_bounds(Size::new(Px(0.0), Px(0.0)));

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
            let mut layout = compute_layout_map(&dock.graph, root, dock_bounds);

            for floating in dock.graph.floating_windows(self.window) {
                let chrome = Self::floating_chrome(floating.rect);
                let floating_layout =
                    compute_layout_map(&dock.graph, floating.floating, chrome.inner);
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
                self.clamp_and_ensure_active_visible(node_id, tab_bar, tabs.len(), *active);
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

            cx.app.with_global_mut_untracked(
                fret_runtime::WindowInteractionDiagnosticsStore::default,
                |svc, _app| {
                    svc.record_docking(
                        self.window,
                        frame_id,
                        fret_runtime::DockingInteractionDiagnostics {
                            dock_drag,
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

        let paint_panels = app.with_global_mut_untracked(DockManager::default, |dock, _app| {
            dock.register_dock_space_node(self.window, dock_space_node);
            let Some(root) = dock.graph.window_root(self.window) else {
                return None;
            };

            let root_layout = compute_layout_map(&dock.graph, root, dock_bounds);

            let mut floating_layouts: Vec<(
                fret_core::DockFloatingWindow,
                FloatingChrome,
                HashMap<DockNodeId, Rect>,
            )> = Vec::new();
            let mut layout_all = root_layout.clone();
            for floating in dock.graph.floating_windows(self.window) {
                let chrome = Self::floating_chrome(floating.rect);
                let layout = compute_layout_map(&dock.graph, floating.floating, chrome.inner);
                for (k, v) in layout.iter() {
                    layout_all.insert(*k, *v);
                }
                floating_layouts.push((*floating, chrome, layout));
            }

            let hover = dock.hover.clone();

            self.rebuild_tab_titles(services, theme, scale_factor, &*dock, &layout_all);
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
                    pressed_tab_close: self.pressed_tab_close.as_ref().map(|(n, i, _)| (*n, *i)),
                    tab_scroll: &self.tab_scroll,
                    tab_close_glyph: self.tab_close_glyph,
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
                    background: surface,
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
                        pressed_tab_close: self
                            .pressed_tab_close
                            .as_ref()
                            .map(|(n, i, _)| (*n, *i)),
                        tab_scroll: &self.tab_scroll,
                        tab_close_glyph: self.tab_close_glyph,
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
                scale_factor,
                scene,
            );

            if is_dock_dragging {
                paint_drop_hints(
                    theme,
                    hover.clone(),
                    self.window,
                    bounds,
                    &layout_all,
                    scene,
                );
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
