use anyhow::Context as _;
use fret_app::CreateWindowKind;
use fret_app::{App, CommandId, Effect, Model, WindowRequest};
use fret_bootstrap::ui_diagnostics::UiDiagnosticsService;
use fret_core::{
    AppWindowId, Color, Corners, DrawOrder, Edges, Event, Modifiers, MouseButton, MouseButtons,
    Point, Rect, RenderTargetId, Scene, SceneOp, Size, UiServices, ViewportInputEvent,
    dock::DropZone, geometry::Px,
};
use fret_docking::{
    DockManager, DockPanel, DockPanelRegistry, DockPanelRegistryService, DockSpace,
    DockViewportOverlayHooks, DockViewportOverlayHooksService, DockingPolicy, DockingPolicyService,
    DockingRuntime, render_and_bind_dock_panels, render_cached_panel_root,
};
use fret_launch::{
    DevStateExport, DevStateHook, DevStateHooks, DevStateWindowKeyRegistry, WindowCreateSpec,
    WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
    WinitWindowContext,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::declarative;
use fret_ui::element::{ContainerProps, LayoutStyle, Length};
use fret_ui::retained_bridge::resizable_panel_group as resizable;
use fret_ui::retained_bridge::{LayoutCx, PaintCx, SemanticsCx, UiTreeRetainedExt as _, Widget};
use fret_ui::{Invalidation, Theme, UiTree};
use fret_ui_kit::OverlayController;
use fret_ui_kit::declarative::stack::{VStackProps, vstack};
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn as shadcn;
use serde_json::json;
use slotmap::KeyData;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

type ViewportKey = (AppWindowId, RenderTargetId);

// Keep these in sync with `fret-docking` floating chrome constants. This harness duplicates
// the geometry in order to keep diagnostics anchors stable without reaching into crate-private
// helpers.
const DOCKING_ARBITRATION_FLOATING_BORDER_PX: f32 = 1.0;
const DOCKING_ARBITRATION_FLOATING_TITLE_H_PX: f32 = 22.0;
const DOCKING_ARBITRATION_TAB_BAR_H: Px = Px(28.0);
const DOCKING_ARBITRATION_DRAG_ANCHOR_SIZE: Px = Px(12.0);
const DOCKING_ARBITRATION_SPLIT_HANDLE_ANCHOR_SIZE: Px = Px(12.0);

const DEV_STATE_DOCKING_LAYOUT_KEY: &str = "docking_arbitration.layout";

#[derive(Debug, Default)]
struct DockingArbitrationDevStateIncoming {
    layout: Option<fret_core::DockLayout>,
}

#[derive(Debug, Default)]
struct DockingArbitrationDevStateModels {
    persist_layout_on_exit: bool,
    windows: Vec<(AppWindowId, String)>,
}

#[derive(Debug, Default)]
struct DockingArbitrationDevStateGate {
    export_ready: bool,
}

struct DockingArbitrationDragAnchor {
    test_id: Arc<str>,
}

impl DockingArbitrationDragAnchor {
    fn new(test_id: impl Into<Arc<str>>) -> Self {
        Self {
            test_id: test_id.into(),
        }
    }
}

impl<H: fret_ui::UiHost> Widget<H> for DockingArbitrationDragAnchor {
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        false
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(fret_core::SemanticsRole::Group);
        cx.set_test_id(self.test_id.as_ref());
    }
}

#[derive(Clone)]
struct DockingArbitrationPolicyFlags {
    disallow_left_edge: Arc<AtomicBool>,
    disallow_drop_targets: Arc<AtomicBool>,
}

impl DockingArbitrationPolicyFlags {
    fn new() -> Self {
        let disallow_drop_targets = std::env::var("FRET_DOCK_ARB_DISALLOW_DROP_TARGETS")
            .ok()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
        Self {
            disallow_left_edge: Arc::new(AtomicBool::new(false)),
            disallow_drop_targets: Arc::new(AtomicBool::new(disallow_drop_targets)),
        }
    }
}

struct DockingArbitrationDockingPolicy {
    flags: DockingArbitrationPolicyFlags,
}

impl DockingPolicy for DockingArbitrationDockingPolicy {
    fn allow_dock_drop_target(
        &self,
        _window: AppWindowId,
        _layout_root: fret_core::DockNodeId,
        _tabs: fret_core::DockNodeId,
        zone: DropZone,
        _outer: bool,
    ) -> bool {
        if self.flags.disallow_drop_targets.load(Ordering::Relaxed) {
            return false;
        }
        if zone == DropZone::Left && self.flags.disallow_left_edge.load(Ordering::Relaxed) {
            return false;
        }
        true
    }
}

struct DockingArbitrationHarnessRoot {
    window: AppWindowId,
    dock_space: fret_core::NodeId,
    left_anchor: fret_core::NodeId,
    right_anchor: fret_core::NodeId,
    right_tabs_group_anchor: fret_core::NodeId,
    extra_anchors: Vec<fret_core::NodeId>,
    float_zone_anchor: fret_core::NodeId,
    viewport_split_handle_anchor: fret_core::NodeId,
    floating_title_bar_anchor: fret_core::NodeId,
    tab_drop_end_anchor: fret_core::NodeId,
    tab_overflow_button_anchor: fret_core::NodeId,
    tab_overflow_menu_row_1_anchor: fret_core::NodeId,
    tab_scroll_edge_left_anchor: fret_core::NodeId,
    tab_scroll_edge_right_anchor: fret_core::NodeId,
    dock_hint_inner_anchors: Vec<(DropZone, fret_core::NodeId)>,
    dock_hint_outer_anchors: Vec<(DropZone, fret_core::NodeId)>,
}

impl<H: fret_ui::UiHost> Widget<H> for DockingArbitrationHarnessRoot {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let bounds = cx.bounds;
        let _ = cx.layout_in(self.dock_space, bounds);

        let docking_interaction_settings = cx
            .app
            .global::<fret_runtime::DockingInteractionSettings>()
            .copied()
            .unwrap_or_default();
        let split_handle_gap = docking_interaction_settings.split_handle_gap;
        let split_handle_hit_thickness = docking_interaction_settings.split_handle_hit_thickness;

        let half = DOCKING_ARBITRATION_DRAG_ANCHOR_SIZE.0 * 0.5;
        let rect = |x: f32, y: f32| {
            Rect::new(
                Point::new(Px((x - half).max(bounds.origin.x.0)), Px(y - half)),
                Size::new(
                    DOCKING_ARBITRATION_DRAG_ANCHOR_SIZE,
                    DOCKING_ARBITRATION_DRAG_ANCHOR_SIZE,
                ),
            )
        };

        // Scripted drag anchors must start on the *tab* itself (not empty space in the tab bar),
        // otherwise docking will interpret the interaction as a "tabs group" drag.
        //
        // For multi-window tear-off scenarios, this distinction matters:
        // - panel drags tear off a single tab (ImGui-style),
        // - tabs-group drags tear off the whole stack.
        //
        // Avoid depending on the viewport layout cache produced during `DockSpace::paint(...)`.
        // That cache is populated after layout, which makes script anchors race-y on the first
        // few frames (especially around window resize / multi-window tear-off).
        let fallback_y = bounds.origin.y.0 + (DOCKING_ARBITRATION_TAB_BAR_H.0 * 0.5);
        let fallback_mid_x = bounds.origin.x.0 + bounds.size.width.0 * 0.5;
        let fallback_pad_x = 48.0_f32.min((bounds.size.width.0 * 0.25).max(0.0));
        let mut left_anchor_pos = (bounds.origin.x.0 + fallback_pad_x, fallback_y);
        let mut right_anchor_pos = (fallback_mid_x + fallback_pad_x, fallback_y);
        let mut right_tabs_group_anchor_pos = (right_anchor_pos.0 + 96.0, fallback_y);
        let mut extra_anchor_pos: Vec<(f32, f32)> = (0..self.extra_anchors.len())
            .map(|ix| (bounds.origin.x.0 + 24.0 + (ix as f32 * 4.0), fallback_y))
            .collect();

        if let Some(dock) = cx.app.global::<DockManager>() {
            use fret_core::{DockGraph, DockNode, DockNodeId, PanelKey};

            fn tabs_rect_for_panel(
                graph: &DockGraph,
                node: DockNodeId,
                rect: Rect,
                split_handle_gap: Px,
                split_handle_hit_thickness: Px,
                panel: &PanelKey,
            ) -> Option<Rect> {
                #[allow(unreachable_patterns)]
                match graph.node(node)? {
                    DockNode::Tabs { tabs, .. } => tabs.iter().any(|p| p == panel).then_some(rect),
                    DockNode::Floating { child } => tabs_rect_for_panel(
                        graph,
                        *child,
                        rect,
                        split_handle_gap,
                        split_handle_hit_thickness,
                        panel,
                    ),
                    DockNode::Split {
                        axis,
                        children,
                        fractions,
                    } => {
                        let min_px = vec![Px(0.0); children.len()];
                        let computed = resizable::compute_layout(
                            *axis,
                            rect,
                            children.len(),
                            fractions,
                            split_handle_gap,
                            split_handle_hit_thickness,
                            &min_px,
                        );
                        for (child, &child_rect) in children.iter().zip(computed.panel_rects.iter())
                        {
                            if let Some(found) = tabs_rect_for_panel(
                                graph,
                                *child,
                                child_rect,
                                split_handle_gap,
                                split_handle_hit_thickness,
                                panel,
                            ) {
                                return Some(found);
                            }
                        }
                        None
                    }
                }
            }

            fn tab_bar_anchor_for_panel(
                dock: &DockManager,
                window: AppWindowId,
                bounds: Rect,
                split_handle_gap: Px,
                split_handle_hit_thickness: Px,
                panel: &PanelKey,
            ) -> Option<(f32, f32)> {
                let anchor_for_rect = |tabs_rect: Rect, floating: bool| {
                    let (x0, y0) = if floating {
                        (
                            tabs_rect.origin.x.0 + DOCKING_ARBITRATION_FLOATING_BORDER_PX,
                            tabs_rect.origin.y.0
                                + DOCKING_ARBITRATION_FLOATING_BORDER_PX
                                + DOCKING_ARBITRATION_FLOATING_TITLE_H_PX,
                        )
                    } else {
                        (tabs_rect.origin.x.0, tabs_rect.origin.y.0)
                    };

                    let x = if floating {
                        x0 + (tabs_rect.size.width.0 * 0.2).clamp(48.0, 96.0)
                    } else {
                        x0 + 16.0
                    };
                    let y = y0 + (DOCKING_ARBITRATION_TAB_BAR_H.0 * 0.5);
                    (x, y)
                };

                if let Some(root) = dock.graph.window_root(window) {
                    if let Some(tabs_rect) = tabs_rect_for_panel(
                        &dock.graph,
                        root,
                        bounds,
                        split_handle_gap,
                        split_handle_hit_thickness,
                        panel,
                    ) {
                        return Some(anchor_for_rect(tabs_rect, false));
                    }
                }

                for floating in dock.graph.floating_windows(window) {
                    if let Some(tabs_rect) = tabs_rect_for_panel(
                        &dock.graph,
                        floating.floating,
                        floating.rect,
                        split_handle_gap,
                        split_handle_hit_thickness,
                        panel,
                    ) {
                        return Some(anchor_for_rect(tabs_rect, true));
                    }
                }

                None
            }

            fn tab_bar_tabs_group_anchor_for_panel(
                dock: &DockManager,
                window: AppWindowId,
                bounds: Rect,
                split_handle_gap: Px,
                split_handle_hit_thickness: Px,
                panel: &PanelKey,
            ) -> Option<(f32, f32)> {
                let anchor_for_rect = |tabs_rect: Rect, floating: bool| {
                    let (x0, y0) = if floating {
                        (
                            tabs_rect.origin.x.0 + DOCKING_ARBITRATION_FLOATING_BORDER_PX,
                            tabs_rect.origin.y.0
                                + DOCKING_ARBITRATION_FLOATING_BORDER_PX
                                + DOCKING_ARBITRATION_FLOATING_TITLE_H_PX,
                        )
                    } else {
                        (tabs_rect.origin.x.0, tabs_rect.origin.y.0)
                    };

                    let x1 = x0 + tabs_rect.size.width.0;

                    // Intentionally aim for empty tab-bar space so docking interprets this as a
                    // "tabs group" drag (rather than a single tab/panel drag).
                    let x = (x1 - 40.0).max(x0 + 8.0);
                    let y = y0 + (DOCKING_ARBITRATION_TAB_BAR_H.0 * 0.5);
                    (x, y)
                };

                if let Some(root) = dock.graph.window_root(window) {
                    if let Some(tabs_rect) = tabs_rect_for_panel(
                        &dock.graph,
                        root,
                        bounds,
                        split_handle_gap,
                        split_handle_hit_thickness,
                        panel,
                    ) {
                        return Some(anchor_for_rect(tabs_rect, false));
                    }
                }

                for floating in dock.graph.floating_windows(window) {
                    if let Some(tabs_rect) = tabs_rect_for_panel(
                        &dock.graph,
                        floating.floating,
                        floating.rect,
                        split_handle_gap,
                        split_handle_hit_thickness,
                        panel,
                    ) {
                        return Some(anchor_for_rect(tabs_rect, true));
                    }
                }

                None
            }

            let viewport_left = PanelKey::new("demo.viewport.left");
            let viewport_right = PanelKey::new("demo.viewport.right");
            if let Some(p) = tab_bar_anchor_for_panel(
                dock,
                self.window,
                bounds,
                split_handle_gap,
                split_handle_hit_thickness,
                &viewport_left,
            ) {
                left_anchor_pos = p;
            }
            if let Some(p) = tab_bar_anchor_for_panel(
                dock,
                self.window,
                bounds,
                split_handle_gap,
                split_handle_hit_thickness,
                &viewport_right,
            ) {
                right_anchor_pos = p;
            }
            if let Some(p) = tab_bar_tabs_group_anchor_for_panel(
                dock,
                self.window,
                bounds,
                split_handle_gap,
                split_handle_hit_thickness,
                &viewport_right,
            ) {
                right_tabs_group_anchor_pos = p;
            }

            for (ix, pos) in extra_anchor_pos.iter_mut().enumerate() {
                let panel = PanelKey::new(format!("demo.viewport.extra.{ix}"));
                if let Some(p) = tab_bar_anchor_for_panel(
                    dock,
                    self.window,
                    bounds,
                    split_handle_gap,
                    split_handle_hit_thickness,
                    &panel,
                ) {
                    *pos = p;
                }
            }
        }

        let _ = cx.layout_in(self.left_anchor, rect(left_anchor_pos.0, left_anchor_pos.1));
        let _ = cx.layout_in(
            self.right_anchor,
            rect(right_anchor_pos.0, right_anchor_pos.1),
        );
        let _ = cx.layout_in(
            self.right_tabs_group_anchor,
            rect(right_tabs_group_anchor_pos.0, right_tabs_group_anchor_pos.1),
        );
        for (anchor, (x, y)) in self
            .extra_anchors
            .iter()
            .copied()
            .zip(extra_anchor_pos.iter().copied())
        {
            let _ = cx.layout_in(anchor, rect(x, y));
        }

        let float_zone_anchor_rect = {
            // Mirror `fret_docking::dock::layout::float_zone(...)` logic for stable, pixel-free
            // scripted diagnostics.
            //
            // Note: This is intentionally duplicated here (demo-only harness) to avoid relying on
            // crate-private helpers.
            let pad = 2.0_f32;
            let size = (DOCKING_ARBITRATION_TAB_BAR_H.0 - pad * 2.0).max(0.0);
            let x =
                (bounds.origin.x.0 + bounds.size.width.0 - pad - size).max(bounds.origin.x.0 + pad);
            let y = bounds.origin.y.0 + pad;
            let cx = x + size * 0.5;
            let cy = y + size * 0.5;
            Rect::new(
                Point::new(Px(cx - half), Px(cy - half)),
                Size::new(
                    DOCKING_ARBITRATION_DRAG_ANCHOR_SIZE,
                    DOCKING_ARBITRATION_DRAG_ANCHOR_SIZE,
                ),
            )
        };
        let _ = cx.layout_in(self.float_zone_anchor, float_zone_anchor_rect);

        let hidden = Rect::new(
            Point::new(Px(-1_000_000.0), Px(-1_000_000.0)),
            Size::new(
                DOCKING_ARBITRATION_DRAG_ANCHOR_SIZE,
                DOCKING_ARBITRATION_DRAG_ANCHOR_SIZE,
            ),
        );

        let hint_candidates = cx
            .app
            .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
            .and_then(|store| store.docking_latest_for_window(self.window))
            .and_then(|d| d.dock_drop_resolve.as_ref())
            .map(|d| d.candidates.clone())
            .unwrap_or_default();
        let fallback_hint_rect = Rect::new(
            Point::new(
                Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5 - half),
                Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5 - half),
            ),
            Size::new(
                DOCKING_ARBITRATION_DRAG_ANCHOR_SIZE,
                DOCKING_ARBITRATION_DRAG_ANCHOR_SIZE,
            ),
        );

        let (
            overflow_button_anchor_rect,
            overflow_row_1_anchor_rect,
            tab_drop_end_anchor_rect,
            tab_scroll_edge_left_anchor_rect,
            tab_scroll_edge_right_anchor_rect,
        ) = (|| {
            use fret_core::{DockGraph, DockNode, DockNodeId, PanelKey};

            fn tabs_rect_for_panel(
                graph: &DockGraph,
                node: DockNodeId,
                rect: Rect,
                split_handle_gap: Px,
                split_handle_hit_thickness: Px,
                panel: &PanelKey,
            ) -> Option<Rect> {
                match graph.node(node)? {
                    DockNode::Tabs { tabs, .. } => tabs.iter().any(|p| p == panel).then_some(rect),
                    DockNode::Floating { child } => tabs_rect_for_panel(
                        graph,
                        *child,
                        rect,
                        split_handle_gap,
                        split_handle_hit_thickness,
                        panel,
                    ),
                    DockNode::Split {
                        axis,
                        children,
                        fractions,
                    } => {
                        let min_px = vec![Px(0.0); children.len()];
                        let computed = resizable::compute_layout(
                            *axis,
                            rect,
                            children.len(),
                            fractions,
                            split_handle_gap,
                            split_handle_hit_thickness,
                            &min_px,
                        );
                        for (child, &child_rect) in children.iter().zip(computed.panel_rects.iter())
                        {
                            if let Some(found) = tabs_rect_for_panel(
                                graph,
                                *child,
                                child_rect,
                                split_handle_gap,
                                split_handle_hit_thickness,
                                panel,
                            ) {
                                return Some(found);
                            }
                        }
                        None
                    }
                }
            }

            let dock = cx.app.global::<DockManager>()?;
            let root = dock.graph.window_root(self.window)?;

            // Anchor overflow geometry to the left viewport's tabs container. Most arbitration
            // scripts build tab overflow by merging additional tabs into that leaf.
            let left_panel = PanelKey::new("demo.viewport.left");
            let tabs_rect = tabs_rect_for_panel(
                &dock.graph,
                root,
                bounds,
                split_handle_gap,
                split_handle_hit_thickness,
                &left_panel,
            )?;
            let (tabs_node, _active) = dock.graph.find_panel_in_window(self.window, &left_panel)?;
            let tab_count = match dock.graph.node(tabs_node)? {
                DockNode::Tabs { tabs, .. } => tabs.len(),
                _ => 0,
            };
            if tab_count == 0 {
                return None;
            }

            // Duplicate the docking overflow geometry formula in order to keep scripted anchors
            // stable without reaching into crate-private helpers.
            let theme = cx.theme().snapshot();
            let tab_bar = Rect {
                origin: tabs_rect.origin,
                size: Size::new(
                    tabs_rect.size.width,
                    Px(DOCKING_ARBITRATION_TAB_BAR_H.0.min(tabs_rect.size.height.0)),
                ),
            };
            let pad = theme.metric_token("metric.padding.sm").0.max(0.0);
            let button_size = (tab_bar.size.height.0 * 0.80).clamp(18.0, 24.0);
            let button_rect = Rect::new(
                Point::new(
                    Px(tab_bar.origin.x.0 + tab_bar.size.width.0 - pad - button_size),
                    Px(tab_bar.origin.y.0 + (tab_bar.size.height.0 - button_size) * 0.5),
                ),
                Size::new(Px(button_size), Px(button_size)),
            );

            let menu_width = (tab_bar.size.width.0 * 0.55).clamp(180.0, 320.0);
            let rows = (tab_count.clamp(1, 10)) as f32;
            let menu_height =
                (rows * tab_bar.size.height.0).clamp(tab_bar.size.height.0 * 2.0, 320.0);
            let menu_rect = Rect::new(
                Point::new(
                    Px(tab_bar.origin.x.0 + tab_bar.size.width.0 - pad - menu_width),
                    Px(tab_bar.origin.y.0 + tab_bar.size.height.0 + pad),
                ),
                Size::new(Px(menu_width), Px(menu_height)),
            );
            let row_h = Px(tab_bar.size.height.0.max(0.0));
            let row_1_rect = Rect::new(
                Point::new(menu_rect.origin.x, Px(menu_rect.origin.y.0 + row_h.0)),
                Size::new(menu_rect.size.width, row_h),
            );

            let button_cx = button_rect.origin.x.0 + button_rect.size.width.0 * 0.5;
            let button_cy = button_rect.origin.y.0 + button_rect.size.height.0 * 0.5;

            // Prefer clicking towards the left side of the row to avoid the trailing close
            // affordance.
            let row_cx = row_1_rect.origin.x.0 + row_1_rect.size.width.0 * 0.25;
            let row_cy = row_1_rect.origin.y.0 + row_1_rect.size.height.0 * 0.5;

            // Anchor a "drop at end" position to the reserved header space to the left of the
            // overflow button. This avoids fragile `set_cursor_in_window_logical` coordinates in
            // scripts while still gating the same dock drop resolution contract.
            let end_cx = {
                let x_right = tab_bar.origin.x.0 + tab_bar.size.width.0;
                let x_before_button = (button_rect.origin.x.0 - 2.0).max(tab_bar.origin.x.0 + pad);
                (x_right - pad - 2.0).min(x_before_button)
            };
            let end_cy = tab_bar.origin.y.0 + tab_bar.size.height.0 * 0.5;

            let edge_left_cx = tab_bar.origin.x.0 + 1.0_f32.min(tab_bar.size.width.0.max(0.0));
            let edge_right_cx =
                tab_bar.origin.x.0 + (tab_bar.size.width.0.max(0.0) - 1.0_f32).max(0.0);
            let edge_cy = tab_bar.origin.y.0 + tab_bar.size.height.0 * 0.5;

            Some((
                rect(button_cx, button_cy),
                rect(row_cx, row_cy),
                rect(end_cx, end_cy),
                rect(edge_left_cx, edge_cy),
                rect(edge_right_cx, edge_cy),
            ))
        })()
        .unwrap_or((hidden, hidden, hidden, hidden, hidden));

        let _ = cx.layout_in(self.tab_overflow_button_anchor, overflow_button_anchor_rect);
        let _ = cx.layout_in(
            self.tab_overflow_menu_row_1_anchor,
            overflow_row_1_anchor_rect,
        );
        let _ = cx.layout_in(self.tab_drop_end_anchor, tab_drop_end_anchor_rect);
        let _ = cx.layout_in(
            self.tab_scroll_edge_left_anchor,
            tab_scroll_edge_left_anchor_rect,
        );
        let _ = cx.layout_in(
            self.tab_scroll_edge_right_anchor,
            tab_scroll_edge_right_anchor_rect,
        );

        let candidate_rect_for = |kind: fret_runtime::DockDropCandidateRectKind, zone: DropZone| {
            hint_candidates
                .iter()
                .find(|c| c.kind == kind && c.zone == Some(zone))
                .map(|c| c.rect)
        };
        for (zone, anchor) in self.dock_hint_inner_anchors.iter().copied() {
            let rect =
                candidate_rect_for(fret_runtime::DockDropCandidateRectKind::InnerHintRect, zone)
                    .unwrap_or(fallback_hint_rect);
            let _ = cx.layout_in(anchor, rect);
        }
        for (zone, anchor) in self.dock_hint_outer_anchors.iter().copied() {
            let rect =
                candidate_rect_for(fret_runtime::DockDropCandidateRectKind::OuterHintRect, zone)
                    .unwrap_or(fallback_hint_rect);
            let _ = cx.layout_in(anchor, rect);
        }

        let floating_anchor_rect = (|| {
            let dock = cx.app.global::<DockManager>()?;
            let floating = dock.graph.floating_windows(self.window).last()?;
            let outer = floating.rect;
            let x = outer.origin.x.0 + outer.size.width.0 * 0.5;
            // Heuristic: stay inside the floating title bar even if tokens vary.
            let y = outer.origin.y.0 + 12.0;
            Some(Rect::new(
                Point::new(Px(x - half), Px(y - half)),
                Size::new(
                    DOCKING_ARBITRATION_DRAG_ANCHOR_SIZE,
                    DOCKING_ARBITRATION_DRAG_ANCHOR_SIZE,
                ),
            ))
        })()
        // When no in-window floating exists yet, keep the anchor offscreen so scripts won't
        // accidentally hit it before creating a floating container.
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(
                    Px(bounds.origin.x.0 - 2000.0),
                    Px(bounds.origin.y.0 - 2000.0),
                ),
                Size::new(
                    DOCKING_ARBITRATION_DRAG_ANCHOR_SIZE,
                    DOCKING_ARBITRATION_DRAG_ANCHOR_SIZE,
                ),
            )
        });
        let _ = cx.layout_in(self.floating_title_bar_anchor, floating_anchor_rect);

        let handle_bounds = (|| {
            fn first_handle_for_axis(
                graph: &fret_core::DockGraph,
                node: fret_core::DockNodeId,
                bounds: Rect,
                desired_axis: fret_core::Axis,
                split_handle_gap: Px,
                split_handle_hit_thickness: Px,
            ) -> Option<Rect> {
                let n = graph.node(node)?;
                match n {
                    fret_core::DockNode::Tabs { .. } => None,
                    fret_core::DockNode::Floating { child } => first_handle_for_axis(
                        graph,
                        *child,
                        bounds,
                        desired_axis,
                        split_handle_gap,
                        split_handle_hit_thickness,
                    ),
                    fret_core::DockNode::Split {
                        axis,
                        children,
                        fractions,
                    } => {
                        let count = children.len();
                        if count == 0 {
                            return None;
                        }
                        let computed = resizable::compute_layout(
                            *axis,
                            bounds,
                            count,
                            fractions,
                            split_handle_gap,
                            split_handle_hit_thickness,
                            &[],
                        );
                        if *axis == desired_axis {
                            if let Some(handle) = computed.handle_hit_rects.first().copied() {
                                return Some(handle);
                            }
                        }
                        for (&child, &rect) in children.iter().zip(computed.panel_rects.iter()) {
                            if let Some(found) = first_handle_for_axis(
                                graph,
                                child,
                                rect,
                                desired_axis,
                                split_handle_gap,
                                split_handle_hit_thickness,
                            ) {
                                return Some(found);
                            }
                        }
                        None
                    }
                }
            }

            let dock = cx.app.global::<DockManager>()?;
            let root = dock.graph.window_root(self.window)?;
            first_handle_for_axis(
                &dock.graph,
                root,
                bounds,
                fret_core::Axis::Horizontal,
                split_handle_gap,
                split_handle_hit_thickness,
            )
        })();

        let handle_rect = handle_bounds.map(|r| {
            let cx = r.origin.x.0 + r.size.width.0 * 0.5;
            let cy = r.origin.y.0 + r.size.height.0 * 0.5;
            let half = DOCKING_ARBITRATION_SPLIT_HANDLE_ANCHOR_SIZE.0 * 0.5;
            Rect::new(
                Point::new(Px(cx - half), Px(cy - half)),
                Size::new(
                    DOCKING_ARBITRATION_SPLIT_HANDLE_ANCHOR_SIZE,
                    DOCKING_ARBITRATION_SPLIT_HANDLE_ANCHOR_SIZE,
                ),
            )
        });
        let hidden = Rect::new(
            Point::new(Px(-1_000_000.0), Px(-1_000_000.0)),
            Size::new(
                DOCKING_ARBITRATION_SPLIT_HANDLE_ANCHOR_SIZE,
                DOCKING_ARBITRATION_SPLIT_HANDLE_ANCHOR_SIZE,
            ),
        );
        let _ = cx.layout_in(
            self.viewport_split_handle_anchor,
            handle_rect.unwrap_or(hidden),
        );

        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        // Keep retained diagnostics anchors (e.g. dock hint rect test_ids) synced to the latest
        // docking candidate rects while a dock drag is in flight. The harness lays out these
        // anchors from `WindowInteractionDiagnosticsStore` during layout, so ensure layout runs
        // continuously during drags even if only paint would otherwise be invalidated.
        let dock_drag_active = cx.app.drag(fret_core::PointerId(0)).is_some_and(|d| {
            (d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                || d.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
                && d.dragging
        });
        if dock_drag_active {
            cx.request_animation_frame();
            cx.tree.invalidate(cx.node, Invalidation::Layout);
        }

        if let Some(bounds) = cx.child_bounds(self.dock_space) {
            cx.paint(self.dock_space, bounds);
        } else {
            cx.paint(self.dock_space, cx.bounds);
        }
    }
}

#[derive(Default)]
struct DemoViewportToolState {
    tools: HashMap<ViewportKey, fret_editor::ViewportToolManager>,
}
#[derive(Clone)]
struct DockingArbitrationPanelModels {
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    drop_mask_disallow_left_edge: Model<bool>,
    last_viewport_input: Model<Arc<str>>,
    synth_pointer_debug: Model<Arc<str>>,
}

#[derive(Default)]
struct DockingArbitrationPanelModelsService {
    by_window: HashMap<AppWindowId, DockingArbitrationPanelModels>,
}

impl DockingArbitrationPanelModelsService {
    fn set(&mut self, window: AppWindowId, models: DockingArbitrationPanelModels) {
        self.by_window.insert(window, models);
    }

    fn get(&self, window: AppWindowId) -> Option<&DockingArbitrationPanelModels> {
        self.by_window.get(&window)
    }
}

struct DockingArbitrationDockPanelRegistry;

impl DockPanelRegistry<App> for DockingArbitrationDockPanelRegistry {
    fn render_panel(
        &self,
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        panel: &fret_core::PanelKey,
    ) -> Option<fret_core::NodeId> {
        let kind = panel.kind.0.as_str();
        match kind {
            "demo.viewport.left" => {
                let root_name = "dock.panel.viewport_left";
                return Some(render_cached_panel_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    root_name,
                    |cx| {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Fill;
                        layout.size.height = fret_ui::element::Length::Fill;
                        vec![cx.semantics(
                            fret_ui::element::SemanticsProps {
                                layout,
                                role: fret_core::SemanticsRole::Viewport,
                                test_id: Some(Arc::<str>::from("dock-arb-viewport-left")),
                                ..Default::default()
                            },
                            |_cx| vec![],
                        )]
                    },
                ));
            }
            "demo.viewport.right" => {
                let root_name = "dock.panel.viewport_right";
                return Some(render_cached_panel_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    root_name,
                    |cx| {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Fill;
                        layout.size.height = fret_ui::element::Length::Fill;
                        vec![cx.semantics(
                            fret_ui::element::SemanticsProps {
                                layout,
                                role: fret_core::SemanticsRole::Viewport,
                                test_id: Some(Arc::<str>::from("dock-arb-viewport-right")),
                                ..Default::default()
                            },
                            |_cx| vec![],
                        )]
                    },
                ));
            }
            "demo.controls" => {}
            _ => {
                let Some(suffix) = kind.strip_prefix("demo.viewport.extra.") else {
                    return None;
                };

                let root_name = format!("dock.panel.viewport_extra_{suffix}");
                let test_id = Arc::<str>::from(format!("dock-arb-viewport-extra-{suffix}"));
                return Some(render_cached_panel_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    &root_name,
                    |cx| {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Fill;
                        layout.size.height = fret_ui::element::Length::Fill;
                        vec![cx.semantics(
                            fret_ui::element::SemanticsProps {
                                layout,
                                role: fret_core::SemanticsRole::Viewport,
                                test_id: Some(test_id.clone()),
                                ..Default::default()
                            },
                            |_cx| vec![],
                        )]
                    },
                ));
            }
        }

        let models = app
            .global::<DockingArbitrationPanelModelsService>()
            .and_then(|svc| svc.get(window))
            .cloned()?;

        let captured = format!("captured={:?}", ui.captured());
        let layer_lines: Vec<String> = ui
            .debug_layers_in_paint_order()
            .iter()
            .enumerate()
            .map(|(ix, layer)| {
                format!(
                    "#{ix} root={:?} visible={} barrier={} hit_testable={} outside={} move={} timer={}",
                    layer.root,
                    layer.visible,
                    layer.blocks_underlay_input,
                    layer.hit_testable,
                    layer.wants_pointer_down_outside_events,
                    layer.wants_pointer_move_events,
                    layer.wants_timer_events
                )
            })
            .collect();

        let root_name = "dock.panel.controls";
        Some(
            declarative::RenderRootContext::new(ui, app, services, window, bounds).render_root(
                root_name,
                |cx| {
                cx.observe_model(&models.popover_open, Invalidation::Layout);
                cx.observe_model(&models.dialog_open, Invalidation::Layout);
                cx.observe_model(&models.drop_mask_disallow_left_edge, Invalidation::Layout);
                cx.observe_model(&models.last_viewport_input, Invalidation::Layout);
                cx.observe_model(&models.synth_pointer_debug, Invalidation::Layout);

                let theme = Theme::global(&*cx.app);
                let padding = theme.metric_token("metric.padding.md");
                let background = theme.color_token("background");

                let drag_state = cx
                    .app
                    .drag(fret_core::PointerId(0))
                    .map(|d| format!("drag(kind={:?}, dragging={})", d.kind, d.dragging))
                    .unwrap_or_else(|| "drag(<none>)".to_string());

                let last = cx
                    .app
                    .models()
                    .get_cloned(&models.last_viewport_input)
                    .unwrap_or_else(|| Arc::<str>::from("<missing>"));
                let synth_debug = cx
                    .app
                    .models()
                    .get_cloned(&models.synth_pointer_debug)
                    .unwrap_or_else(|| Arc::<str>::from("<missing>"));

                let popover_open = models.popover_open.clone();
                let dialog_open = models.dialog_open.clone();
                let drop_mask_disallow_left_edge = models.drop_mask_disallow_left_edge.clone();
                let sonner = shadcn::Sonner::global(&mut *cx.app);
                let popover_is_open = cx
                    .app
                    .models()
                    .get_cloned(&popover_open)
                    .unwrap_or(false);
                let dialog_is_open = cx
                    .app
                    .models()
                    .get_cloned(&dialog_open)
                    .unwrap_or(false);
                let drop_mask_left_disallowed = cx
                    .app
                    .models()
                    .get_cloned(&drop_mask_disallow_left_edge)
                    .unwrap_or(false);
                let disallow_left_flag = cx
                    .app
                    .global::<DockingArbitrationPolicyFlags>()
                    .map(|f| f.disallow_left_edge.clone())
                    .unwrap_or_else(|| Arc::new(AtomicBool::new(false)));

                let popover = shadcn::Popover::new(popover_open.clone())
                    .auto_focus(true)
                    .into_element(
                        cx,
                        |cx| {
                            shadcn::Button::new("Open popover")
                                .variant(shadcn::ButtonVariant::Outline)
                                .test_id("dock-arb-popover-trigger")
                                .toggle_model(popover_open.clone())
                                .into_element(cx)
                        },
                        |cx| {
                            shadcn::PopoverContent::new(vec![
                                cx.text("Non-modal overlay (Popover)."),
                                shadcn::Button::new("Close")
                                    .variant(shadcn::ButtonVariant::Secondary)
                                    .test_id("dock-arb-popover-close")
                                    .toggle_model(popover_open.clone())
                                    .into_element(cx),
                            ])
                            .into_element(cx)
                        },
                    );

                let dialog = shadcn::Dialog::new(dialog_open.clone()).into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new("Open modal dialog")
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("dock-arb-dialog-trigger")
                            .toggle_model(dialog_open.clone())
                            .into_element(cx)
                    },
                    |cx| {
                        let sonner_for_dialog = sonner.clone();
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Fill;
                        layout.size.height = fret_ui::element::Length::Fill;
                        cx.semantics(
                            fret_ui::element::SemanticsProps {
                                layout,
                                role: fret_core::SemanticsRole::Dialog,
                                test_id: Some(Arc::<str>::from("dock-arb-dialog-content")),
                                ..Default::default()
                            },
                            |cx| {
                                vec![shadcn::DialogContent::new(vec![
                                    shadcn::DialogHeader::new(vec![
                                        shadcn::DialogTitle::new("Dialog").into_element(cx),
                                        shadcn::DialogDescription::new(
                                            "Modal barrier should block docking + viewport input.",
                                        )
                                        .into_element(cx),
                                    ])
                                    .into_element(cx),
                                    shadcn::Button::new("Trigger toast (Sonner)")
                                        .variant(shadcn::ButtonVariant::Secondary)
                                        .test_id("dock-arb-sonner-trigger")
                                        .on_activate(Arc::new(move |host, action_cx, _reason| {
                                            sonner_for_dialog.toast(
                                                host,
                                                action_cx.window,
                                                shadcn::ToastRequest::new(
                                                    "Toast while modal is open",
                                                )
                                                .duration(None)
                                                .test_id("dock-arb-sonner-toast"),
                                            );
                                        }))
                                        .into_element(cx),
                                    shadcn::DialogFooter::new(vec![
                                        shadcn::Button::new("Close")
                                            .variant(shadcn::ButtonVariant::Secondary)
                                            .test_id("dock-arb-dialog-close")
                                            .toggle_model(dialog_open.clone())
                                            .into_element(cx),
                                    ])
                                    .into_element(cx),
                                ])
                                .into_element(cx)]
                            },
                        )
                    },
                );

                vec![cx.container(
                ContainerProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    padding: fret_core::Edges::all(padding).into(),
                    background: Some(background),
                    ..Default::default()
                },
                |cx| {
                    let header_card = shadcn::Card::new(vec![
                        shadcn::CardHeader::new(vec![
                            shadcn::CardTitle::new("Docking arbitration demo").into_element(cx),
                            shadcn::CardDescription::new("ADR 0072").into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::CardContent::new(vec![vstack(
                            cx,
                            VStackProps::default()
                                .gap(Space::N1)
                                .layout(LayoutRefinement::default().w_full()),
                            |cx| {
                                vec![
                                    shadcn::CardDescription::new(
                                        "Open a popover, then drag a dock tab.",
                                    )
                                    .into_element(cx),
                                    shadcn::CardDescription::new(
                                        "Start viewport drag inside the blue border.",
                                    )
                                    .into_element(cx),
                                    shadcn::CardDescription::new(
                                        "Open a modal dialog to validate underlay blocking.",
                                    )
                                    .into_element(cx),
                                ]
                            },
                        )])
                        .into_element(cx),
                    ])
                    .size(shadcn::CardSize::Sm)
                    .into_element(cx);

                    let state_card = shadcn::Card::new(vec![
                        shadcn::CardHeader::new(vec![shadcn::CardTitle::new("State").into_element(cx)])
                            .into_element(cx),
                        shadcn::CardContent::new(vec![vstack(
                            cx,
                            VStackProps::default()
                                .gap(Space::N2)
                                .layout(LayoutRefinement::default().w_full()),
                            |cx| {
                                vec![
                                    shadcn::CardDescription::new(drag_state).into_element(cx),
                                    shadcn::CardDescription::new(captured.clone()).into_element(cx),
                                    shadcn::CardDescription::new(format!(
                                        "last_viewport_input={last}"
                                    ))
                                    .into_element(cx),
                                    shadcn::Separator::new().into_element(cx),
                                    shadcn::CardDescription::new(
                                        "Synth pointer: F1 toggle; I/J/K/L move; Space down/up; B right down/up; U/O wheel up/down.",
                                    )
                                    .into_element(cx),
                                    shadcn::CardDescription::new(synth_debug.to_string())
                                        .into_element(cx),
                                    shadcn::Separator::new().into_element(cx),
                                    cx.semantics(
                                        fret_ui::element::SemanticsProps {
                                            role: fret_core::SemanticsRole::Text,
                                            test_id: Some(Arc::<str>::from(if popover_is_open {
                                                "dock-arb-popover-open"
                                            } else {
                                                "dock-arb-popover-closed"
                                            })),
                                            label: Some(Arc::<str>::from(if popover_is_open {
                                                "popover:open"
                                            } else {
                                                "popover:closed"
                                            })),
                                            ..Default::default()
                                        },
                                        |cx| {
                                            vec![cx.text(if popover_is_open {
                                                "Popover: open"
                                            } else {
                                                "Popover: closed"
                                            })]
                                        },
                                    ),
                                    cx.semantics(
                                        fret_ui::element::SemanticsProps {
                                            role: fret_core::SemanticsRole::Text,
                                            test_id: Some(Arc::<str>::from(if dialog_is_open {
                                                "dock-arb-dialog-open"
                                            } else {
                                                "dock-arb-dialog-closed"
                                            })),
                                            label: Some(Arc::<str>::from(if dialog_is_open {
                                                "dialog:open"
                                            } else {
                                                "dialog:closed"
                                            })),
                                            ..Default::default()
                                        },
                                        |cx| {
                                            vec![cx.text(if dialog_is_open {
                                                "Dialog: open"
                                            } else {
                                                "Dialog: closed"
                                            })]
                                        },
                                    ),
                                    cx.semantics(
                                        fret_ui::element::SemanticsProps {
                                            role: fret_core::SemanticsRole::Text,
                                            test_id: Some(Arc::<str>::from(if drop_mask_left_disallowed {
                                                "dock-arb-drop-mask-left-disallowed"
                                            } else {
                                                "dock-arb-drop-mask-left-allowed"
                                            })),
                                            label: Some(Arc::<str>::from(if drop_mask_left_disallowed {
                                                "drop_mask_left:disallowed"
                                            } else {
                                                "drop_mask_left:allowed"
                                            })),
                                            ..Default::default()
                                        },
                                        |cx| {
                                            vec![cx.text(if drop_mask_left_disallowed {
                                                "Drop mask: left edge docking disallowed"
                                            } else {
                                                "Drop mask: left edge docking allowed"
                                            })]
                                        },
                                    ),
                                ]
                            },
                        )])
                        .into_element(cx),
                    ])
                    .size(shadcn::CardSize::Sm)
                    .into_element(cx);

                    let actions_card = shadcn::Card::new(vec![
                        shadcn::CardContent::new(vec![vstack(
                            cx,
                            VStackProps::default()
                                .gap(Space::N1)
                                .layout(LayoutRefinement::default().w_full()),
                            |cx| {
                                vec![
                                    cx.keyed("dock-arb-action-close-left", |cx| {
                                        shadcn::Button::new("Close viewport left (panel)")
                                            .variant(shadcn::ButtonVariant::Outline)
                                            .test_id("dock-arb-close-viewport-left")
                                            .on_activate(Arc::new(
                                                move |host, action_cx, _reason| {
                                                    host.push_effect(Effect::Dock(
                                                        fret_core::DockOp::ClosePanel {
                                                            window: action_cx.window,
                                                            panel: fret_core::PanelKey::new(
                                                                "demo.viewport.left",
                                                            ),
                                                        },
                                                    ));
                                                    host.request_redraw(action_cx.window);
                                                },
                                            ))
                                            .into_element(cx)
                                    }),
                                    cx.keyed("dock-arb-action-close-right", |cx| {
                                        shadcn::Button::new("Close viewport right (panel)")
                                            .variant(shadcn::ButtonVariant::Outline)
                                            .test_id("dock-arb-close-viewport-right")
                                            .on_activate(Arc::new(
                                                move |host, action_cx, _reason| {
                                                    host.push_effect(Effect::Dock(
                                                        fret_core::DockOp::ClosePanel {
                                                            window: action_cx.window,
                                                            panel: fret_core::PanelKey::new(
                                                                "demo.viewport.right",
                                                            ),
                                                        },
                                                    ));
                                                    host.request_redraw(action_cx.window);
                                                },
                                            ))
                                            .into_element(cx)
                                    }),
                                    cx.keyed("dock-arb-action-close-controls", |cx| {
                                        shadcn::Button::new("Close controls (panel)")
                                            .variant(shadcn::ButtonVariant::Outline)
                                            .test_id("dock-arb-close-controls")
                                            .on_activate(Arc::new(
                                                move |host, action_cx, _reason| {
                                                    host.push_effect(Effect::Dock(
                                                        fret_core::DockOp::ClosePanel {
                                                            window: action_cx.window,
                                                            panel: fret_core::PanelKey::new(
                                                                "demo.controls",
                                                            ),
                                                        },
                                                    ));
                                                    host.request_redraw(action_cx.window);
                                                },
                                            ))
                                            .into_element(cx)
                                    }),
                                    cx.keyed("dock-arb-action-drop-mask", |cx| {
                                        shadcn::Button::new("Toggle drop mask (left edge)")
                                            .variant(shadcn::ButtonVariant::Outline)
                                            .test_id("dock-arb-toggle-drop-mask-left-edge")
                                            .on_activate(Arc::new(move |host, _action_cx, _reason| {
                                                let mut next = false;
                                                let _ = host.models_mut().update(
                                                    &drop_mask_disallow_left_edge,
                                                    |v| {
                                                        *v = !*v;
                                                        next = *v;
                                                    },
                                                );
                                                disallow_left_flag.store(next, Ordering::Relaxed);
                                            }))
                                            .into_element(cx)
                                    }),
                                    cx.keyed("dock-arb-action-popover", |_cx| popover),
                                    cx.keyed("dock-arb-action-dialog", |_cx| dialog),
                                    cx.keyed("dock-arb-action-underlay", |cx| {
                                        shadcn::Button::new("Underlay (modal barrier target)")
                                            .variant(shadcn::ButtonVariant::Secondary)
                                            .test_id("dock-arb-underlay-probe")
                                            .into_element(cx)
                                    }),
                                ]
                            },
                        )])
                        .into_element(cx),
                    ])
                    .size(shadcn::CardSize::Sm)
                    .into_element(cx);

                    let debug_layers_card = shadcn::Card::new(vec![
                        shadcn::CardHeader::new(vec![
                            shadcn::CardTitle::new("Debug").into_element(cx),
                            shadcn::CardDescription::new("Paint order layers").into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::CardContent::new(vec![shadcn::Collapsible::uncontrolled(false)
                            .into_element(
                                cx,
                                |cx, is_open| {
                                    shadcn::Button::new(if is_open {
                                        "Hide debug layers"
                                    } else {
                                        "Show debug layers"
                                    })
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .into_element(cx)
                                },
                                |cx| {
                                    let content = vstack(
                                        cx,
                                        VStackProps::default()
                                            .gap(Space::N1)
                                            .layout(LayoutRefinement::default().w_full()),
                                        |cx| {
                                            layer_lines
                                                .iter()
                                                .cloned()
                                                .map(|v| cx.text(v))
                                                .collect::<Vec<_>>()
                                        },
                                    );

                                    shadcn::ScrollArea::new(vec![content])
                                        .refine_layout(
                                            LayoutRefinement::default()
                                                .w_full()
                                                .h_px(Px(240.0))
                                                .min_w_0()
                                                .min_h_0(),
                                        )
                                        .into_element(cx)
                                },
                            )])
                        .into_element(cx),
                    ])
                    .size(shadcn::CardSize::Sm)
                    .into_element(cx);

                    vec![vstack(
                        cx,
                        VStackProps::default()
                            .gap(Space::N3)
                            .layout(LayoutRefinement::default().size_full().min_w_0().min_h_0()),
                        |_cx| vec![actions_card, header_card, state_card, debug_layers_card],
                    ), shadcn::Toaster::new().into_element(cx)]
                },
            )]
                },
            ),
        )
    }
}

#[derive(Default)]
struct ViewportDebugService {
    last_event: HashMap<AppWindowId, Model<Arc<str>>>,
}

struct DemoViewportOverlayHooks {
    tools: Arc<Mutex<DemoViewportToolState>>,
}

impl DockViewportOverlayHooks for DemoViewportOverlayHooks {
    fn paint_with_layout(
        &self,
        theme: fret_ui::ThemeSnapshot,
        window: AppWindowId,
        _panel: &fret_core::PanelKey,
        viewport: fret_docking::ViewportPanel,
        layout: fret_docking::DockViewportLayout,
        scene: &mut Scene,
    ) {
        let border_color = Color {
            a: 0.80,
            ..theme.color_token("primary")
        };
        let draw_rect = layout.draw_rect;
        scene.push(SceneOp::Quad {
            order: DrawOrder(6),
            rect: draw_rect,
            background: fret_core::Paint::TRANSPARENT.into(),

            border: Edges::all(Px(2.0)),
            border_paint: fret_core::Paint::Solid(border_color).into(),
            corner_radii: Corners::all(Px(0.0)),
        });

        let overlay = self.tools.lock().ok().and_then(|state| {
            state
                .tools
                .get(&(window, viewport.target))
                .map(|m| m.overlay())
        });
        if let Some(overlay) = overlay {
            fret_editor::paint_viewport_overlay(theme, layout.draw_rect, overlay, scene);
        }
    }
}

struct DockingArbitrationWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    dock_space: Option<fret_core::NodeId>,
}

#[derive(Default)]
struct DockingArbitrationDriver {
    main_window: Option<AppWindowId>,
    docking_runtime: Option<DockingRuntime>,
    pending_layout: Option<fret_core::DockLayout>,
    restore: Option<DockLayoutRestoreState>,
    logical_windows: HashMap<AppWindowId, String>,
    next_logical_window_ix: u32,
    viewport_tools: Arc<Mutex<DemoViewportToolState>>,
    synth_pointers: HashMap<AppWindowId, SynthPointerState>,
    next_synth_touch_id: u64,
    layout_preset: DockingArbitrationLayoutPreset,
    persist_layout_on_exit: bool,
    diag_layout_sanitized: bool,
}

#[derive(Debug, Clone)]
struct SynthPointerState {
    enabled: bool,
    pointer_id: fret_core::PointerId,
    position: Point,
    pressed: bool,
    mouse_right_pressed: bool,
}

impl Default for SynthPointerState {
    fn default() -> Self {
        Self {
            enabled: false,
            // Use the same namespace as runner touch IDs (see `fret-runner-winit`):
            // `PointerId(0)` is reserved for mouse, so we pick a stable touch-like ID.
            pointer_id: fret_core::PointerId((1u64 << 56) | 42),
            position: Point::new(Px(160.0), Px(120.0)),
            pressed: false,
            mouse_right_pressed: false,
        }
    }
}

struct DockLayoutRestoreState {
    layout: fret_core::DockLayout,
    pending_logical_window_ids: HashSet<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum DockingArbitrationLayoutPreset {
    #[default]
    Default,
    Large,
    OverflowTabs,
}

impl DockingArbitrationLayoutPreset {
    fn from_env() -> Self {
        let Some(raw) = std::env::var("FRET_DOCK_ARB_PRESET").ok() else {
            return Self::Default;
        };
        match raw.trim().to_ascii_lowercase().as_str() {
            "" | "default" => Self::Default,
            "large" | "perf_large" | "perf-large" => Self::Large,
            "overflow_tabs" | "overflow-tabs" => Self::OverflowTabs,
            _ => Self::Default,
        }
    }
}

impl DockingArbitrationDriver {
    const DOCK_LAYOUT_PATH: &'static str = ".fret/layout.json";
    const MAIN_LOGICAL_WINDOW_ID: &'static str = "main";

    fn sync_dev_state_models(&self, app: &mut App) {
        let mut windows: Vec<(AppWindowId, String)> = self
            .logical_windows
            .iter()
            .map(|(w, id)| (*w, id.clone()))
            .collect();
        windows.sort_by(|a, b| a.1.cmp(&b.1));

        let persist_layout_on_exit = self.persist_layout_on_exit;
        app.with_global_mut_untracked(DockingArbitrationDevStateModels::default, |st, _app| {
            st.persist_layout_on_exit = persist_layout_on_exit;
            st.windows = windows;
        });
    }

    fn mark_dev_state_export_ready(app: &mut App) {
        app.with_global_mut_untracked(DockingArbitrationDevStateGate::default, |gate, _app| {
            gate.export_ready = true;
        });
    }

    fn new(
        pending_layout: Option<fret_core::DockLayout>,
        viewport_tools: Arc<Mutex<DemoViewportToolState>>,
        layout_preset: DockingArbitrationLayoutPreset,
        persist_layout_on_exit: bool,
    ) -> Self {
        let mut next_logical_window_ix = 1;
        if let Some(layout) = &pending_layout {
            for w in &layout.windows {
                let Some(suffix) = w.logical_window_id.strip_prefix("floating-") else {
                    continue;
                };
                let Ok(ix) = suffix.parse::<u32>() else {
                    continue;
                };
                next_logical_window_ix = next_logical_window_ix.max(ix.saturating_add(1));
            }
        }
        Self {
            main_window: None,
            docking_runtime: None,
            pending_layout,
            restore: None,
            logical_windows: HashMap::new(),
            next_logical_window_ix,
            viewport_tools,
            synth_pointers: HashMap::new(),
            next_synth_touch_id: 42,
            layout_preset,
            persist_layout_on_exit,
            diag_layout_sanitized: false,
        }
    }

    fn synth_pointer_mut(&mut self, app: &App, window: AppWindowId) -> &mut SynthPointerState {
        let st = match self.synth_pointers.entry(window) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let mut st = SynthPointerState::default();
                st.pointer_id = fret_core::PointerId((1u64 << 56) | self.next_synth_touch_id);
                self.next_synth_touch_id = self.next_synth_touch_id.saturating_add(1);
                entry.insert(st)
            }
        };
        if !st.enabled {
            if let Some(metrics) = app.global::<fret_core::WindowMetricsService>()
                && let Some(size) = metrics.inner_size(window)
            {
                st.position = Point::new(Px(size.width.0 * 0.5), Px(size.height.0 * 0.5));
            }
        }
        st
    }

    fn update_synth_debug(app: &mut App, window: AppWindowId, synth: &SynthPointerState) {
        let Some(models) = app
            .global::<DockingArbitrationPanelModelsService>()
            .and_then(|svc| svc.get(window))
            .cloned()
        else {
            return;
        };

        let drag = app
            .drag(synth.pointer_id)
            .map(|d| format!("drag(kind={:?}, dragging={})", d.kind, d.dragging))
            .unwrap_or_else(|| "drag(<none>)".to_string());

        let msg: Arc<str> = Arc::from(
            format!(
                "synth_pointer: enabled={} id={:?} pos=({:.1},{:.1}) down={} mouse_right_down={} {}",
                synth.enabled,
                synth.pointer_id,
                synth.position.x.0,
                synth.position.y.0,
                synth.pressed,
                synth.mouse_right_pressed,
                drag
            )
            .into_boxed_str(),
        );
        let _ = app
            .models_mut()
            .update(&models.synth_pointer_debug, |v| *v = msg);
        app.request_redraw(window);
    }

    fn dispatch_synth_pointer_move(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        synth: &SynthPointerState,
        modifiers: Modifiers,
    ) {
        if !synth.enabled {
            return;
        }
        let buttons = MouseButtons {
            left: synth.pressed,
            ..Default::default()
        };
        ui.dispatch_event(
            app,
            services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                pointer_id: synth.pointer_id,
                position: synth.position,
                buttons,
                modifiers,
                pointer_type: fret_core::PointerType::Touch,
            }),
        );
        app.request_redraw(window);
    }

    fn dispatch_synth_pointer_button(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        synth: &SynthPointerState,
        pressed: bool,
        modifiers: Modifiers,
    ) {
        if !synth.enabled {
            return;
        }
        let evt = if pressed {
            fret_core::PointerEvent::Down {
                pointer_id: synth.pointer_id,
                position: synth.position,
                button: MouseButton::Left,
                modifiers,
                click_count: 1,
                pointer_type: fret_core::PointerType::Touch,
            }
        } else {
            fret_core::PointerEvent::Up {
                pointer_id: synth.pointer_id,
                position: synth.position,
                button: MouseButton::Left,
                modifiers,
                is_click: true,
                click_count: 1,
                pointer_type: fret_core::PointerType::Touch,
            }
        };
        ui.dispatch_event(app, services, &Event::Pointer(evt));
        app.request_redraw(window);
    }

    fn dispatch_synth_mouse_button(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        synth: &SynthPointerState,
        button: MouseButton,
        pressed: bool,
        modifiers: Modifiers,
    ) {
        if !synth.enabled {
            return;
        }
        let evt = if pressed {
            fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: synth.position,
                button,
                modifiers,
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
            }
        } else {
            fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: synth.position,
                button,
                modifiers,
                is_click: true,
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
            }
        };
        ui.dispatch_event(app, services, &Event::Pointer(evt));
        app.request_redraw(window);
    }

    fn dispatch_synth_mouse_wheel(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        synth: &SynthPointerState,
        delta: Point,
        modifiers: Modifiers,
    ) {
        if !synth.enabled {
            return;
        }
        ui.dispatch_event(
            app,
            services,
            &Event::Pointer(fret_core::PointerEvent::Wheel {
                pointer_id: fret_core::PointerId(0),
                position: synth.position,
                delta,
                modifiers,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        app.request_redraw(window);
    }

    fn alloc_floating_logical_window_id(&mut self) -> String {
        let reserved = self.restore.as_ref().map(|r| &r.pending_logical_window_ids);

        loop {
            let logical = format!("floating-{}", self.next_logical_window_ix);
            self.next_logical_window_ix = self.next_logical_window_ix.saturating_add(1);

            if self.logical_windows.values().any(|v| v == &logical) {
                continue;
            }
            if reserved.is_some_and(|r| r.contains(&logical)) {
                continue;
            }
            return logical;
        }
    }

    fn build_ui(app: &mut App, window: AppWindowId) -> DockingArbitrationWindowState {
        let popover_open = app.models_mut().insert(false);
        let dialog_open = app.models_mut().insert(false);
        let drop_mask_disallow_left_edge = app.models_mut().insert(false);
        let last_viewport_input = app.models_mut().insert(Arc::<str>::from("<none>"));
        let synth_pointer_debug = app.models_mut().insert(Arc::<str>::from(
            "synth_pointer: enabled=false id=<unset> pos=(n/a) down=false mouse_right_down=false drag(<none>)",
        ));

        app.with_global_mut(ViewportDebugService::default, |svc, _app| {
            svc.last_event.insert(window, last_viewport_input.clone());
        });

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(std::env::var_os("FRET_EXAMPLES_VIEW_CACHE").is_some());

        app.with_global_mut(
            DockingArbitrationPanelModelsService::default,
            |svc, _app| {
                svc.set(
                    window,
                    DockingArbitrationPanelModels {
                        popover_open: popover_open.clone(),
                        dialog_open: dialog_open.clone(),
                        drop_mask_disallow_left_edge: drop_mask_disallow_left_edge.clone(),
                        last_viewport_input: last_viewport_input.clone(),
                        synth_pointer_debug: synth_pointer_debug.clone(),
                    },
                );
            },
        );

        DockingArbitrationWindowState {
            ui,
            root: None,
            dock_space: None,
        }
    }

    fn ensure_dock_graph(&mut self, app: &mut App, window: AppWindowId) {
        use fret_core::{DockNode, PanelKey};

        let diag_enabled = app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| svc.is_enabled());
        let layout_preset = self.layout_preset;
        let main_window = self.main_window;
        let should_sanitize_for_diag =
            diag_enabled && Some(window) == main_window && !self.diag_layout_sanitized;
        if should_sanitize_for_diag {
            self.diag_layout_sanitized = true;
        }

        app.with_global_mut(DockManager::default, |dock, _app| {
            let viewport_left = PanelKey::new("demo.viewport.left");
            let viewport_right = PanelKey::new("demo.viewport.right");
            let controls_panel = PanelKey::new("demo.controls");

            dock.ensure_panel(&viewport_left, || DockPanel {
                title: "Viewport Left".to_string(),
                color: Color::TRANSPARENT,
                viewport: Some(fret_docking::ViewportPanel {
                    target: RenderTargetId::from(KeyData::from_ffi(1)),
                    target_px_size: (960, 540),
                    fit: fret_core::ViewportFit::Stretch,
                    context_menu_enabled: true,
                }),
            });
            dock.ensure_panel(&viewport_right, || DockPanel {
                title: "Viewport Right".to_string(),
                color: Color::TRANSPARENT,
                viewport: Some(fret_docking::ViewportPanel {
                    target: RenderTargetId::from(KeyData::from_ffi(2)),
                    target_px_size: (960, 540),
                    fit: fret_core::ViewportFit::Stretch,
                    context_menu_enabled: true,
                }),
            });
            dock.ensure_panel(&controls_panel, || DockPanel {
                title: "Controls".to_string(),
                color: Color::TRANSPARENT,
                viewport: None,
            });

            // Diagnostics scripts assume a stable baseline dock graph. When diagnostics are enabled,
            // force the main window to start from the selected preset even if a previous session
            // left a persisted layout in place.
            //
            // Important: only do this for the main window and only once, otherwise multi-window
            // tear-off scripts would be disrupted by resetting the graph after creating new OS
            // windows.
            if should_sanitize_for_diag {
                dock.graph = fret_core::DockGraph::new();
            } else if dock.graph.window_root(window).is_some() {
                return;
            }

            // Only seed a baseline root for the main window. Tear-off windows should start from
            // the docking runtime's state (often an in-window floating payload hosted by a new OS
            // window), and multi-window restore will populate roots once all windows exist.
            if Some(window) != main_window {
                return;
            }

            fn tabs_for_panel(
                graph: &mut fret_core::DockGraph,
                panel: PanelKey,
            ) -> fret_core::DockNodeId {
                graph.insert_node(DockNode::Tabs {
                    tabs: vec![panel],
                    active: 0,
                })
            }

            fn row_split(
                graph: &mut fret_core::DockGraph,
                children: Vec<fret_core::DockNodeId>,
            ) -> fret_core::DockNodeId {
                let denom = (children.len().max(1) as f32).max(1.0);
                let fractions = vec![1.0 / denom; children.len()];
                graph.insert_node(DockNode::Split {
                    axis: fret_core::Axis::Horizontal,
                    children,
                    fractions,
                })
            }

            match layout_preset {
                DockingArbitrationLayoutPreset::Default => {
                    let tabs_left = tabs_for_panel(&mut dock.graph, viewport_left);
                    let tabs_right = tabs_for_panel(&mut dock.graph, viewport_right);
                    let viewport_split = dock.graph.insert_node(DockNode::Split {
                        axis: fret_core::Axis::Horizontal,
                        children: vec![tabs_left, tabs_right],
                        fractions: vec![0.5, 0.5],
                    });
                    let tabs_controls = tabs_for_panel(&mut dock.graph, controls_panel);
                    let root = dock.graph.insert_node(DockNode::Split {
                        axis: fret_core::Axis::Vertical,
                        children: vec![viewport_split, tabs_controls],
                        fractions: vec![0.7, 0.3],
                    });
                    dock.graph.set_window_root(window, root);
                }
                DockingArbitrationLayoutPreset::Large => {
                    let extra_viewports: Vec<PanelKey> = (0..10)
                        .map(|ix| PanelKey::new(format!("demo.viewport.extra.{ix}")))
                        .collect();

                    for (ix, key) in extra_viewports.iter().enumerate() {
                        let title = format!("Viewport Extra {ix}");
                        let target = RenderTargetId::from(KeyData::from_ffi(10 + ix as u64));
                        dock.ensure_panel(key, || DockPanel {
                            title,
                            color: Color::TRANSPARENT,
                            viewport: Some(fret_docking::ViewportPanel {
                                target,
                                target_px_size: (960, 540),
                                fit: fret_core::ViewportFit::Stretch,
                                context_menu_enabled: true,
                            }),
                        });
                    }

                    let row1 = vec![
                        tabs_for_panel(&mut dock.graph, viewport_left),
                        tabs_for_panel(&mut dock.graph, viewport_right),
                        tabs_for_panel(&mut dock.graph, extra_viewports[0].clone()),
                        tabs_for_panel(&mut dock.graph, extra_viewports[1].clone()),
                    ];
                    let row2: Vec<fret_core::DockNodeId> = (2..6)
                        .map(|ix| tabs_for_panel(&mut dock.graph, extra_viewports[ix].clone()))
                        .collect();
                    let row3: Vec<fret_core::DockNodeId> = (6..10)
                        .map(|ix| tabs_for_panel(&mut dock.graph, extra_viewports[ix].clone()))
                        .collect();

                    let row1 = row_split(&mut dock.graph, row1);
                    let row2 = row_split(&mut dock.graph, row2);
                    let row3 = row_split(&mut dock.graph, row3);
                    let controls = tabs_for_panel(&mut dock.graph, controls_panel);

                    let root_weights = [1.0f32, 1.0f32, 1.0f32, 0.8f32];
                    let root_sum: f32 = root_weights.iter().copied().sum();
                    let root_fractions: Vec<f32> = if root_sum.is_finite() && root_sum > 0.0 {
                        root_weights.iter().map(|w| w / root_sum).collect()
                    } else {
                        vec![0.25; 4]
                    };

                    let root = dock.graph.insert_node(DockNode::Split {
                        axis: fret_core::Axis::Vertical,
                        children: vec![row1, row2, row3, controls],
                        fractions: root_fractions,
                    });
                    dock.graph.set_window_root(window, root);
                }
                DockingArbitrationLayoutPreset::OverflowTabs => {
                    let extra_viewports: Vec<PanelKey> = (0..10)
                        .map(|ix| PanelKey::new(format!("demo.viewport.extra.{ix}")))
                        .collect();

                    for (ix, key) in extra_viewports.iter().enumerate() {
                        let title = format!("Viewport Extra {ix}");
                        let target = RenderTargetId::from(KeyData::from_ffi(10 + ix as u64));
                        dock.ensure_panel(key, || DockPanel {
                            title,
                            color: Color::TRANSPARENT,
                            viewport: Some(fret_docking::ViewportPanel {
                                target,
                                target_px_size: (960, 540),
                                fit: fret_core::ViewportFit::Stretch,
                                context_menu_enabled: true,
                            }),
                        });
                    }

                    let dummy_panel = PanelKey::new("demo.dummy.overflow");
                    dock.ensure_panel(&dummy_panel, || DockPanel {
                        title: "Dummy".to_string(),
                        color: Color::TRANSPARENT,
                        viewport: None,
                    });

                    let overflow_tabs = vec![
                        viewport_left.clone(),
                        viewport_right.clone(),
                        extra_viewports[0].clone(),
                        extra_viewports[1].clone(),
                        extra_viewports[2].clone(),
                        extra_viewports[3].clone(),
                        extra_viewports[4].clone(),
                        extra_viewports[5].clone(),
                        extra_viewports[6].clone(),
                        extra_viewports[7].clone(),
                    ];
                    let tabs_overflow = dock.graph.insert_node(DockNode::Tabs {
                        tabs: overflow_tabs,
                        active: 0,
                    });

                    let row1 = vec![
                        tabs_overflow,
                        tabs_for_panel(&mut dock.graph, extra_viewports[8].clone()),
                        tabs_for_panel(&mut dock.graph, extra_viewports[9].clone()),
                        tabs_for_panel(&mut dock.graph, dummy_panel),
                    ];
                    let row1 = row_split(&mut dock.graph, row1);
                    let controls = tabs_for_panel(&mut dock.graph, controls_panel);
                    let root = dock.graph.insert_node(DockNode::Split {
                        axis: fret_core::Axis::Vertical,
                        children: vec![row1, controls],
                        fractions: vec![0.7, 0.3],
                    });
                    dock.graph.set_window_root(window, root);
                }
            }
        });
    }

    fn apply_layout_if_ready(&mut self, app: &mut App) {
        let Some(main_window) = self.main_window else {
            return;
        };
        let Some(restore) = self.restore.as_mut() else {
            return;
        };
        if !restore.pending_logical_window_ids.is_empty() {
            return;
        }

        let mut windows: Vec<(AppWindowId, String)> = self
            .logical_windows
            .iter()
            .map(|(w, id)| (*w, id.clone()))
            .collect();
        windows.sort_by(|a, b| a.1.cmp(&b.1));

        app.with_global_mut(DockManager::default, |dock, app| {
            let changed = dock
                .graph
                .import_layout_for_windows(&restore.layout, &windows);
            if changed {
                fret_docking::runtime::request_dock_invalidation(app, dock.graph.windows());
                for w in dock.graph.windows() {
                    app.request_redraw(w);
                }
            }
        });

        self.restore = None;
        self.sync_dev_state_models(app);
        Self::mark_dev_state_export_ready(app);
        DevStateHooks::export_all(app);
        app.request_redraw(main_window);
    }

    fn try_restore_layout_on_init(&mut self, app: &mut App, main_window: AppWindowId) {
        if self.pending_layout.is_none()
            && self.layout_preset == DockingArbitrationLayoutPreset::Default
            && !diag_enabled_env()
        {
            let incoming = app.with_global_mut_untracked(
                DockingArbitrationDevStateIncoming::default,
                |st, _app| st.layout.take(),
            );
            if incoming.is_some() {
                self.pending_layout = incoming;
            }
        }

        let Some(layout) = self.pending_layout.take() else {
            self.sync_dev_state_models(app);
            Self::mark_dev_state_export_ready(app);
            return;
        };

        let multi_window = app
            .global::<PlatformCapabilities>()
            .map(|c| c.ui.multi_window)
            .unwrap_or(true);

        if !multi_window {
            app.with_global_mut(DockManager::default, |dock, app| {
                let changed = dock
                    .graph
                    .import_layout_for_windows_with_fallback_floatings(
                        &layout,
                        &[(main_window, Self::MAIN_LOGICAL_WINDOW_ID.to_string())],
                        main_window,
                    );
                if changed {
                    fret_docking::runtime::request_dock_invalidation(app, [main_window]);
                    app.request_redraw(main_window);
                }
            });
            self.sync_dev_state_models(app);
            Self::mark_dev_state_export_ready(app);
            DevStateHooks::export_all(app);
            return;
        }

        // Multi-window restore (best-effort): create OS windows for non-main logical windows, then
        // import the full layout once all windows exist. Until then, main window can still render
        // a default dock graph.
        let mut pending: HashSet<String> = HashSet::new();
        for w in &layout.windows {
            if w.logical_window_id == Self::MAIN_LOGICAL_WINDOW_ID {
                continue;
            }
            pending.insert(w.logical_window_id.clone());
            app.push_effect(Effect::Window(WindowRequest::Create(
                fret_app::CreateWindowRequest {
                    kind: CreateWindowKind::DockRestore {
                        logical_window_id: w.logical_window_id.clone(),
                    },
                    anchor: None,
                    role: fret_runtime::WindowRole::Auxiliary,
                    style: fret_runtime::WindowStyleRequest::default(),
                },
            )));
        }
        self.restore = Some(DockLayoutRestoreState {
            layout,
            pending_logical_window_ids: pending,
        });
        self.apply_layout_if_ready(app);
    }

    fn save_layout_on_exit(&mut self, app: &mut App) {
        let Some(main_window) = self.main_window else {
            return;
        };

        let mut windows: Vec<(AppWindowId, String)> = self
            .logical_windows
            .iter()
            .map(|(w, id)| (*w, id.clone()))
            .collect();
        windows.sort_by(|a, b| a.1.cmp(&b.1));

        let Some(metrics) = app.global::<fret_core::WindowMetricsService>() else {
            return;
        };

        let placements: HashMap<AppWindowId, fret_core::DockWindowPlacement> = windows
            .iter()
            .filter_map(|(window, _logical_window_id)| {
                let size = metrics.inner_size(*window)?;
                let width = (size.width.0.max(1.0).round() as u32).max(1);
                let height = (size.height.0.max(1.0).round() as u32).max(1);
                Some((
                    *window,
                    fret_core::DockWindowPlacement {
                        width,
                        height,
                        x: None,
                        y: None,
                        monitor_hint: None,
                    },
                ))
            })
            .collect();

        let layout = app.with_global_mut(DockManager::default, |dock, _app| {
            dock.graph
                .export_layout_with_placement(&windows, |window| placements.get(&window).cloned())
        });

        let file = fret_app::DockLayoutFileV1 { layout };
        if let Err(err) = file.save_json(Self::DOCK_LAYOUT_PATH) {
            tracing::warn!("failed to save dock layout: {err}");
        } else {
            app.request_redraw(main_window);
        }

        self.sync_dev_state_models(app);
        DevStateHooks::export_all(app);
    }

    fn render_dock(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut DockingArbitrationWindowState,
        bounds: Rect,
    ) {
        self.ensure_dock_graph(app, window);

        OverlayController::begin_frame(app, window);

        let dock_space = state.dock_space.get_or_insert_with(|| {
            use fret_ui::retained_bridge::UiTreeRetainedExt as _;
            let allow_chained_tear_off =
                std::env::var_os("FRET_DOCK_ALLOW_MULTI_WINDOW_TEAR_OFF").is_some();
            state.ui.create_node_retained(
                DockSpace::new(window)
                    .with_allow_multi_window_tear_off(allow_chained_tear_off)
                    .with_semantics_test_id("dock-arb-dock-space"),
            )
        });
        let _ = state.root.get_or_insert_with(|| {
            let left_anchor = state
                .ui
                .create_node_retained(DockingArbitrationDragAnchor::new(
                    "dock-arb-tab-drag-anchor-left",
                ));
            let right_anchor = state
                .ui
                .create_node_retained(DockingArbitrationDragAnchor::new(
                    "dock-arb-tab-drag-anchor-right",
                ));
            let right_tabs_group_anchor =
                state
                    .ui
                    .create_node_retained(DockingArbitrationDragAnchor::new(
                        "dock-arb-tabs-group-drag-anchor-right",
                    ));
            let extra_anchors: Vec<fret_core::NodeId> = (0..10)
                .map(|ix| {
                    state
                        .ui
                        .create_node_retained(DockingArbitrationDragAnchor::new(format!(
                            "dock-arb-tab-drag-anchor-extra-{ix}"
                        )))
                })
                .collect();
            let viewport_split_handle_anchor =
                state
                    .ui
                    .create_node_retained(DockingArbitrationDragAnchor::new(
                        "dock-arb-split-handle-viewport",
                    ));
            let floating_title_bar_anchor =
                state
                    .ui
                    .create_node_retained(DockingArbitrationDragAnchor::new(
                        "dock-arb-floating-title-bar-anchor",
                    ));
            let float_zone_anchor =
                state
                    .ui
                    .create_node_retained(DockingArbitrationDragAnchor::new(
                        "dock-arb-float-zone-anchor",
                    ));
            let tab_overflow_button_anchor =
                state
                    .ui
                    .create_node_retained(DockingArbitrationDragAnchor::new(
                        "dock-arb-tab-overflow-button-anchor-left",
                    ));
            let tab_overflow_menu_row_1_anchor =
                state
                    .ui
                    .create_node_retained(DockingArbitrationDragAnchor::new(
                        "dock-arb-tab-overflow-menu-row-anchor-left-1",
                    ));
            let tab_drop_end_anchor =
                state
                    .ui
                    .create_node_retained(DockingArbitrationDragAnchor::new(
                        "dock-arb-tab-drop-end-anchor-left",
                    ));
            let tab_scroll_edge_left_anchor =
                state
                    .ui
                    .create_node_retained(DockingArbitrationDragAnchor::new(
                        "dock-arb-tab-scroll-edge-anchor-left",
                    ));
            let tab_scroll_edge_right_anchor =
                state
                    .ui
                    .create_node_retained(DockingArbitrationDragAnchor::new(
                        "dock-arb-tab-scroll-edge-anchor-right",
                    ));
            let hint_zones = [
                (DropZone::Center, "center"),
                (DropZone::Left, "left"),
                (DropZone::Right, "right"),
                (DropZone::Top, "top"),
                (DropZone::Bottom, "bottom"),
            ];
            let dock_hint_inner_anchors = hint_zones
                .iter()
                .map(|(zone, label)| {
                    let node = state
                        .ui
                        .create_node_retained(DockingArbitrationDragAnchor::new(format!(
                            "dock-arb-hint-inner-{label}"
                        )));
                    (*zone, node)
                })
                .collect::<Vec<_>>();
            let dock_hint_outer_anchors = hint_zones
                .iter()
                .map(|(zone, label)| {
                    let node = state
                        .ui
                        .create_node_retained(DockingArbitrationDragAnchor::new(format!(
                            "dock-arb-hint-outer-{label}"
                        )));
                    (*zone, node)
                })
                .collect::<Vec<_>>();
            let root = state
                .ui
                .create_node_retained(DockingArbitrationHarnessRoot {
                    window,
                    dock_space: *dock_space,
                    left_anchor,
                    right_anchor,
                    right_tabs_group_anchor,
                    extra_anchors: extra_anchors.clone(),
                    float_zone_anchor,
                    viewport_split_handle_anchor,
                    floating_title_bar_anchor,
                    tab_drop_end_anchor,
                    tab_overflow_button_anchor,
                    tab_overflow_menu_row_1_anchor,
                    tab_scroll_edge_left_anchor,
                    tab_scroll_edge_right_anchor,
                    dock_hint_inner_anchors: dock_hint_inner_anchors.clone(),
                    dock_hint_outer_anchors: dock_hint_outer_anchors.clone(),
                });
            state.ui.set_root(root);
            // Ensure the retained harness nodes participate in hit-testing and event routing.
            // Without explicit parent/child wiring, `layout_in` can position nodes for paint, but
            // pointer hit-testing will not descend into them (it only follows the UI tree).
            let children: Vec<fret_core::NodeId> = std::iter::once(*dock_space)
                .chain(std::iter::once(left_anchor))
                .chain(std::iter::once(right_anchor))
                .chain(std::iter::once(right_tabs_group_anchor))
                .chain(extra_anchors.iter().copied())
                .chain(std::iter::once(float_zone_anchor))
                .chain(std::iter::once(viewport_split_handle_anchor))
                .chain(std::iter::once(floating_title_bar_anchor))
                .chain(dock_hint_inner_anchors.iter().map(|(_, node)| *node))
                .chain(dock_hint_outer_anchors.iter().map(|(_, node)| *node))
                .chain(std::iter::once(tab_drop_end_anchor))
                .chain(std::iter::once(tab_overflow_button_anchor))
                .chain(std::iter::once(tab_overflow_menu_row_1_anchor))
                .chain(std::iter::once(tab_scroll_edge_left_anchor))
                .chain(std::iter::once(tab_scroll_edge_right_anchor))
                .collect();
            state.ui.set_children(root, children);
            root
        });

        render_and_bind_dock_panels(&mut state.ui, app, services, window, bounds, *dock_space);

        OverlayController::render(&mut state.ui, app, services, window, bounds);
    }
}

impl WinitAppDriver for DockingArbitrationDriver {
    type WindowState = DockingArbitrationWindowState;

    fn init(&mut self, app: &mut App, main_window: AppWindowId) {
        self.main_window = Some(main_window);
        self.docking_runtime = Some(DockingRuntime::new(main_window));
        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
            svc.set_app_snapshot_provider(Some(Arc::new(|_app, _window| {
                Some(json!({
                    "env": {
                        "FRET_DOCK_ARB_PRESET": std::env::var("FRET_DOCK_ARB_PRESET").ok(),
                        "FRET_DOCK_ALLOW_MULTI_WINDOW_TEAR_OFF": std::env::var("FRET_DOCK_ALLOW_MULTI_WINDOW_TEAR_OFF").ok(),
                        "FRET_DOCK_TEAROFF_TRANSPARENT_PAYLOAD": std::env::var("FRET_DOCK_TEAROFF_TRANSPARENT_PAYLOAD").ok(),
                    }
                }))
            })));
        });
        self.logical_windows
            .insert(main_window, Self::MAIN_LOGICAL_WINDOW_ID.to_string());
        self.sync_dev_state_models(app);
        self.try_restore_layout_on_init(app, main_window);
    }

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn hot_reload_window(
        &mut self,
        app: &mut App,
        _services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        state: &mut Self::WindowState,
    ) {
        crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
        state.root = None;
        state.dock_space = None;
    }

    fn handle_model_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[fret_app::ModelId],
    ) {
        context
            .app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
                svc.record_model_changes(context.window, changed);
            });
        context
            .state
            .ui
            .propagate_model_changes(context.app, changed);
    }

    fn handle_global_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[std::any::TypeId],
    ) {
        context
            .app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
                svc.record_global_changes(app, context.window, changed);
            });
        context
            .state
            .ui
            .propagate_global_changes(context.app, changed);
    }

    fn handle_command(
        &mut self,
        context: WinitCommandContext<'_, Self::WindowState>,
        command: CommandId,
    ) {
        let WinitCommandContext {
            app,
            services,
            state,
            ..
        } = context;
        if state.ui.dispatch_command(app, services, &command) {
            return;
        }
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
        } = context;

        if fret_bootstrap::maybe_consume_event(app, window, event) {
            return;
        }

        if matches!(event, Event::WindowCloseRequested) {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        // Demo-only: synthesize a second pointer stream to validate ADR 0072 multi-pointer
        // arbitration even on hardware without a touch screen / pen.
        //
        // F1 toggles the synth pointer. While enabled:
        // - I/J/K/L move the pointer (logical pixels),
        // - Space presses/releases the pointer (Left button semantics).
        // - B presses/releases the mouse right button at the synth pointer position,
        // - U/O emit mouse wheel up/down at the synth pointer position.
        //
        // These keys are consumed while enabled to keep the demo deterministic.
        let mut dispatch_to_ui = true;
        let mut synth_move_after = false;
        let mut synth_button_after: Option<bool> = None;
        let mut synth_mouse_right_after: Option<bool> = None;
        let mut synth_wheel_after: Option<Point> = None;
        let mut synth_mods = Modifiers::default();

        match event {
            Event::KeyDown {
                key: fret_core::KeyCode::F1,
                repeat: false,
                modifiers,
            } => {
                let synth = self.synth_pointer_mut(app, window);
                synth.enabled = !synth.enabled;
                if !synth.enabled && synth.pressed {
                    // Release deterministically before disabling.
                    synth.pressed = false;
                    synth_button_after = Some(false);
                }
                if !synth.enabled && synth.mouse_right_pressed {
                    synth.mouse_right_pressed = false;
                    synth_mouse_right_after = Some(false);
                }
                synth_mods = *modifiers;
                dispatch_to_ui = false;
                DockingArbitrationDriver::update_synth_debug(app, window, synth);
            }
            Event::KeyDown {
                key:
                    fret_core::KeyCode::KeyI
                    | fret_core::KeyCode::KeyJ
                    | fret_core::KeyCode::KeyK
                    | fret_core::KeyCode::KeyL,
                repeat: false,
                modifiers,
            } => {
                let synth = self.synth_pointer_mut(app, window);
                if synth.enabled {
                    let step = 24.0;
                    match event {
                        Event::KeyDown {
                            key: fret_core::KeyCode::KeyI,
                            ..
                        } => synth.position.y.0 -= step,
                        Event::KeyDown {
                            key: fret_core::KeyCode::KeyK,
                            ..
                        } => synth.position.y.0 += step,
                        Event::KeyDown {
                            key: fret_core::KeyCode::KeyJ,
                            ..
                        } => synth.position.x.0 -= step,
                        Event::KeyDown {
                            key: fret_core::KeyCode::KeyL,
                            ..
                        } => synth.position.x.0 += step,
                        _ => {}
                    }

                    if let Some(metrics) = app.global::<fret_core::WindowMetricsService>()
                        && let Some(size) = metrics.inner_size(window)
                    {
                        synth.position.x.0 = synth.position.x.0.clamp(0.0, size.width.0.max(0.0));
                        synth.position.y.0 = synth.position.y.0.clamp(0.0, size.height.0.max(0.0));
                    }

                    synth_mods = *modifiers;
                    synth_move_after = true;
                    dispatch_to_ui = false;
                    DockingArbitrationDriver::update_synth_debug(app, window, synth);
                }
            }
            Event::KeyDown {
                key: fret_core::KeyCode::Space,
                repeat: false,
                modifiers,
            } => {
                let synth = self.synth_pointer_mut(app, window);
                if synth.enabled && !synth.pressed {
                    synth.pressed = true;
                    synth_button_after = Some(true);
                    synth_mods = *modifiers;
                    dispatch_to_ui = false;
                    DockingArbitrationDriver::update_synth_debug(app, window, synth);
                }
            }
            Event::KeyUp {
                key: fret_core::KeyCode::Space,
                modifiers,
            } => {
                let synth = self.synth_pointer_mut(app, window);
                if synth.enabled && synth.pressed {
                    synth.pressed = false;
                    synth_button_after = Some(false);
                    synth_mods = *modifiers;
                    dispatch_to_ui = false;
                    DockingArbitrationDriver::update_synth_debug(app, window, synth);
                }
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyB,
                repeat: false,
                modifiers,
            } => {
                let synth = self.synth_pointer_mut(app, window);
                if synth.enabled && !synth.mouse_right_pressed {
                    synth.mouse_right_pressed = true;
                    synth_mouse_right_after = Some(true);
                    synth_mods = *modifiers;
                    dispatch_to_ui = false;
                    DockingArbitrationDriver::update_synth_debug(app, window, synth);
                }
            }
            Event::KeyUp {
                key: fret_core::KeyCode::KeyB,
                modifiers,
            } => {
                let synth = self.synth_pointer_mut(app, window);
                if synth.enabled && synth.mouse_right_pressed {
                    synth.mouse_right_pressed = false;
                    synth_mouse_right_after = Some(false);
                    synth_mods = *modifiers;
                    dispatch_to_ui = false;
                    DockingArbitrationDriver::update_synth_debug(app, window, synth);
                }
            }
            Event::KeyDown {
                key: fret_core::KeyCode::KeyU | fret_core::KeyCode::KeyO,
                repeat: false,
                modifiers,
            } => {
                let synth = self.synth_pointer_mut(app, window);
                if synth.enabled {
                    // Positive wheel Y means scrolling up (winit semantics).
                    let delta = match event {
                        Event::KeyDown {
                            key: fret_core::KeyCode::KeyU,
                            ..
                        } => Point::new(Px(0.0), Px(24.0)),
                        Event::KeyDown {
                            key: fret_core::KeyCode::KeyO,
                            ..
                        } => Point::new(Px(0.0), Px(-24.0)),
                        _ => Point::new(Px(0.0), Px(0.0)),
                    };
                    synth_wheel_after = Some(delta);
                    synth_mods = *modifiers;
                    dispatch_to_ui = false;
                }
            }
            _ => {}
        }

        if let Event::KeyDown {
            key: fret_core::KeyCode::KeyQ | fret_core::KeyCode::KeyW | fret_core::KeyCode::KeyE,
            repeat: false,
            ..
        } = event
        {
            let mode = match event {
                Event::KeyDown {
                    key: fret_core::KeyCode::KeyQ,
                    ..
                } => fret_editor::ViewportToolMode::Select,
                Event::KeyDown {
                    key: fret_core::KeyCode::KeyW,
                    ..
                } => fret_editor::ViewportToolMode::Move,
                Event::KeyDown {
                    key: fret_core::KeyCode::KeyE,
                    ..
                } => fret_editor::ViewportToolMode::Rotate,
                _ => return,
            };

            if let Ok(mut tools) = self.viewport_tools.lock() {
                let mut did_change = false;
                for ((w, _target), mgr) in tools.tools.iter_mut() {
                    if *w != window {
                        continue;
                    }
                    if mgr.active != mode {
                        mgr.active = mode;
                        mgr.interaction = None;
                        did_change = true;
                    }
                }
                if did_change {
                    app.request_redraw(window);
                }
            }
        }

        if dispatch_to_ui {
            state.ui.dispatch_event(app, services, event);
        }

        // Inject synth pointer events after we update the synth state, so dock/overlay policies
        // observe the correct final state for this frame.
        if let Some(st) = self.synth_pointers.get(&window).cloned() {
            if synth_move_after {
                DockingArbitrationDriver::dispatch_synth_pointer_move(
                    &mut state.ui,
                    app,
                    services,
                    window,
                    &st,
                    synth_mods,
                );
            }
            if let Some(pressed) = synth_button_after {
                DockingArbitrationDriver::dispatch_synth_pointer_button(
                    &mut state.ui,
                    app,
                    services,
                    window,
                    &st,
                    pressed,
                    synth_mods,
                );
            }
            if let Some(pressed) = synth_mouse_right_after {
                DockingArbitrationDriver::dispatch_synth_mouse_button(
                    &mut state.ui,
                    app,
                    services,
                    window,
                    &st,
                    MouseButton::Right,
                    pressed,
                    synth_mods,
                );
            }
            if let Some(delta) = synth_wheel_after {
                DockingArbitrationDriver::dispatch_synth_mouse_wheel(
                    &mut state.ui,
                    app,
                    services,
                    window,
                    &st,
                    delta,
                    synth_mods,
                );
            }
        }
    }

    fn viewport_input(&mut self, app: &mut App, event: ViewportInputEvent) {
        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
            svc.record_viewport_input(event.clone());
        });

        let cursor_target_px = event
            .cursor_target_px_f32()
            .map(|(x, y)| format!("{x:.1},{y:.1}"))
            .unwrap_or_else(|| "n/a".to_string());
        let target_px_per_screen_px = event.target_px_per_screen_px().unwrap_or(0.0);
        let msg: Arc<str> = Arc::from(
            format!(
                "{:?} cursor_px=({:.1},{:.1}) uv=({:.3},{:.3}) target_px=({}, {}) cursor_target_px=({}) target_px_per_screen_px={:.3} target={:?} window={:?}",
                event.kind,
                event.cursor_px.x.0,
                event.cursor_px.y.0,
                event.uv.0,
                event.uv.1,
                event.target_px.0,
                event.target_px.1,
                cursor_target_px,
                target_px_per_screen_px,
                event.target,
                event.window,
            )
            .into_boxed_str(),
        );
        app.with_global_mut(ViewportDebugService::default, |svc, app| {
            if let Some(model) = svc.last_event.get(&event.window).cloned() {
                let _ = app.models_mut().update(&model, |v| *v = msg.clone());
                app.request_redraw(event.window);
            }
        });

        if let Ok(mut state) = self.viewport_tools.lock() {
            let key = (event.window, event.target);
            let mgr = state.tools.entry(key).or_insert_with(Default::default);
            if mgr.handle_viewport_input(&event) {
                app.request_redraw(event.window);
            }
        }
    }

    fn dock_op(&mut self, app: &mut App, op: fret_core::DockOp) {
        let changed = self
            .docking_runtime
            .as_ref()
            .map(|rt| rt.on_dock_op(app, op))
            .unwrap_or(false);
        if changed {
            self.sync_dev_state_models(app);
            DevStateHooks::export_all(app);
        }
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let WinitRenderContext {
            app,
            services,
            window,
            state,
            bounds,
            scale_factor,
            scene,
        } = context;
        self.render_dock(app, services, window, state, bounds);

        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);

        let inspection_active = app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
                svc.wants_inspection_active(window)
            });
        state.ui.set_inspection_active(inspection_active);

        let diag_enabled = app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| svc.is_enabled());
        if diag_enabled {
            app.with_global_mut_untracked(
                fret_runtime::WindowInteractionDiagnosticsStore::default,
                |store, app| store.begin_frame(window, app.frame_id()),
            );
        }

        scene.clear();
        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();

        let semantics_snapshot = state.ui.semantics_snapshot_arc();
        let drive = app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            svc.drive_script_for_window(
                app,
                window,
                bounds,
                scale_factor,
                Some(&mut state.ui),
                semantics_snapshot.as_deref(),
            )
        });

        for effect in drive.effects {
            app.push_effect(effect);
        }

        if drive.request_redraw {
            app.request_redraw(window);
            app.push_effect(Effect::RequestAnimationFrame(window));
        }

        let mut injected_any = false;
        for event in drive.events {
            injected_any = true;
            state.ui.dispatch_event(app, services, &event);
        }

        if injected_any {
            // Let the runner apply effects (including commands) through the normal effect pipeline.
            // Synchronous command flushing can stall pointer-heavy scripted drags and cause
            // `fretboard diag run` to time out waiting for script progress.
            state.ui.request_semantics_snapshot();
            let mut frame =
                fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
            frame.layout_all();
        }

        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.paint_all(scene);

        if let Some(synth) = self.synth_pointers.get(&window)
            && synth.enabled
        {
            let red = Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            };
            let half = 6.0;
            let rect = Rect::new(
                Point::new(Px(synth.position.x.0 - half), Px(synth.position.y.0 - half)),
                Size::new(Px(half * 2.0), Px(half * 2.0)),
            );
            scene.push(SceneOp::Quad {
                order: DrawOrder(10_000),
                rect,
                background: fret_core::Paint::Solid(if synth.pressed {
                    Color { a: 0.25, ..red }
                } else {
                    Color::TRANSPARENT
                })
                .into(),
                border: Edges::all(Px(2.0)),
                border_paint: fret_core::Paint::Solid(red).into(),

                corner_radii: Corners::all(Px(2.0)),
            });
        }

        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            let element_runtime = app.global::<fret_ui::elements::ElementRuntime>();
            svc.record_snapshot(
                app,
                window,
                bounds,
                scale_factor,
                &mut state.ui,
                element_runtime,
                scene,
            );
            let _ = svc.maybe_dump_if_triggered();
            if svc.is_enabled() {
                app.push_effect(Effect::RequestAnimationFrame(window));
            }
        });
    }

    fn window_create_spec(
        &mut self,
        _app: &mut App,
        request: &fret_app::CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        match &request.kind {
            CreateWindowKind::DockFloating { panel, .. } => Some(WindowCreateSpec::new(
                format!("fret-demo docking_arbitration_demo — {}", panel.kind.0),
                fret_launch::WindowLogicalSize::new(720.0, 520.0),
            )),
            CreateWindowKind::DockRestore { logical_window_id } => {
                let mut size = fret_launch::WindowLogicalSize::new(980.0, 720.0);
                if let Some(restore) = &self.restore
                    && let Some(window) = restore
                        .layout
                        .windows
                        .iter()
                        .find(|w| w.logical_window_id == logical_window_id.as_str())
                    && let Some(p) = &window.placement
                {
                    size = fret_launch::WindowLogicalSize::new(p.width as f64, p.height as f64);
                }
                Some(WindowCreateSpec::new(
                    format!("fret-demo docking_arbitration_demo — {logical_window_id}"),
                    size,
                ))
            }
        }
    }

    fn window_created(
        &mut self,
        app: &mut App,
        request: &fret_app::CreateWindowRequest,
        new_window: AppWindowId,
    ) {
        match &request.kind {
            CreateWindowKind::DockFloating { .. } => {
                let _ = self
                    .docking_runtime
                    .as_ref()
                    .map(|rt| rt.on_window_created(app, request, new_window))
                    .unwrap_or(false);
                let logical = self.alloc_floating_logical_window_id();
                let logical_key = logical.clone();
                self.logical_windows.insert(new_window, logical);
                app.with_global_mut_untracked(DevStateWindowKeyRegistry::default, |reg, _app| {
                    reg.register(new_window, logical_key);
                });
                self.sync_dev_state_models(app);
                DevStateHooks::export_all(app);
            }
            CreateWindowKind::DockRestore { logical_window_id } => {
                self.logical_windows
                    .insert(new_window, logical_window_id.clone());
                app.with_global_mut_untracked(DevStateWindowKeyRegistry::default, |reg, _app| {
                    reg.register(new_window, logical_window_id.clone());
                });
                self.sync_dev_state_models(app);
                if let Some(restore) = self.restore.as_mut() {
                    restore.pending_logical_window_ids.remove(logical_window_id);
                }
                self.apply_layout_if_ready(app);
            }
        }
    }

    fn before_close_window(&mut self, app: &mut App, window: AppWindowId) -> bool {
        if Some(window) == self.main_window {
            if self.persist_layout_on_exit {
                self.save_layout_on_exit(app);
            }
        } else {
            self.logical_windows.remove(&window);
            app.with_global_mut_untracked(DevStateWindowKeyRegistry::default, |reg, _app| {
                reg.unregister(window);
            });
            self.sync_dev_state_models(app);
            DevStateHooks::export_all(app);
        }

        let _ = self
            .docking_runtime
            .as_ref()
            .map(|rt| rt.before_close_window(app, window))
            .unwrap_or(false);
        true
    }

    fn accessibility_snapshot(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
    ) -> Option<Arc<fret_core::SemanticsSnapshot>> {
        state.ui.semantics_snapshot_arc()
    }

    fn accessibility_focus(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
    ) {
        state.ui.set_focus(Some(target));
    }

    fn accessibility_invoke(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
    ) {
        fret_ui_app::accessibility_actions::invoke(&mut state.ui, app, services, target);
    }

    fn accessibility_set_value_text(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: &str,
    ) {
        fret_ui_app::accessibility_actions::set_value_text(
            &mut state.ui,
            app,
            services,
            target,
            value,
        );
    }

    fn accessibility_set_value_numeric(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: f64,
    ) {
        fret_ui_app::accessibility_actions::set_value_numeric(
            &mut state.ui,
            app,
            services,
            target,
            value,
        );
    }

    fn accessibility_set_text_selection(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        anchor: u32,
        focus: u32,
    ) {
        fret_ui_app::accessibility_actions::set_text_selection(
            &mut state.ui,
            app,
            services,
            target,
            anchor,
            focus,
        );
    }

    fn accessibility_replace_selected_text(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: &str,
    ) {
        fret_ui_app::accessibility_actions::replace_selected_text(
            &mut state.ui,
            app,
            services,
            target,
            value,
        );
    }
}

pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    let mut app = App::new();

    let policy_flags = DockingArbitrationPolicyFlags::new();
    app.set_global(policy_flags.clone());
    app.with_global_mut(DockingPolicyService::default, |svc, _app| {
        svc.set(Arc::new(DockingArbitrationDockingPolicy {
            flags: policy_flags,
        }));
    });

    let viewport_tools = Arc::new(Mutex::new(DemoViewportToolState::default()));
    let diag_enabled =
        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| svc.is_enabled());

    // Start from the runner-provided capabilities; diagnostics + docking behavior depends on
    // accurate platform flags (multi-window, tear-off, hover detection quality).
    let mut caps = app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();
    if std::env::var("FRET_SINGLE_WINDOW")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    {
        caps.ui.multi_window = false;
        caps.ui.window_tear_off = true;
    }

    // Optional diagnostics knob: force single-window hover semantics.
    //
    // Note: when window hover detection is enabled, docking can route hover using a runner-owned
    // cursor screen position override (physical pixels). Prefer `set_cursor_in_window_logical` in
    // scripts to avoid DPI-dependent drift when exercising drag previews.
    let disable_cross_window_hover_in_diag =
        std::env::var("FRET_DOCK_ARB_DIAG_DISABLE_CROSS_WINDOW_HOVER")
            .ok()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
    if diag_enabled && disable_cross_window_hover_in_diag {
        caps.ui.window_hover_detection = fret_runtime::WindowHoverDetectionQuality::None;
        caps.ui.window_tear_off = false;
    }
    app.set_global(caps);
    app.with_global_mut(DockPanelRegistryService::<App>::default, |svc, _app| {
        svc.set(Arc::new(DockingArbitrationDockPanelRegistry));
    });
    app.with_global_mut(DockViewportOverlayHooksService::default, |svc, _app| {
        svc.set(Arc::new(DemoViewportOverlayHooks {
            tools: viewport_tools.clone(),
        }));
    });
    app.with_global_mut_untracked(DevStateHooks::default, |hooks, _app| {
        hooks.register(
            DevStateHook::new(DEV_STATE_DOCKING_LAYOUT_KEY, |app| {
                let Some(gate) = app.global::<DockingArbitrationDevStateGate>() else {
                    return DevStateExport::Noop;
                };
                if !gate.export_ready {
                    return DevStateExport::Noop;
                }

                let Some(models) = app.global::<DockingArbitrationDevStateModels>() else {
                    return DevStateExport::Noop;
                };
                if !models.persist_layout_on_exit {
                    return DevStateExport::Noop;
                }
                if models.windows.is_empty() {
                    return DevStateExport::Noop;
                }
                let windows = models.windows.clone();
                let Some(dock) = app.global::<DockManager>() else {
                    return DevStateExport::Noop;
                };
                let Some(metrics) = app.global::<fret_core::WindowMetricsService>() else {
                    return DevStateExport::Noop;
                };

                let placements: HashMap<AppWindowId, fret_core::DockWindowPlacement> = windows
                    .iter()
                    .filter_map(|(window, _logical_window_id)| {
                        let size = metrics.inner_size(*window)?;
                        let width = (size.width.0.max(1.0).round() as u32).max(1);
                        let height = (size.height.0.max(1.0).round() as u32).max(1);
                        Some((
                            *window,
                            fret_core::DockWindowPlacement {
                                width,
                                height,
                                x: None,
                                y: None,
                                monitor_hint: None,
                            },
                        ))
                    })
                    .collect();

                let layout = dock.graph.export_layout_with_placement(&windows, |window| {
                    placements.get(&window).cloned()
                });
                match serde_json::to_value(layout) {
                    Ok(value) => DevStateExport::Set(value),
                    Err(_) => DevStateExport::Noop,
                }
            })
            .with_import(|app, value| {
                let layout = serde_json::from_value(value).map_err(|e| e.to_string())?;
                app.with_global_mut_untracked(
                    DockingArbitrationDevStateIncoming::default,
                    |st, _app| {
                        st.layout = Some(layout);
                    },
                );
                Ok(())
            }),
        );
    });

    let config = WinitRunnerConfig {
        main_window_title: "fret-demo docking_arbitration_demo".to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    let layout_preset = DockingArbitrationLayoutPreset::from_env();
    let no_persist = std::env::var("FRET_DOCK_ARB_NO_PERSIST")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
    let persist_layout_on_exit =
        layout_preset == DockingArbitrationLayoutPreset::Default && !no_persist && !diag_enabled;

    let pending_layout = if layout_preset == DockingArbitrationLayoutPreset::Default
        && !diag_enabled
    {
        fret_app::DockLayoutFileV1::load_json_if_exists(DockingArbitrationDriver::DOCK_LAYOUT_PATH)
            .map(|v| v.map(|f| f.layout))
            .unwrap_or_else(|err| {
                tracing::warn!("failed to load dock layout: {err}");
                None
            })
    } else {
        None
    };

    let driver = DockingArbitrationDriver::new(
        pending_layout,
        viewport_tools,
        layout_preset,
        persist_layout_on_exit,
    );
    fret::run_native_demo(config, app, driver).context("run docking_arbitration_demo app")
}

fn diag_enabled_env() -> bool {
    std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty())
        || std::env::var_os("FRET_DIAG_DIR").is_some_and(|v| !v.is_empty())
}
