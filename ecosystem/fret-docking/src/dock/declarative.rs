use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{AppWindowId, NodeId, PanelKey, Rect, SemanticsRole, Size};
use fret_ui::element::{AnyElement, ManagedSurfaceProps};
use fret_ui::{ElementContext, UiHost};

use super::diagnostics::{
    DockingDiagnosticsExtras, diagnostics_env_enabled, dock_drag_ghost_snapshot_for_window,
    publish_docking_diagnostics_snapshot, should_publish_docking_diagnostics,
};
use super::drop_resolve::{
    DockPanelDropDrag, DockTabsDropDrag, apply_dock_drop_intent,
    compute_dock_drop_resolve_diagnostics, dock_drop_intent_debug_kind,
    dock_drop_target_diagnostics, resolve_dock_drop_intent_panel, resolve_dock_drop_intent_tabs,
    resolve_dock_drop_target,
};
use super::hit_test::{hit_test_split_handle, hit_test_tab};
use super::host_frame::{
    DockSpaceLayoutSnapshot, begin_cross_window_dock_drag, panel_root_placements_for_snapshot,
};
use super::layout::{dock_space_regions, hidden_bounds};
use super::manager::DockManager;
use super::paint::{
    ComplexDropOverlayPaintInput, DockDragGhostPaint, FloatingChromePaintInput,
    SplitHandlePaintInput, TabChromePaintInput, TabDetailPaintInput, ViewportSurfacePaintInput,
    complex_drop_overlay_paint_inputs, paint_basic_drop_overlay, paint_complex_drop_overlay_inputs,
    paint_drag_payload_ghost, paint_drop_hints, paint_floating_chrome_inputs,
    paint_split_handle_inputs, paint_tab_chrome_inputs, paint_tab_detail_inputs,
    paint_tab_insert_preview_title, paint_viewport_surface_inputs, split_handle_paint_inputs,
    tab_chrome_paint_inputs, tab_detail_paint_inputs, viewport_surface_paint_inputs,
};
use super::services::{DockFocusRequestService, DockPanelContentService, DockingPolicyService};
use super::tab_overflow::{
    TabOverflowMenuState, compute_tab_overflow_menu_items, overflow_menu_close_rect,
    overflow_menu_max_scroll, overflow_menu_row_at_pos, overflow_menu_row_count,
    overflow_menu_row_height, overflow_menu_row_rect, tab_overflow_button_rect,
    tab_overflow_menu_rect,
};
use super::types::{
    DividerDragState, DockDragGhostSnapshot, DockDropHints, DockDropTarget, DockPanelDragPayload,
    DockTabsDragPayload, HoverTarget,
};
use super::viewport::{
    ViewportCaptureState, ViewportHit, hit_test_active_viewport_panel, viewport_input_from_hit,
    viewport_input_from_hit_clamped,
};
use fret_runtime::Effect;
use fret_ui_headless::tab_strip_controller as tabstrip_controller;

mod interaction;
mod registry;
mod tab_metrics;

use interaction::{
    DeclarativeDividerDrag, DeclarativeDockInteractionService, DeclarativeFloatingDrag,
    DeclarativeFloatingHover, DeclarativePendingDockDrag, DeclarativePendingDockTabsDrag,
    DeclarativePressedFloatingClose, DeclarativePressedTabClose, DeclarativeTabHover,
};
pub use registry::{
    DockPanelElement, DockPanelElementRegistry, DockPanelElementRegistryService,
    DockSpaceElementOptions, dock_panel_element,
};
use registry::{
    bind_panel_children, collect_panels_for_window, missing_panel_element, panel_nodes_for_window,
};
use tab_metrics::{
    declarative_apply_tab_bar_drag_auto_scroll, declarative_clamp_and_ensure_active_visible,
    declarative_sync_tab_scroll_for_window, declarative_tab_bar_geometry,
    declarative_tab_detail_titles, declarative_tab_scroll_for_frame,
    declarative_tab_widths_for_layout, declarative_tab_widths_from_prepared_titles,
    prepare_declarative_tab_detail_paint, prepare_declarative_tab_title,
};

#[derive(Debug, Clone)]
struct DockSpaceElementFrame {
    paint_panel_bounds: Vec<(PanelKey, Rect)>,
    panel_last_sizes: HashMap<PanelKey, Size>,
    layout_all: HashMap<fret_core::DockNodeId, Rect>,
    hover: Option<super::types::DockDropTarget>,
    drop_hints: Option<DockDropHints>,
    tab_chrome_inputs: Vec<TabChromePaintInput>,
    tab_detail_inputs: Vec<TabDetailPaintInput>,
    tab_widths: HashMap<fret_core::DockNodeId, Arc<[fret_core::Px]>>,
    tab_scroll: HashMap<fret_core::DockNodeId, fret_core::Px>,
    complex_drop_overlay_inputs: Vec<ComplexDropOverlayPaintInput>,
    floating_chrome_inputs: Vec<FloatingChromePaintInput>,
    floating_chrome_nodes: Vec<fret_core::DockNodeId>,
    dock_drag_ghost: Option<DockDragGhostSnapshot>,
    split_handle_inputs: Vec<SplitHandlePaintInput>,
    viewport_surface_inputs: Vec<ViewportSurfacePaintInput>,
    split_handle_gap: fret_core::Px,
    split_handle_hit_thickness: fret_core::Px,
}

impl DockSpaceElementFrame {
    fn empty(panel_last_sizes: HashMap<PanelKey, Size>) -> Self {
        Self {
            paint_panel_bounds: Vec::new(),
            panel_last_sizes,
            layout_all: HashMap::new(),
            hover: None,
            drop_hints: None,
            tab_chrome_inputs: Vec::new(),
            tab_detail_inputs: Vec::new(),
            tab_widths: HashMap::new(),
            tab_scroll: HashMap::new(),
            complex_drop_overlay_inputs: Vec::new(),
            floating_chrome_inputs: Vec::new(),
            floating_chrome_nodes: Vec::new(),
            dock_drag_ghost: None,
            split_handle_inputs: Vec::new(),
            viewport_surface_inputs: Vec::new(),
            split_handle_gap: fret_core::Px(0.0),
            split_handle_hit_thickness: fret_core::Px(0.0),
        }
    }

    fn from_snapshot(
        snapshot: &DockSpaceLayoutSnapshot,
        panel_last_sizes: HashMap<PanelKey, Size>,
        hover: Option<super::types::DockDropTarget>,
        tab_chrome_inputs: Vec<TabChromePaintInput>,
        tab_detail_inputs: Vec<TabDetailPaintInput>,
        tab_widths: HashMap<fret_core::DockNodeId, Arc<[fret_core::Px]>>,
        tab_scroll: HashMap<fret_core::DockNodeId, fret_core::Px>,
        complex_drop_overlay_inputs: Vec<ComplexDropOverlayPaintInput>,
        floating_chrome_inputs: Vec<FloatingChromePaintInput>,
        dock_drag_ghost: Option<DockDragGhostSnapshot>,
        split_handle_inputs: Vec<SplitHandlePaintInput>,
        viewport_surface_inputs: Vec<ViewportSurfacePaintInput>,
    ) -> Self {
        Self {
            paint_panel_bounds: snapshot.paint_panel_bounds.clone(),
            panel_last_sizes,
            layout_all: snapshot.layout_all.clone(),
            drop_hints: drop_hints_from_hover(hover.as_ref()),
            hover,
            tab_chrome_inputs,
            tab_detail_inputs,
            tab_widths,
            tab_scroll,
            complex_drop_overlay_inputs,
            floating_chrome_nodes: snapshot
                .floating_layouts
                .iter()
                .map(|floating| floating.floating.floating)
                .collect(),
            floating_chrome_inputs,
            dock_drag_ghost,
            split_handle_inputs,
            viewport_surface_inputs,
            split_handle_gap: snapshot.split_handle_gap,
            split_handle_hit_thickness: snapshot.split_handle_hit_thickness,
        }
    }
}

fn keep_internal_drag_route_alive<H: UiHost>(app: &mut H, window: AppWindowId, host_node: NodeId) {
    fret_ui::internal_drag::set_route(app, window, fret_runtime::DRAG_KIND_DOCK_PANEL, host_node);
    fret_ui::internal_drag::set_route(app, window, fret_runtime::DRAG_KIND_DOCK_TABS, host_node);
    if app.global::<DockManager>().is_some() {
        app.with_global_mut_untracked(DockManager::default, |dock, _app| {
            dock.register_dock_space_node(window, host_node);
        });
    }
}

fn dock_dragging_affects_window<H: UiHost>(app: &H, window: AppWindowId) -> bool {
    app.any_drag_session(|drag| {
        (drag.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
            || drag.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
            && (drag.source_window == window || drag.current_window == window)
            && drag.dragging
    })
}

fn dock_drag_ghost_for_window<H: UiHost>(
    app: &H,
    window: AppWindowId,
) -> Option<DockDragGhostSnapshot> {
    let pointer_id = app.find_drag_pointer_id(|drag| {
        (drag.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
            || drag.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
            && drag.current_window == window
    })?;
    app.drag(pointer_id)
        .and_then(|drag| dock_drag_ghost_snapshot_for_window(drag, window))
}

fn dock_drag_source_tabs_for_window<H: UiHost>(
    app: &H,
    window: AppWindowId,
) -> Option<fret_core::DockNodeId> {
    let pointer_id = app.find_drag_pointer_id(|drag| {
        (drag.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
            || drag.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
            && (drag.source_window == window || drag.current_window == window)
    })?;
    let drag = app.drag(pointer_id)?;
    if let Some(payload) = drag.payload::<DockTabsDragPayload>() {
        return Some(payload.source_tabs);
    }
    let payload = drag.payload::<DockPanelDragPayload>()?;
    app.global::<DockManager>()?
        .graph
        .find_panel_in_window(drag.source_window, &payload.panel)
        .map(|(tabs, _active)| tabs)
}

fn publish_declarative_docking_diagnostics<H: UiHost>(app: &mut H, window: AppWindowId) {
    if !should_publish_docking_diagnostics(app, diagnostics_env_enabled()) {
        return;
    }
    let frame_id = app.frame_id();
    publish_docking_diagnostics_snapshot(
        app,
        window,
        frame_id,
        DockingDiagnosticsExtras::default(),
    );
}

fn sync_declarative_viewport_layouts<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    snapshot: &DockSpaceLayoutSnapshot,
) {
    let viewport_layouts = snapshot.viewport_layouts.clone();
    app.with_global_mut_untracked(DockManager::default, |dock, _app| {
        dock.sync_viewport_layouts_for_window(window, viewport_layouts);
    });
}

fn declarative_hit_test_tab_close<H: UiHost>(
    app: &H,
    window: AppWindowId,
    bounds: Rect,
    theme: fret_ui::ThemeSnapshot,
    position: fret_core::Point,
) -> Option<(fret_core::DockNodeId, usize, PanelKey)> {
    let dock = app.global::<DockManager>()?;
    let settings = app
        .global::<fret_runtime::DockingInteractionSettings>()
        .copied()
        .unwrap_or_default();
    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let snapshot = DockSpaceLayoutSnapshot::build(
        dock,
        window,
        dock_bounds,
        settings.split_handle_gap,
        settings.split_handle_hit_thickness,
        &HashMap::new(),
    )?;
    let tab_widths =
        declarative_tab_widths_for_layout(app, window, theme.clone(), &snapshot.layout_all);
    let tab_scroll = declarative_tab_scroll_for_frame(
        app,
        window,
        theme.clone(),
        &snapshot.layout_all,
        &tab_widths,
        false,
    );
    hit_test_tab(
        &dock.graph,
        &snapshot.layout_all,
        &tab_scroll,
        &tab_widths,
        theme,
        position,
    )
    .and_then(|(tabs, index, panel, close)| close.then_some((tabs, index, panel)))
}

fn declarative_hit_test_tab_content<H: UiHost>(
    app: &H,
    window: AppWindowId,
    bounds: Rect,
    theme: fret_ui::ThemeSnapshot,
    position: fret_core::Point,
) -> Option<(fret_core::DockNodeId, usize, PanelKey, fret_core::Point)> {
    let dock = app.global::<DockManager>()?;
    let settings = app
        .global::<fret_runtime::DockingInteractionSettings>()
        .copied()
        .unwrap_or_default();
    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let snapshot = DockSpaceLayoutSnapshot::build(
        dock,
        window,
        dock_bounds,
        settings.split_handle_gap,
        settings.split_handle_hit_thickness,
        &HashMap::new(),
    )?;
    let tab_widths =
        declarative_tab_widths_for_layout(app, window, theme.clone(), &snapshot.layout_all);
    let tab_scroll = declarative_tab_scroll_for_frame(
        app,
        window,
        theme.clone(),
        &snapshot.layout_all,
        &tab_widths,
        false,
    );
    let (tabs, index, panel, close) = hit_test_tab(
        &dock.graph,
        &snapshot.layout_all,
        &tab_scroll,
        &tab_widths,
        theme.clone(),
        position,
    )?;
    if close {
        return None;
    }
    let tabs_rect = snapshot.layout_all.get(&tabs).copied()?;
    let (tab_bar, _content) = super::layout::split_tab_bar(tabs_rect);
    let tab_count = match dock.graph.node(tabs) {
        Some(fret_core::DockNode::Tabs { tabs, .. }) => tabs.len(),
        _ => 0,
    };
    let (geom, _overflow) =
        declarative_tab_bar_geometry(theme, &tab_widths, tabs, tab_bar, tab_count);
    let scroll = tab_scroll.get(&tabs).copied().unwrap_or(fret_core::Px(0.0));
    let tab_rect = geom.tab_rect(index, scroll);
    let grab_offset = fret_core::Point::new(
        fret_core::Px((position.x.0 - tab_rect.origin.x.0).max(0.0)),
        fret_core::Px((position.y.0 - tab_rect.origin.y.0).max(0.0)),
    );
    Some((tabs, index, panel, grab_offset))
}

fn declarative_hit_test_tab_bar_empty_space<H: UiHost>(
    app: &H,
    window: AppWindowId,
    bounds: Rect,
    theme: fret_ui::ThemeSnapshot,
    position: fret_core::Point,
) -> Option<(fret_core::DockNodeId, fret_core::Point)> {
    let dock = app.global::<DockManager>()?;
    let snapshot = declarative_layout_snapshot_for_bounds(app, window, bounds)?;
    let tab_widths =
        declarative_tab_widths_for_layout(app, window, theme.clone(), &snapshot.layout_all);
    let tab_scroll = declarative_tab_scroll_for_frame(
        app,
        window,
        theme.clone(),
        &snapshot.layout_all,
        &tab_widths,
        false,
    );
    if hit_test_tab(
        &dock.graph,
        &snapshot.layout_all,
        &tab_scroll,
        &tab_widths,
        theme.clone(),
        position,
    )
    .is_some()
    {
        return None;
    }

    let mut best: Option<(fret_core::DockNodeId, Rect, f32)> = None;
    for (&node, &rect) in &snapshot.layout_all {
        let Some(fret_core::DockNode::Tabs { tabs, .. }) = dock.graph.node(node) else {
            continue;
        };
        if tabs.is_empty() || !rect.contains(position) {
            continue;
        }
        let (tab_bar, _content) = super::layout::split_tab_bar(rect);
        if !tab_bar.contains(position)
            || tab_overflow_button_rect(theme.clone(), tab_bar).contains(position)
        {
            continue;
        }
        let area = rect.size.width.0 * rect.size.height.0;
        match best {
            None => best = Some((node, tab_bar, area)),
            Some((_node, _tab_bar, best_area)) if area < best_area => {
                best = Some((node, tab_bar, area));
            }
            _ => {}
        }
    }

    best.map(|(tabs, tab_bar, _area)| {
        let grab_offset = fret_core::Point::new(
            fret_core::Px((position.x.0 - tab_bar.origin.x.0).max(0.0)),
            fret_core::Px((position.y.0 - tab_bar.origin.y.0).max(0.0)),
        );
        (tabs, grab_offset)
    })
}

fn declarative_layout_snapshot_for_bounds<H: UiHost>(
    app: &H,
    window: AppWindowId,
    bounds: Rect,
) -> Option<DockSpaceLayoutSnapshot> {
    let dock = app.global::<DockManager>()?;
    let settings = app
        .global::<fret_runtime::DockingInteractionSettings>()
        .copied()
        .unwrap_or_default();
    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    DockSpaceLayoutSnapshot::build(
        dock,
        window,
        dock_bounds,
        settings.split_handle_gap,
        settings.split_handle_hit_thickness,
        &HashMap::new(),
    )
}

fn declarative_panel_min_content_size(
    docking_policy: Option<&dyn super::DockingPolicy>,
    dock: &DockManager,
    panel: &PanelKey,
) -> Option<Size> {
    let info = dock.panel(panel);
    if let Some(policy) = docking_policy
        && let Some(size) = policy.panel_min_content_size(panel, info)
    {
        return Some(size);
    }

    info.is_some_and(|panel| panel.viewport.is_some())
        .then(super::default_viewport_min_content_size)
}

fn declarative_node_min_size(
    docking_policy: Option<&dyn super::DockingPolicy>,
    dock: &DockManager,
    node: fret_core::DockNodeId,
    split_handle_gap: fret_core::Px,
) -> Size {
    let Some(node) = dock.graph.node(node) else {
        return Size::new(fret_core::Px(0.0), fret_core::Px(0.0));
    };

    match node {
        fret_core::DockNode::Tabs { tabs, .. } => {
            let mut min_w: f32 = 0.0;
            let mut min_h: f32 = 0.0;
            for panel in tabs {
                let Some(size) = declarative_panel_min_content_size(docking_policy, dock, panel)
                else {
                    continue;
                };
                min_w = min_w.max(size.width.0);
                min_h = min_h.max(size.height.0);
            }
            min_h = min_h.max(0.0) + super::consts::DOCK_TAB_H.0.max(0.0);
            Size::new(fret_core::Px(min_w.max(0.0)), fret_core::Px(min_h.max(0.0)))
        }
        fret_core::DockNode::Floating { child } => {
            declarative_node_min_size(docking_policy, dock, *child, split_handle_gap)
        }
        fret_core::DockNode::Split { axis, children, .. } => {
            if children.is_empty() {
                return Size::new(fret_core::Px(0.0), fret_core::Px(0.0));
            }

            match axis {
                fret_core::Axis::Horizontal => {
                    let mut sum_w: f32 = 0.0;
                    let mut min_h: f32 = 0.0;
                    for child in children {
                        let size = declarative_node_min_size(
                            docking_policy,
                            dock,
                            *child,
                            split_handle_gap,
                        );
                        sum_w += size.width.0.max(0.0);
                        min_h = min_h.max(size.height.0.max(0.0));
                    }
                    sum_w += split_handle_gap.0.max(0.0) * children.len().saturating_sub(1) as f32;
                    Size::new(fret_core::Px(sum_w.max(0.0)), fret_core::Px(min_h.max(0.0)))
                }
                fret_core::Axis::Vertical => {
                    let mut sum_h: f32 = 0.0;
                    let mut min_w: f32 = 0.0;
                    for child in children {
                        let size = declarative_node_min_size(
                            docking_policy,
                            dock,
                            *child,
                            split_handle_gap,
                        );
                        sum_h += size.height.0.max(0.0);
                        min_w = min_w.max(size.width.0.max(0.0));
                    }
                    sum_h += split_handle_gap.0.max(0.0) * children.len().saturating_sub(1) as f32;
                    Size::new(fret_core::Px(min_w.max(0.0)), fret_core::Px(sum_h.max(0.0)))
                }
            }
        }
    }
}

fn declarative_split_child_min_px(
    docking_policy: Option<&dyn super::DockingPolicy>,
    dock: &DockManager,
    split: fret_core::DockNodeId,
    axis: fret_core::Axis,
    split_handle_gap: fret_core::Px,
) -> Vec<fret_core::Px> {
    let Some(fret_core::DockNode::Split { children, .. }) = dock.graph.node(split) else {
        return Vec::new();
    };

    children
        .iter()
        .map(|child| {
            let size = declarative_node_min_size(docking_policy, dock, *child, split_handle_gap);
            let min = match axis {
                fret_core::Axis::Horizontal => size.width.0,
                fret_core::Axis::Vertical => size.height.0,
            };
            fret_core::Px(if min.is_finite() { min.max(0.0) } else { 0.0 })
        })
        .collect()
}

fn declarative_split_handle_hit_for_position<H: UiHost>(
    app: &H,
    snapshot: &DockSpaceLayoutSnapshot,
    position: fret_core::Point,
) -> Option<(DividerDragState, Vec<fret_core::Px>)> {
    let dock = app.global::<DockManager>()?;
    let policy = app
        .global::<DockingPolicyService>()
        .and_then(|service| service.policy());
    let handle = hit_test_split_handle(
        &dock.graph,
        &snapshot.layout_all,
        snapshot.split_handle_gap,
        snapshot.split_handle_hit_thickness,
        position,
        |split, axis, _children| {
            declarative_split_child_min_px(
                policy.as_deref(),
                dock,
                split,
                axis,
                snapshot.split_handle_gap,
            )
        },
    )?;
    let min_px = declarative_split_child_min_px(
        policy.as_deref(),
        dock,
        handle.split,
        handle.axis,
        snapshot.split_handle_gap,
    );
    Some((handle, min_px))
}

fn declarative_split_handle_cursor(axis: fret_core::Axis) -> fret_core::CursorIcon {
    match axis {
        fret_core::Axis::Horizontal => fret_core::CursorIcon::ColResize,
        fret_core::Axis::Vertical => fret_core::CursorIcon::RowResize,
    }
}

fn declarative_pixels_per_point<H: UiHost>(app: &H, window: AppWindowId) -> f32 {
    app.global::<fret_core::WindowMetricsService>()
        .and_then(|svc| svc.scale_factor(window))
        .unwrap_or(1.0)
}

fn declarative_hit_test_active_viewport_panel<H: UiHost>(
    app: &H,
    window: AppWindowId,
    bounds: Rect,
    position: fret_core::Point,
) -> Option<ViewportHit> {
    let dock = app.global::<DockManager>()?;
    let snapshot = declarative_layout_snapshot_for_bounds(app, window, bounds)?;
    hit_test_active_viewport_panel(&dock.graph, &dock.panels, &snapshot.layout_all, position)
}

fn declarative_tab_overflow_menu_for_window<H: UiHost>(
    app: &H,
    window: AppWindowId,
) -> Option<TabOverflowMenuState> {
    app.global::<DeclarativeDockInteractionService>()
        .and_then(|service| service.tab_overflow_menu(window))
}

fn declarative_tab_hover_for_window<H: UiHost>(
    app: &H,
    window: AppWindowId,
) -> DeclarativeTabHover {
    app.global::<DeclarativeDockInteractionService>()
        .map(|service| service.tab_hover(window))
        .unwrap_or_default()
}

fn declarative_floating_hover_for_window<H: UiHost>(
    app: &H,
    window: AppWindowId,
) -> DeclarativeFloatingHover {
    app.global::<DeclarativeDockInteractionService>()
        .map(|service| service.floating_hover(window))
        .unwrap_or_default()
}

fn declarative_dragged_tab_for_drop<H: UiHost>(
    app: &H,
    drag: &fret_runtime::DragSession,
) -> Option<(fret_core::DockNodeId, usize)> {
    let payload = drag.payload::<DockPanelDragPayload>()?;
    app.global::<DockManager>()?
        .graph
        .find_panel_in_window(drag.source_window, &payload.panel)
}

fn declarative_default_floating_rect_for_panel(
    panel: &PanelKey,
    cursor: fret_core::Point,
    tab_grab_offset: fret_core::Point,
    window_bounds: Rect,
    panel_last_sizes: &HashMap<PanelKey, Size>,
) -> Rect {
    let content = panel_last_sizes
        .get(panel)
        .copied()
        .unwrap_or(Size::new(fret_core::Px(360.0), fret_core::Px(240.0)));

    let inner_w = content.width.0.max(160.0);
    let inner_h = (content.height.0 + super::consts::DOCK_TAB_H.0).max(120.0);

    let border = super::consts::DOCK_FLOATING_BORDER.0.max(0.0);
    let title_h = super::consts::DOCK_FLOATING_TITLE_H.0.max(0.0);
    let outer_w = inner_w + border * 2.0;
    let outer_h = inner_h + border * 2.0 + title_h;

    let inner_origin = fret_core::Point::new(
        fret_core::Px(cursor.x.0 - tab_grab_offset.x.0),
        fret_core::Px(cursor.y.0 - tab_grab_offset.y.0),
    );
    let outer_origin = fret_core::Point::new(
        fret_core::Px(inner_origin.x.0 - border),
        fret_core::Px(inner_origin.y.0 - border - title_h),
    );

    clamp_declarative_floating_rect_to_bounds(
        Rect::new(
            outer_origin,
            Size::new(fret_core::Px(outer_w), fret_core::Px(outer_h)),
        ),
        window_bounds,
    )
}

fn declarative_allow_tear_off_for_panel<H: UiHost>(
    app: &H,
    allow_tear_off: bool,
    allow_multi_window_tear_off: bool,
    source_window: AppWindowId,
    panel: &PanelKey,
) -> bool {
    if !allow_tear_off {
        return false;
    }
    let Some(dock) = app.global::<DockManager>() else {
        return false;
    };

    if crate::runtime::is_dock_floating_os_window(app, source_window)
        && dock.graph.collect_panels_in_window(source_window).len() == 1
    {
        return false;
    }

    if dock.graph.windows().len() > 1
        && dock.graph.collect_panels_in_window(source_window).len() == 1
    {
        return false;
    }

    let info = dock.panels.get(panel);
    let policy = app
        .global::<DockingPolicyService>()
        .and_then(|service| service.policy());
    if policy
        .as_deref()
        .is_some_and(|policy| !policy.allow_tear_off(source_window, panel, info))
    {
        return false;
    }

    if dock.graph.windows().len() <= 1 || allow_multi_window_tear_off {
        return true;
    }

    policy
        .as_deref()
        .is_some_and(|policy| policy.allow_multi_window_tear_off(source_window, panel, info))
}

fn declarative_is_outside_bounds_with_margin(
    bounds: Rect,
    position: fret_core::Point,
    margin: fret_core::Px,
) -> bool {
    position.x.0 < bounds.origin.x.0 - margin.0
        || position.y.0 < bounds.origin.y.0 - margin.0
        || position.x.0 > bounds.origin.x.0 + bounds.size.width.0 + margin.0
        || position.y.0 > bounds.origin.y.0 + bounds.size.height.0 + margin.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeclarativeTearOffRetryTarget {
    Panel,
    Tabs,
}

#[derive(Default)]
struct DeclarativeTearOffHoverResult {
    effects: Vec<Effect>,
    requested_tear_off: bool,
}

fn declarative_resolve_tear_off_hover<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    pointer_id: fret_core::PointerId,
    bounds: Rect,
    position: fret_core::Point,
    allow_tear_off: bool,
    allow_multi_window_tear_off: bool,
) -> DeclarativeTearOffHoverResult {
    let now_frame = app.frame_id();
    let now_tick = app.tick_id();
    let Some(drag) = app.drag(pointer_id) else {
        return DeclarativeTearOffHoverResult::default();
    };
    let source_window = drag.source_window;
    let panel_payload = drag.payload::<DockPanelDragPayload>().cloned();
    let tabs_payload = drag.payload::<DockTabsDragPayload>().cloned();
    if panel_payload.is_none() && tabs_payload.is_none() {
        return DeclarativeTearOffHoverResult::default();
    }

    let oob = declarative_is_outside_bounds_with_margin(bounds, position, fret_core::Px(10.0));
    let mut set_tear_off_oob_start_frame: Option<Option<fret_runtime::FrameId>> = None;
    let mut mark_tear_off_requested = false;
    let mut effects = Vec::new();

    if let Some(payload) = panel_payload.as_ref() {
        if allow_tear_off && source_window == window {
            match (oob, payload.tear_off_oob_start_frame) {
                (true, None) => {
                    set_tear_off_oob_start_frame = Some(Some(now_frame));
                }
                (false, Some(_)) => {
                    set_tear_off_oob_start_frame = Some(None);
                }
                _ => {}
            }
        }

        let stable_oob = oob
            && payload
                .tear_off_oob_start_frame
                .is_some_and(|frame| frame != now_frame);
        let disallow_chained_tear_off = app.global::<DockManager>().is_some_and(|dock| {
            dock.graph.windows().len() > 1 && dock.graph.collect_panels_in_window(window).len() == 1
        });
        let allow_panel_tear_off = declarative_allow_tear_off_for_panel(
            app,
            allow_tear_off,
            allow_multi_window_tear_off,
            source_window,
            &payload.panel,
        );
        let requested_tear_off = allow_panel_tear_off
            && source_window == window
            && stable_oob
            && !disallow_chained_tear_off
            && !payload.tear_off_requested;

        if requested_tear_off {
            mark_tear_off_requested = true;
            effects.push(Effect::Dock(
                fret_core::DockOp::RequestFloatPanelToNewWindow {
                    source_window,
                    panel: payload.panel.clone(),
                    anchor: Some(fret_core::WindowAnchor {
                        window,
                        position: payload.grab_offset,
                    }),
                },
            ));
        }
    } else if let Some(payload) = tabs_payload.as_ref() {
        if allow_tear_off && source_window == window {
            match (oob, payload.tear_off_oob_start_frame) {
                (true, None) => {
                    set_tear_off_oob_start_frame = Some(Some(now_frame));
                }
                (false, Some(_)) => {
                    set_tear_off_oob_start_frame = Some(None);
                }
                _ => {}
            }
        }

        let stable_oob = oob
            && payload
                .tear_off_oob_start_frame
                .is_some_and(|frame| frame != now_frame);
        let panel = payload
            .tabs
            .get(payload.active)
            .or_else(|| payload.tabs.first());
        let allow_tabs_tear_off = panel.is_some_and(|panel| {
            declarative_allow_tear_off_for_panel(
                app,
                allow_tear_off,
                allow_multi_window_tear_off,
                source_window,
                panel,
            )
        });
        let requested_tear_off = allow_tabs_tear_off
            && source_window == window
            && stable_oob
            && !payload.tear_off_requested;

        if requested_tear_off && let Some(panel) = panel {
            mark_tear_off_requested = true;
            effects.push(Effect::Dock(
                fret_core::DockOp::RequestFloatTabsToNewWindow {
                    source_window,
                    source_tabs: payload.source_tabs,
                    panel: panel.clone(),
                    anchor: Some(fret_core::WindowAnchor {
                        window,
                        position: payload.grab_offset,
                    }),
                },
            ));
        }
    }

    let retry_target = (!mark_tear_off_requested
        && !bounds.contains(position)
        && source_window == window)
        .then(|| {
            if let Some(payload) = panel_payload.as_ref() {
                let requested_at = payload.tear_off_requested_at_tick?;
                if !payload.tear_off_requested || now_tick.0.saturating_sub(requested_at.0) <= 600 {
                    return None;
                }
                let dock = app.global::<DockManager>()?;
                dock.graph
                    .find_panel_in_window(source_window, &payload.panel)
                    .is_some()
                    .then_some(DeclarativeTearOffRetryTarget::Panel)
            } else if let Some(payload) = tabs_payload.as_ref() {
                let requested_at = payload.tear_off_requested_at_tick?;
                let panel = payload
                    .tabs
                    .get(payload.active)
                    .or_else(|| payload.tabs.first())?;
                if !payload.tear_off_requested || now_tick.0.saturating_sub(requested_at.0) <= 600 {
                    return None;
                }
                let dock = app.global::<DockManager>()?;
                dock.graph
                    .find_panel_in_window(source_window, panel)
                    .is_some()
                    .then_some(DeclarativeTearOffRetryTarget::Tabs)
            } else {
                None
            }
        })
        .flatten();

    if let Some(drag) = app.drag_mut(pointer_id) {
        drag.position = position;
        drag.dragging = true;
        if let Some(payload) = drag.payload_mut::<DockPanelDragPayload>() {
            if retry_target == Some(DeclarativeTearOffRetryTarget::Panel) {
                payload.tear_off_requested = false;
                payload.tear_off_requested_at_tick = None;
                payload.tear_off_oob_start_frame = None;
            }
            if mark_tear_off_requested {
                payload.tear_off_requested = true;
                payload.tear_off_requested_at_tick = Some(now_tick);
                payload.tear_off_oob_start_frame = None;
            }
            if let Some(next) = set_tear_off_oob_start_frame {
                payload.tear_off_oob_start_frame = next;
            }
        } else if let Some(payload) = drag.payload_mut::<DockTabsDragPayload>() {
            if retry_target == Some(DeclarativeTearOffRetryTarget::Tabs) {
                payload.tear_off_requested = false;
                payload.tear_off_requested_at_tick = None;
                payload.tear_off_oob_start_frame = None;
            }
            if mark_tear_off_requested {
                payload.tear_off_requested = true;
                payload.tear_off_requested_at_tick = Some(now_tick);
                payload.tear_off_oob_start_frame = None;
            }
            if let Some(next) = set_tear_off_oob_start_frame {
                payload.tear_off_oob_start_frame = next;
            }
        }
    }

    DeclarativeTearOffHoverResult {
        effects,
        requested_tear_off: mark_tear_off_requested,
    }
}

#[allow(clippy::too_many_arguments)]
fn declarative_resolve_internal_drag_drop<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    pointer_id: fret_core::PointerId,
    bounds: Rect,
    theme: fret_ui::ThemeSnapshot,
    position: fret_core::Point,
    allow_tear_off: bool,
    allow_multi_window_tear_off: bool,
) -> (Vec<Effect>, bool, bool, bool) {
    let Some(drag) = app.drag(pointer_id) else {
        let hover_cleared = app.with_global_mut(DockManager::default, |dock, _app| {
            dock.hover.take().is_some()
        });
        return (Vec::new(), hover_cleared, false, false);
    };

    let dock_previews_enabled = drag
        .payload::<DockPanelDragPayload>()
        .map(|payload| payload.dock_previews_enabled)
        .or_else(|| {
            drag.payload::<DockTabsDragPayload>()
                .map(|payload| payload.dock_previews_enabled)
        })
        .unwrap_or(false);
    let dragging = drag.dragging || drag.source_window != window;
    if !dragging {
        let hover_cleared = app.with_global_mut(DockManager::default, |dock, _app| {
            dock.hover.take().is_some()
        });
        return (Vec::new(), hover_cleared, false, true);
    }

    let source_window = drag.source_window;
    let dragged_tab_for_drop = declarative_dragged_tab_for_drop(app, drag);
    let panel_payload = drag.payload::<DockPanelDragPayload>().cloned();
    let tabs_payload = drag.payload::<DockTabsDragPayload>().cloned();

    let Some(snapshot) = declarative_layout_snapshot_for_bounds(app, window, bounds) else {
        let hover_cleared = app.with_global_mut(DockManager::default, |dock, _app| {
            dock.hover.take().is_some()
        });
        return (Vec::new(), hover_cleared, false, true);
    };
    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let settings = app
        .global::<fret_runtime::DockingInteractionSettings>()
        .copied()
        .unwrap_or_default();
    let font_size = theme.metric_token("font.size");
    let hint_font_size_inner =
        fret_core::Px((font_size.0 * settings.dock_hint_scale_inner.max(0.0)).max(0.0));
    let hint_font_size_outer =
        fret_core::Px((font_size.0 * settings.dock_hint_scale_outer.max(0.0)).max(0.0));
    let tab_widths =
        declarative_tab_widths_for_layout(app, window, theme.clone(), &snapshot.layout_all);
    let tab_scroll = declarative_tab_scroll_for_frame(
        app,
        window,
        theme.clone(),
        &snapshot.layout_all,
        &tab_widths,
        false,
    );
    let policy = app
        .global::<DockingPolicyService>()
        .and_then(|service| service.policy());
    let diagnostics_enabled = should_publish_docking_diagnostics(app, diagnostics_env_enabled());
    let prev_hover = app
        .global::<DockManager>()
        .and_then(|dock| dock.hover.clone());
    let mut candidates = Vec::<fret_runtime::DockDropCandidateRectDiagnostics>::new();
    let graph = &app.global::<DockManager>().expect("dock manager").graph;
    let (target, source) = resolve_dock_drop_target(
        prev_hover,
        !dock_previews_enabled,
        true,
        window,
        policy.as_deref(),
        graph,
        snapshot.root,
        dock_bounds,
        bounds,
        &tab_scroll,
        &tab_widths,
        theme.clone(),
        hint_font_size_inner,
        hint_font_size_outer,
        snapshot.split_handle_gap,
        snapshot.split_handle_hit_thickness,
        position,
        dragged_tab_for_drop,
        diagnostics_enabled.then_some(&mut candidates),
    );

    let panel_last_sizes: HashMap<PanelKey, Size> = snapshot
        .paint_panel_bounds
        .iter()
        .map(|(panel, rect)| (panel.clone(), rect.size))
        .collect();
    let mut effects = Vec::new();
    let mut invalidate_layout = false;
    let intent = if let Some(payload) = panel_payload.as_ref() {
        let allow_panel_tear_off = declarative_allow_tear_off_for_panel(
            app,
            allow_tear_off,
            allow_multi_window_tear_off,
            source_window,
            &payload.panel,
        );
        resolve_dock_drop_intent_panel(
            target.clone(),
            DockPanelDropDrag {
                source_window,
                panel: &payload.panel,
                grab_offset: payload.grab_offset,
                tear_off_requested: payload.tear_off_requested,
            },
            window,
            bounds,
            position,
            allow_panel_tear_off,
            false,
            |panel, position, grab_offset, window_bounds| {
                declarative_default_floating_rect_for_panel(
                    panel,
                    position,
                    grab_offset,
                    window_bounds,
                    &panel_last_sizes,
                )
            },
        )
    } else if let Some(payload) = tabs_payload.as_ref() {
        let panel = payload
            .tabs
            .get(payload.active)
            .or_else(|| payload.tabs.first());
        let allow_tabs_tear_off = panel.is_some_and(|panel| {
            declarative_allow_tear_off_for_panel(
                app,
                allow_tear_off,
                allow_multi_window_tear_off,
                source_window,
                panel,
            )
        });
        resolve_dock_drop_intent_tabs(
            target.clone(),
            DockTabsDropDrag {
                source_window,
                source_tabs: payload.source_tabs,
                tabs: &payload.tabs,
                active: payload.active,
                grab_offset: payload.grab_offset,
                tear_off_requested: payload.tear_off_requested,
            },
            window,
            bounds,
            position,
            allow_tabs_tear_off,
            false,
            |panel, position, grab_offset, window_bounds| {
                declarative_default_floating_rect_for_panel(
                    panel,
                    position,
                    grab_offset,
                    window_bounds,
                    &panel_last_sizes,
                )
            },
        )
    } else {
        super::types::DockDropIntent::None
    };
    apply_dock_drop_intent(intent.clone(), &mut effects, &mut invalidate_layout);

    let (graph_stats, graph_signature, diagnostics) =
        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.hover = None;
            let graph_stats = diagnostics_enabled
                .then(|| super::diagnostics::dock_graph_stats_for_window(&dock.graph, window));
            let graph_signature = diagnostics_enabled
                .then(|| super::diagnostics::dock_graph_signature_for_window(&dock.graph, window));
            let diagnostics = diagnostics_enabled.then(|| {
                compute_dock_drop_resolve_diagnostics(
                    pointer_id,
                    position,
                    bounds,
                    dock_bounds,
                    source,
                    &dock.graph,
                    window,
                    target.as_ref(),
                    candidates,
                )
            });
            (graph_stats, graph_signature, diagnostics)
        });

    let frame_id = app.frame_id();
    if let Some(dock_drop_resolve) = diagnostics {
        app.with_global_mut_untracked(
            fret_runtime::WindowInteractionDiagnosticsStore::default,
            |svc, _app| {
                svc.record_docking(
                    window,
                    frame_id,
                    fret_runtime::DockingInteractionDiagnostics {
                        dock_drop_resolve: Some(dock_drop_resolve),
                        dock_graph_stats: graph_stats,
                        dock_graph_signature: graph_signature,
                        ..Default::default()
                    },
                );
            },
        );
    }
    if std::env::var_os("FRET_DOCK_DRAG_DEBUG").is_some_and(|v| !v.is_empty()) {
        let drop_target_diag = dock_drop_target_diagnostics(target.as_ref());
        tracing::info!(
            window = ?window,
            source_window = ?source_window,
            pointer_id = ?pointer_id,
            pos = ?position,
            invert_docking = !dock_previews_enabled,
            resolve_source = ?source,
            drop_target = ?drop_target_diag,
            intent_kind = dock_drop_intent_debug_kind(&intent),
            "declarative dock drag drop"
        );
    }

    (effects, true, invalidate_layout, true)
}

fn declarative_resolve_internal_drag_hover<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    pointer_id: fret_core::PointerId,
    bounds: Rect,
    theme: fret_ui::ThemeSnapshot,
    position: fret_core::Point,
    allow_tear_off: bool,
    allow_multi_window_tear_off: bool,
) -> (Vec<Effect>, bool, bool) {
    let Some(drag) = app.drag(pointer_id) else {
        let hover_cleared = app.with_global_mut(DockManager::default, |dock, _app| {
            dock.hover.take().is_some()
        });
        return (Vec::new(), hover_cleared, false);
    };
    let dock_previews_enabled = drag
        .payload::<DockPanelDragPayload>()
        .map(|payload| payload.dock_previews_enabled)
        .or_else(|| {
            drag.payload::<DockTabsDragPayload>()
                .map(|payload| payload.dock_previews_enabled)
        })
        .unwrap_or(false);
    let dragging = drag.dragging || drag.source_window != window;
    if !dragging {
        let hover_cleared = app.with_global_mut(DockManager::default, |dock, _app| {
            dock.hover.take().is_some()
        });
        return (Vec::new(), hover_cleared, false);
    }
    let dragged_tab_for_drop = declarative_dragged_tab_for_drop(app, drag);

    let Some(snapshot) = declarative_layout_snapshot_for_bounds(app, window, bounds) else {
        return (Vec::new(), false, false);
    };
    let tear_off = declarative_resolve_tear_off_hover(
        app,
        window,
        pointer_id,
        bounds,
        position,
        allow_tear_off,
        allow_multi_window_tear_off,
    );
    if tear_off.requested_tear_off {
        let _hover_cleared = app.with_global_mut(DockManager::default, |dock, _app| {
            dock.hover.take().is_some()
        });
        return (tear_off.effects, true, true);
    }
    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let settings = app
        .global::<fret_runtime::DockingInteractionSettings>()
        .copied()
        .unwrap_or_default();
    let font_size = theme.metric_token("font.size");
    let hint_font_size_inner =
        fret_core::Px((font_size.0 * settings.dock_hint_scale_inner.max(0.0)).max(0.0));
    let hint_font_size_outer =
        fret_core::Px((font_size.0 * settings.dock_hint_scale_outer.max(0.0)).max(0.0));
    let tab_widths =
        declarative_tab_widths_for_layout(app, window, theme.clone(), &snapshot.layout_all);
    let mut tab_scroll = declarative_tab_scroll_for_frame(
        app,
        window,
        theme.clone(),
        &snapshot.layout_all,
        &tab_widths,
        false,
    );
    let policy = app
        .global::<DockingPolicyService>()
        .and_then(|service| service.policy());
    let diagnostics_enabled = should_publish_docking_diagnostics(app, diagnostics_env_enabled());
    let mut candidates = Vec::<fret_runtime::DockDropCandidateRectDiagnostics>::new();
    let (mut hover, source) = resolve_dock_drop_target(
        None,
        !dock_previews_enabled,
        true,
        window,
        policy.as_deref(),
        &app.global::<DockManager>().expect("dock manager").graph,
        snapshot.root,
        dock_bounds,
        bounds,
        &tab_scroll,
        &tab_widths,
        theme.clone(),
        hint_font_size_inner,
        hint_font_size_outer,
        snapshot.split_handle_gap,
        snapshot.split_handle_hit_thickness,
        position,
        dragged_tab_for_drop,
        diagnostics_enabled.then_some(&mut candidates),
    );
    let mut auto_scrolled = false;
    if let Some(DockDropTarget::Dock(target)) = hover.as_mut() {
        let target_tabs = target.tabs;
        let tabs_len =
            app.global::<DockManager>()
                .and_then(|dock| match dock.graph.node(target_tabs) {
                    Some(fret_core::DockNode::Tabs { tabs, .. }) => Some(tabs.len()),
                    _ => None,
                });
        let tabs_rect = snapshot.layout_all.get(&target_tabs).copied();
        let frame_id = app.frame_id();
        let should_scroll = tabs_len.is_some()
            && tabs_rect.is_some()
            && app.with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| service.should_auto_scroll_tab_drag(window, target_tabs, frame_id),
            );
        if let (true, Some(tabs_len), Some(tabs_rect)) = (should_scroll, tabs_len, tabs_rect) {
            let (tab_bar, _content) = super::layout::split_tab_bar(tabs_rect);
            auto_scrolled = declarative_apply_tab_bar_drag_auto_scroll(
                theme.clone(),
                target,
                tab_bar,
                tabs_len,
                font_size,
                position,
                &tab_widths,
                &mut tab_scroll,
                dragged_tab_for_drop,
            );
        }
    }

    let (changed, graph_stats, graph_signature, diagnostics) =
        app.with_global_mut(DockManager::default, |dock, _app| {
            let changed = dock.hover != hover;
            dock.hover = hover;
            let graph_stats = diagnostics_enabled
                .then(|| super::diagnostics::dock_graph_stats_for_window(&dock.graph, window));
            let graph_signature = diagnostics_enabled
                .then(|| super::diagnostics::dock_graph_signature_for_window(&dock.graph, window));
            let diagnostics = diagnostics_enabled.then(|| {
                compute_dock_drop_resolve_diagnostics(
                    pointer_id,
                    position,
                    bounds,
                    dock_bounds,
                    source,
                    &dock.graph,
                    window,
                    dock.hover.as_ref(),
                    candidates,
                )
            });
            (changed, graph_stats, graph_signature, diagnostics)
        });
    if auto_scrolled {
        declarative_sync_tab_scroll_for_window(
            app,
            window,
            &tab_scroll,
            snapshot.layout_all.keys().copied(),
        );
    }
    let frame_id = app.frame_id();
    if let Some(dock_drop_resolve) = diagnostics {
        app.with_global_mut_untracked(
            fret_runtime::WindowInteractionDiagnosticsStore::default,
            |svc, _app| {
                svc.record_docking(
                    window,
                    frame_id,
                    fret_runtime::DockingInteractionDiagnostics {
                        dock_drop_resolve: Some(dock_drop_resolve),
                        dock_graph_stats: graph_stats,
                        dock_graph_signature: graph_signature,
                        ..Default::default()
                    },
                );
            },
        );
    }
    if std::env::var_os("FRET_DOCK_DRAG_DEBUG").is_some_and(|v| !v.is_empty()) && changed {
        let target = app
            .global::<DockManager>()
            .and_then(|dock| dock_drop_target_diagnostics(dock.hover.as_ref()));
        tracing::info!(
            window = ?window,
            invert_docking = !dock_previews_enabled,
            source = ?source,
            target = ?target,
            "declarative dock drag hover changed"
        );
    }
    (Vec::new(), changed || auto_scrolled, false)
}

fn declarative_panel_drag_allowed<H: UiHost>(
    app: &H,
    window: AppWindowId,
    panel: &PanelKey,
) -> bool {
    let policy = app
        .global::<DockingPolicyService>()
        .and_then(|service| service.policy());
    let info = app
        .global::<DockManager>()
        .and_then(|dock| dock.panel(panel));
    policy
        .as_deref()
        .is_none_or(|policy| policy.allow_panel_drag(window, panel, info))
}

fn declarative_tabs_group_drag_allowed<H: UiHost>(
    app: &H,
    window: AppWindowId,
    tabs: fret_core::DockNodeId,
) -> bool {
    let policy = app
        .global::<DockingPolicyService>()
        .and_then(|service| service.policy());
    policy
        .as_deref()
        .is_none_or(|policy| policy.allow_tabs_group_drag(window, tabs))
}

fn begin_declarative_panel_drag<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    pointer_id: fret_core::PointerId,
    pending: DeclarativePendingDockDrag,
    position: fret_core::Point,
    modifiers: fret_core::Modifiers,
) {
    let settings = app
        .global::<fret_runtime::DockingInteractionSettings>()
        .copied()
        .unwrap_or_default();
    let wants_dock_previews = settings.drag_inversion.wants_dock_previews(modifiers);
    let grab_offset = pending.grab_offset;
    begin_cross_window_dock_drag(
        app,
        pointer_id,
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        pending.start,
        position,
        DockPanelDragPayload {
            panel: pending.panel,
            grab_offset,
            tear_off_requested: false,
            tear_off_requested_at_tick: None,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: wants_dock_previews,
        },
        None,
        grab_offset,
    );
}

fn begin_declarative_tabs_group_drag<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    pointer_id: fret_core::PointerId,
    pending: DeclarativePendingDockTabsDrag,
    position: fret_core::Point,
    modifiers: fret_core::Modifiers,
) {
    let settings = app
        .global::<fret_runtime::DockingInteractionSettings>()
        .copied()
        .unwrap_or_default();
    let wants_dock_previews = settings.drag_inversion.wants_dock_previews(modifiers);
    let (tabs, active) = app
        .global::<DockManager>()
        .and_then(|dock| match dock.graph.node(pending.tabs) {
            Some(fret_core::DockNode::Tabs { tabs, active }) => Some((tabs.clone(), *active)),
            _ => None,
        })
        .unwrap_or_else(|| (Vec::new(), 0));
    let grab_offset = pending.grab_offset;
    begin_cross_window_dock_drag(
        app,
        pointer_id,
        fret_runtime::DRAG_KIND_DOCK_TABS,
        window,
        pending.start,
        position,
        DockTabsDragPayload {
            source_tabs: pending.tabs,
            tabs,
            active,
            grab_offset,
            tear_off_requested: false,
            tear_off_requested_at_tick: None,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: wants_dock_previews,
        },
        None,
        grab_offset,
    );
}

fn declarative_open_tab_overflow_menu<H: UiHost>(
    app: &H,
    window: AppWindowId,
    layout_all: &HashMap<fret_core::DockNodeId, Rect>,
    tab_scroll: &HashMap<fret_core::DockNodeId, fret_core::Px>,
    theme: fret_ui::ThemeSnapshot,
    position: fret_core::Point,
) -> Option<TabOverflowMenuState> {
    let dock = app.global::<DockManager>()?;
    let tab_widths = declarative_tab_widths_for_layout(app, window, theme.clone(), layout_all);
    for (&node_id, &rect) in layout_all {
        let Some(fret_core::DockNode::Tabs { tabs, active }) = dock.graph.node(node_id) else {
            continue;
        };
        if tabs.is_empty() {
            continue;
        }
        let (tab_bar, _content) = super::layout::split_tab_bar(rect);
        if !tab_bar.contains(position) {
            continue;
        }
        let Some(widths) = tab_widths.get(&node_id) else {
            continue;
        };
        let (geom, _overflow) =
            declarative_tab_bar_geometry(theme.clone(), &tab_widths, node_id, tab_bar, tabs.len());
        let max_scroll = geom.max_scroll();
        if max_scroll.0 <= 0.0 {
            continue;
        }
        if !tab_overflow_button_rect(theme.clone(), tab_bar).contains(position) {
            continue;
        }

        let items = compute_tab_overflow_menu_items(
            theme.clone(),
            tab_bar,
            tabs.len(),
            Some(widths),
            tab_scroll
                .get(&node_id)
                .copied()
                .unwrap_or(fret_core::Px(0.0)),
            *active,
        );
        if items.is_empty() {
            continue;
        }
        let item_count = items.len();
        let active_row = items.iter().position(|ix| *ix == *active).unwrap_or(0);
        let row_h = overflow_menu_row_height(tab_bar).0;
        let visible = overflow_menu_row_count(item_count) as f32;
        let active_y = active_row as f32 * row_h;
        let min_scroll = active_y - (visible - 1.0) * row_h;
        let max_scroll_menu = overflow_menu_max_scroll(tab_bar, item_count);
        let scroll = fret_core::Px(min_scroll.clamp(0.0, max_scroll_menu.0.max(0.0)));
        return Some(TabOverflowMenuState {
            tabs: node_id,
            items,
            scroll,
            hovered: None,
        });
    }
    None
}

fn declarative_handle_tab_overflow_menu_left_click<H: UiHost>(
    app: &H,
    window: AppWindowId,
    menu: TabOverflowMenuState,
    layout_all: &HashMap<fret_core::DockNodeId, Rect>,
    theme: fret_ui::ThemeSnapshot,
    position: fret_core::Point,
) -> (bool, Option<TabOverflowMenuState>, Vec<Effect>) {
    let Some(dock) = app.global::<DockManager>() else {
        return (false, None, Vec::new());
    };
    let mut keep_open = true;
    let mut handled = false;
    let mut effects = Vec::new();

    let tabs_rect = layout_all.get(&menu.tabs).copied();
    let node = dock.graph.node(menu.tabs);
    if let (Some(tabs_rect), Some(fret_core::DockNode::Tabs { tabs, .. })) = (tabs_rect, node) {
        let (tab_bar, _content) = super::layout::split_tab_bar(tabs_rect);
        let item_count = menu.items.len();
        let menu_rect = tab_overflow_menu_rect(theme.clone(), tab_bar, item_count);
        let button_rect = tab_overflow_button_rect(theme.clone(), tab_bar);

        if menu_rect.contains(position) {
            handled = true;
            let max_scroll = overflow_menu_max_scroll(tab_bar, item_count);
            let scroll = fret_core::Px(menu.scroll.0.clamp(0.0, max_scroll.0));
            let row = overflow_menu_row_at_pos(menu_rect, tab_bar, item_count, scroll, position);
            if let Some(row) = row
                && let Some(&tab_ix) = menu.items.get(row)
            {
                let row_rect = overflow_menu_row_rect(menu_rect, tab_bar, scroll, row);
                let close_rect = overflow_menu_close_rect(theme.clone(), row_rect);
                let hit = if close_rect.contains(position) {
                    tabstrip_controller::TabStripHitTarget::OverflowMenuRow {
                        index: tab_ix,
                        part: tabstrip_controller::OverflowMenuPart::Close,
                    }
                } else {
                    tabstrip_controller::TabStripHitTarget::OverflowMenuRow {
                        index: tab_ix,
                        part: tabstrip_controller::OverflowMenuPart::Content,
                    }
                };

                match tabstrip_controller::intent_for_click(hit) {
                    tabstrip_controller::TabStripIntent::Close { index } => {
                        if let Some(panel) = tabs.get(index) {
                            effects.push(Effect::Dock(fret_core::DockOp::ClosePanel {
                                window,
                                panel: panel.clone(),
                            }));
                            keep_open = false;
                        }
                    }
                    tabstrip_controller::TabStripIntent::Activate { index, .. } => {
                        effects.push(Effect::Dock(fret_core::DockOp::SetActiveTab {
                            tabs: menu.tabs,
                            active: index,
                        }));
                        keep_open = false;
                    }
                    tabstrip_controller::TabStripIntent::ToggleOverflowMenu
                    | tabstrip_controller::TabStripIntent::None => {}
                }
            }
        } else if button_rect.contains(position) {
            keep_open = false;
            handled = true;
        } else {
            keep_open = false;
        }
    } else {
        keep_open = false;
    }

    let next_menu = keep_open.then_some(menu);
    (handled, next_menu, effects)
}

fn declarative_handle_tab_overflow_menu_wheel<H: UiHost>(
    app: &H,
    menu: TabOverflowMenuState,
    layout_all: &HashMap<fret_core::DockNodeId, Rect>,
    theme: fret_ui::ThemeSnapshot,
    position: fret_core::Point,
    delta: fret_core::Point,
) -> (bool, Option<TabOverflowMenuState>) {
    let Some(dock) = app.global::<DockManager>() else {
        return (false, Some(menu));
    };
    let Some(tabs_rect) = layout_all.get(&menu.tabs).copied() else {
        return (false, Some(menu));
    };
    let Some(fret_core::DockNode::Tabs { .. }) = dock.graph.node(menu.tabs) else {
        return (false, Some(menu));
    };

    let (tab_bar, _content) = super::layout::split_tab_bar(tabs_rect);
    let item_count = menu.items.len();
    if item_count == 0 {
        return (true, None);
    }

    let menu_rect = tab_overflow_menu_rect(theme, tab_bar, item_count);
    if !menu_rect.contains(position) {
        return (false, Some(menu));
    }

    let max_scroll = overflow_menu_max_scroll(tab_bar, item_count);
    let wheel = delta.x.0 + delta.y.0;
    let next_scroll = fret_core::Px((menu.scroll.0 - wheel).clamp(0.0, max_scroll.0));
    let hovered = overflow_menu_row_at_pos(menu_rect, tab_bar, item_count, next_scroll, position);
    let next_menu = TabOverflowMenuState {
        scroll: next_scroll,
        hovered,
        ..menu
    };
    (true, Some(next_menu))
}

fn declarative_handle_tab_strip_wheel<H: UiHost>(
    app: &H,
    window: AppWindowId,
    layout_all: &HashMap<fret_core::DockNodeId, Rect>,
    theme: fret_ui::ThemeSnapshot,
    position: fret_core::Point,
    delta: fret_core::Point,
) -> Option<HashMap<fret_core::DockNodeId, fret_core::Px>> {
    let dock = app.global::<DockManager>()?;
    let tab_widths = declarative_tab_widths_for_layout(app, window, theme.clone(), layout_all);
    let mut tab_scroll = declarative_tab_scroll_for_frame(
        app,
        window,
        theme.clone(),
        layout_all,
        &tab_widths,
        false,
    );

    for (&node_id, &rect) in layout_all {
        let Some(fret_core::DockNode::Tabs { tabs, active }) = dock.graph.node(node_id) else {
            continue;
        };
        if tabs.is_empty() {
            continue;
        }
        let (tab_bar, _content) = super::layout::split_tab_bar(rect);
        if !tab_bar.contains(position) {
            continue;
        }

        declarative_clamp_and_ensure_active_visible(
            &mut tab_scroll,
            &tab_widths,
            theme.clone(),
            node_id,
            tab_bar,
            tabs.len(),
            *active,
        );
        let (geom, _overflow) =
            declarative_tab_bar_geometry(theme.clone(), &tab_widths, node_id, tab_bar, tabs.len());
        let max_scroll = geom.max_scroll();
        if max_scroll.0 <= 0.0 {
            return Some(tab_scroll);
        }

        let wheel = delta.x.0 + delta.y.0;
        let scroll = tab_scroll
            .get(&node_id)
            .copied()
            .unwrap_or(fret_core::Px(0.0));
        let next = fret_core::Px((scroll.0 - wheel).clamp(0.0, max_scroll.0));
        if next.0 <= 0.0 {
            tab_scroll.remove(&node_id);
        } else {
            tab_scroll.insert(node_id, next);
        }
        return Some(tab_scroll);
    }

    None
}

fn declarative_tab_hover_for_position<H: UiHost>(
    app: &H,
    window: AppWindowId,
    layout_all: &HashMap<fret_core::DockNodeId, Rect>,
    theme: fret_ui::ThemeSnapshot,
    position: fret_core::Point,
) -> (DeclarativeTabHover, Option<TabOverflowMenuState>, bool) {
    let Some(dock) = app.global::<DockManager>() else {
        return (DeclarativeTabHover::default(), None, false);
    };
    let tab_widths = declarative_tab_widths_for_layout(app, window, theme.clone(), layout_all);
    let tab_scroll = declarative_tab_scroll_for_frame(
        app,
        window,
        theme.clone(),
        layout_all,
        &tab_widths,
        false,
    );

    let hovered = hit_test_tab(
        &dock.graph,
        layout_all,
        &tab_scroll,
        &tab_widths,
        theme.clone(),
        position,
    )
    .map(|(node, idx, _panel, close)| (node, idx, close));
    let mut hover = DeclarativeTabHover {
        tab: hovered.map(|(node, idx, _close)| (node, idx)),
        tab_close: hovered.map(|(_node, _idx, close)| close).unwrap_or(false),
        overflow_button: None,
    };
    let mut pointer_cursor = hover.tab.is_some();

    for (&node_id, &rect) in layout_all {
        let Some(fret_core::DockNode::Tabs { tabs, .. }) = dock.graph.node(node_id) else {
            continue;
        };
        if tabs.is_empty() {
            continue;
        }
        let (tab_bar, _content) = super::layout::split_tab_bar(rect);
        if !tab_bar.contains(position) {
            continue;
        }
        let (_geom, overflow) =
            declarative_tab_bar_geometry(theme.clone(), &tab_widths, node_id, tab_bar, tabs.len());
        if overflow && tab_overflow_button_rect(theme.clone(), tab_bar).contains(position) {
            hover.overflow_button = Some(node_id);
            pointer_cursor = true;
            break;
        }
    }

    let mut next_menu = declarative_tab_overflow_menu_for_window(app, window);
    if let Some(menu) = next_menu.as_mut() {
        let mut close_menu = false;
        if let Some(&tabs_rect) = layout_all.get(&menu.tabs) {
            if dock.graph.node(menu.tabs).is_some() {
                let (tab_bar, _content) = super::layout::split_tab_bar(tabs_rect);
                let item_count = menu.items.len();
                if item_count == 0 {
                    close_menu = true;
                } else {
                    let menu_rect = tab_overflow_menu_rect(theme.clone(), tab_bar, item_count);
                    menu.hovered = if menu_rect.contains(position) {
                        pointer_cursor = true;
                        overflow_menu_row_at_pos(
                            menu_rect,
                            tab_bar,
                            item_count,
                            menu.scroll,
                            position,
                        )
                    } else {
                        None
                    };
                }
            } else {
                close_menu = true;
            }
        } else {
            close_menu = true;
        }
        if close_menu {
            next_menu = None;
        }
    }

    (hover, next_menu, pointer_cursor)
}

fn apply_declarative_tab_interaction_paint_state(
    frame: &DockSpaceElementFrame,
    hover: DeclarativeTabHover,
    menu: Option<TabOverflowMenuState>,
    tab_chrome_inputs: &mut [TabChromePaintInput],
    tab_detail_inputs: &mut [TabDetailPaintInput],
) {
    for input in tab_chrome_inputs.iter_mut() {
        input.hovered_tab = None;
    }
    for input in tab_detail_inputs.iter_mut() {
        input.hovered_tab = None;
        input.hovered_tab_close = false;
        input.hovered_tab_overflow_button = false;
        input.tab_overflow_menu = None;
    }

    if let Some((tabs, index)) = hover.tab
        && let Some(&rect) = frame.layout_all.get(&tabs)
    {
        for input in tab_chrome_inputs
            .iter_mut()
            .filter(|input| input.rect == rect)
        {
            input.hovered_tab = Some(index);
        }
        for input in tab_detail_inputs
            .iter_mut()
            .filter(|input| input.rect == rect)
        {
            input.hovered_tab = Some(index);
            input.hovered_tab_close = hover.tab_close;
        }
    }

    if let Some(tabs) = hover.overflow_button
        && let Some(&rect) = frame.layout_all.get(&tabs)
    {
        for input in tab_detail_inputs
            .iter_mut()
            .filter(|input| input.rect == rect)
        {
            input.hovered_tab_overflow_button = true;
        }
    }

    if let Some(menu) = menu
        && let Some(&rect) = frame.layout_all.get(&menu.tabs)
        && let Some(input) = tab_detail_inputs
            .iter_mut()
            .find(|input| input.rect == rect)
    {
        input.tab_overflow_menu = Some(menu);
    }
}

fn apply_declarative_floating_hover_paint_state(
    frame: &DockSpaceElementFrame,
    hover: DeclarativeFloatingHover,
    inputs: &mut [FloatingChromePaintInput],
) {
    for (node, input) in frame.floating_chrome_nodes.iter().zip(inputs.iter_mut()) {
        input.title_bar_hovered = hover.title_bar == Some(*node);
        input.close_hovered = hover.close == Some(*node);
    }
}

fn drag_ghost_title<H: UiHost>(app: &H, ghost: &DockDragGhostSnapshot) -> String {
    app.global::<DockManager>()
        .and_then(|dock| dock.panel(&ghost.panel).map(|panel| panel.title.as_str()))
        .filter(|title| !title.is_empty())
        .unwrap_or(ghost.panel.kind.0.as_str())
        .to_string()
}

fn prepare_declarative_drag_ghost(
    services: &mut dyn fret_core::UiServices,
    ghost: &DockDragGhostSnapshot,
    title: &str,
    scale_factor: f32,
) -> DockDragGhostPaint {
    DockDragGhostPaint {
        position: ghost.position,
        grab_offset: ghost.grab_offset,
        title: prepare_declarative_tab_title(services, title, scale_factor),
    }
}

fn declarative_tab_insert_preview_title<H: UiHost>(
    app: &H,
    window: AppWindowId,
    frame: &DockSpaceElementFrame,
) -> Option<(String, Option<fret_core::DockNodeId>, usize)> {
    let Some(DockDropTarget::Dock(target)) = frame.hover.as_ref() else {
        return None;
    };
    if target.zone != fret_core::DropZone::Center {
        return None;
    }
    let ghost = frame.dock_drag_ghost.as_ref()?;
    let drag_source_tabs = dock_drag_source_tabs_for_window(app, window);
    let dock = app.global::<DockManager>()?;
    let title = dock
        .panel(&ghost.panel)
        .map(|panel| panel.title.as_str())
        .filter(|title| !title.is_empty())
        .unwrap_or(ghost.panel.kind.0.as_str())
        .to_string();
    let tab_count = match dock.graph.node(target.tabs) {
        Some(fret_core::DockNode::Tabs { tabs, .. }) => tabs.len(),
        _ => 0,
    };
    Some((title, drag_source_tabs, tab_count))
}

fn floating_chrome_paint_inputs(
    snapshot: &DockSpaceLayoutSnapshot,
    pressed_floating_close: Option<fret_core::DockNodeId>,
    floating_hover: DeclarativeFloatingHover,
) -> Vec<FloatingChromePaintInput> {
    snapshot
        .floating_layouts
        .iter()
        .map(|floating| FloatingChromePaintInput {
            outer: floating.chrome.outer,
            title_bar: floating.chrome.title_bar,
            close_button: floating.chrome.close_button,
            title_bar_hovered: floating_hover.title_bar == Some(floating.floating.floating),
            close_hovered: floating_hover.close == Some(floating.floating.floating),
            close_pressed: pressed_floating_close == Some(floating.floating.floating),
        })
        .collect()
}

fn declarative_pressed_floating_close_for_window<H: UiHost>(
    app: &H,
    window: AppWindowId,
) -> Option<fret_core::DockNodeId> {
    app.global::<DeclarativeDockInteractionService>()
        .and_then(|service| service.pressed_floating_close(window))
}

fn declarative_hit_test_floating_close(
    snapshot: &DockSpaceLayoutSnapshot,
    position: fret_core::Point,
) -> Option<fret_core::DockNodeId> {
    for floating in snapshot.floating_layouts.iter().rev() {
        if !floating.chrome.outer.contains(position) {
            continue;
        }
        return floating
            .chrome
            .close_button
            .contains(position)
            .then_some(floating.floating.floating);
    }
    None
}

fn declarative_hit_test_floating_title_bar(
    snapshot: &DockSpaceLayoutSnapshot,
    position: fret_core::Point,
) -> Option<(fret_core::DockNodeId, fret_core::Point, Rect)> {
    for floating in snapshot.floating_layouts.iter().rev() {
        if !floating.chrome.outer.contains(position) {
            continue;
        }
        if floating.chrome.close_button.contains(position) {
            return None;
        }
        if floating.chrome.title_bar.contains(position) {
            let rect = floating.floating.rect;
            let grab_offset = fret_core::Point::new(
                fret_core::Px(position.x.0 - rect.origin.x.0),
                fret_core::Px(position.y.0 - rect.origin.y.0),
            );
            return Some((floating.floating.floating, grab_offset, rect));
        }
        return None;
    }
    None
}

fn declarative_leaf_tabs_node_at_pos(
    graph: &fret_core::DockGraph,
    layout: &HashMap<fret_core::DockNodeId, Rect>,
    position: fret_core::Point,
) -> Option<(fret_core::DockNodeId, Rect)> {
    let mut best: Option<(fret_core::DockNodeId, Rect, f32)> = None;
    for (&node, &rect) in layout {
        let Some(fret_core::DockNode::Tabs { tabs, .. }) = graph.node(node) else {
            continue;
        };
        if tabs.is_empty() || !rect.contains(position) {
            continue;
        }
        let area = rect.size.width.0 * rect.size.height.0;
        match best {
            None => best = Some((node, rect, area)),
            Some((_node, _rect, best_area)) if area < best_area => {
                best = Some((node, rect, area));
            }
            _ => {}
        }
    }
    best.map(|(node, rect, _area)| (node, rect))
}

fn declarative_resolve_floating_title_bar_drag_target<H: UiHost>(
    app: &H,
    window: AppWindowId,
    bounds: Rect,
    theme: fret_ui::ThemeSnapshot,
    dock_previews_enabled: bool,
    position: fret_core::Point,
) -> Option<DockDropTarget> {
    if !dock_previews_enabled {
        return Some(DockDropTarget::Float { window });
    }
    let dock = app.global::<DockManager>()?;
    let snapshot = declarative_layout_snapshot_for_bounds(app, window, bounds)?;
    let root = snapshot.root?;
    if super::layout::float_zone(snapshot.dock_bounds).contains(position) {
        return Some(DockDropTarget::Float { window });
    }
    if !snapshot.dock_bounds.contains(position) || !bounds.contains(position) {
        return Some(DockDropTarget::Float { window });
    }
    let (tabs, rect) =
        declarative_leaf_tabs_node_at_pos(&dock.graph, &snapshot.root_layout, position)?;

    let font_size = theme.metric_token("font.size");
    let settings = app
        .global::<fret_runtime::DockingInteractionSettings>()
        .copied()
        .unwrap_or_default();
    let hint_font_size_inner =
        fret_core::Px((font_size.0 * settings.dock_hint_scale_inner.max(0.0)).max(0.0));
    let hint_font_size_outer =
        fret_core::Px((font_size.0 * settings.dock_hint_scale_outer.max(0.0)).max(0.0));

    let target = if let Some(root_rect) = snapshot.root_layout.get(&root).copied()
        && root != tabs
        && let Some(zone) =
            super::layout::dock_hint_pick_zone(root_rect, hint_font_size_outer, true, position)
        && zone != fret_core::DropZone::Center
    {
        HoverTarget {
            tabs: root,
            root,
            leaf_tabs: tabs,
            zone,
            insert_index: None,
            outer: true,
            explicit: true,
        }
    } else if let Some(zone) =
        super::layout::dock_hint_pick_zone(rect, hint_font_size_inner, false, position)
    {
        HoverTarget {
            tabs,
            root,
            leaf_tabs: tabs,
            zone,
            insert_index: None,
            outer: false,
            explicit: true,
        }
    } else {
        return None;
    };

    let policy = app
        .global::<DockingPolicyService>()
        .and_then(|service| service.policy());
    if policy.as_deref().is_some_and(|policy| {
        !policy.allow_dock_drop_target(window, target.root, target.tabs, target.zone, target.outer)
    }) {
        return None;
    }
    Some(DockDropTarget::Dock(target))
}

fn clamp_declarative_floating_rect_to_bounds(rect: Rect, bounds: Rect) -> Rect {
    let mut out = rect;
    if bounds.size.width.0 > 0.0 && bounds.size.height.0 > 0.0 {
        let min_x = bounds.origin.x.0;
        let min_y = bounds.origin.y.0;
        let max_x = bounds.origin.x.0 + (bounds.size.width.0 - out.size.width.0).max(0.0);
        let max_y = bounds.origin.y.0 + (bounds.size.height.0 - out.size.height.0).max(0.0);
        out.origin.x = fret_core::Px(out.origin.x.0.clamp(min_x, max_x.max(min_x)));
        out.origin.y = fret_core::Px(out.origin.y.0.clamp(min_y, max_y.max(min_y)));
    }
    out
}

fn drop_hints_from_hover(hover: Option<&super::types::DockDropTarget>) -> Option<DockDropHints> {
    let Some(super::types::DockDropTarget::Dock(target)) = hover else {
        return None;
    };
    Some(DockDropHints {
        root: target.root,
        leaf_tabs: target.leaf_tabs,
    })
}

/// Build a declarative dock-space host from explicit panel roots.
///
/// The host consumes the dock graph and places active panel roots with
/// `DockSpaceLayoutSnapshot`. This is the primary public dock-space entry point when the caller can
/// author panel roots declaratively.
pub fn dock_space_element<H>(
    cx: &mut ElementContext<'_, H>,
    window: AppWindowId,
    options: DockSpaceElementOptions,
    panels: impl IntoIterator<Item = DockPanelElement>,
) -> AnyElement
where
    H: UiHost + 'static,
{
    let panels: Vec<DockPanelElement> = panels.into_iter().collect();
    let panel_keys: Arc<[PanelKey]> = panels
        .iter()
        .map(|panel| panel.panel.clone())
        .collect::<Vec<_>>()
        .into();
    let children: Vec<AnyElement> = panels.into_iter().map(|panel| panel.element).collect();

    let layout_panel_keys = Arc::clone(&panel_keys);
    let layout = options.layout;
    let allow_multi_window_tear_off = options.allow_multi_window_tear_off;
    let mut element = cx.managed_surface_with_prepaint(
        ManagedSurfaceProps {
            layout,
            ..Default::default()
        },
        move |cx| {
            let host_node = cx.node();
            keep_internal_drag_route_alive(cx.app(), window, host_node);
            let children = cx.children().to_vec();
            bind_panel_children(cx.app(), window, &layout_panel_keys, &children);

            let previous_last_sizes = cx
                .output::<DockSpaceElementFrame>()
                .map(|frame| frame.panel_last_sizes.clone())
                .unwrap_or_default();
            let mut panel_last_sizes = previous_last_sizes;

            let settings = cx
                .app()
                .global::<fret_runtime::DockingInteractionSettings>()
                .copied()
                .unwrap_or_default();
            let bounds = cx.bounds();
            let (_chrome, dock_bounds) = dock_space_regions(bounds);
            let snapshot = cx.app().global::<DockManager>().and_then(|dock| {
                DockSpaceLayoutSnapshot::build(
                    dock,
                    window,
                    dock_bounds,
                    settings.split_handle_gap,
                    settings.split_handle_hit_thickness,
                    &HashMap::new(),
                )
            });

            let hidden = hidden_bounds(Size::new(fret_core::Px(0.0), fret_core::Px(0.0)));
            let Some(snapshot) = snapshot else {
                for child in children {
                    let _ = cx.layout_child(child, hidden);
                }
                cx.set_output(DockSpaceElementFrame::empty(panel_last_sizes));
                return;
            };

            let theme = cx.theme().snapshot();
            let tab_widths = declarative_tab_widths_for_layout(
                cx.app(),
                window,
                theme.clone(),
                &snapshot.layout_all,
            );
            let tab_scroll = declarative_tab_scroll_for_frame(
                cx.app(),
                window,
                theme.clone(),
                &snapshot.layout_all,
                &tab_widths,
                true,
            );
            declarative_sync_tab_scroll_for_window(
                cx.app(),
                window,
                &tab_scroll,
                snapshot.layout_all.keys().copied(),
            );
            let tab_overflow_menu = declarative_tab_overflow_menu_for_window(cx.app(), window);
            let tab_hover = declarative_tab_hover_for_window(cx.app(), window);
            let floating_hover = declarative_floating_hover_for_window(cx.app(), window);
            let pressed_floating_close =
                declarative_pressed_floating_close_for_window(cx.app(), window);
            let floating_chrome_inputs =
                floating_chrome_paint_inputs(&snapshot, pressed_floating_close, floating_hover);
            let dock_drag_ghost = dock_drag_ghost_for_window(cx.app(), window);
            let (
                hover,
                tab_chrome_inputs,
                tab_detail_inputs,
                complex_drop_overlay_inputs,
                split_handle_inputs,
                viewport_surface_inputs,
            ) = cx
                .app()
                .global::<DockManager>()
                .map(|dock| {
                    (
                        dock.hover.clone(),
                        tab_chrome_paint_inputs(
                            &dock.graph,
                            &snapshot.layout_all,
                            &tab_widths,
                            &tab_scroll,
                            tab_hover.tab,
                        ),
                        tab_detail_paint_inputs(
                            &dock.graph,
                            &snapshot.layout_all,
                            &tab_widths,
                            &tab_scroll,
                            tab_hover.tab,
                            tab_hover.tab_close,
                            tab_hover.overflow_button,
                            None,
                            tab_overflow_menu.clone(),
                        ),
                        complex_drop_overlay_paint_inputs(
                            theme.clone(),
                            dock.hover.clone(),
                            window,
                            &dock.graph,
                            &snapshot.layout_all,
                            settings.split_handle_gap,
                            settings.split_handle_hit_thickness,
                            &tab_scroll,
                            &tab_widths,
                        ),
                        split_handle_paint_inputs(&dock.graph, &snapshot.layout_all),
                        viewport_surface_paint_inputs(dock, window, &snapshot.layout_all),
                    )
                })
                .unwrap_or_default();
            sync_declarative_viewport_layouts(cx.app(), window, &snapshot);

            let panel_nodes: HashMap<PanelKey, NodeId> = cx
                .app()
                .global::<DockPanelContentService>()
                .map(|content| content.panel_nodes(window).into_iter().collect())
                .unwrap_or_default();
            let mut laid_out = Vec::new();
            for (_panel, node, rect) in
                panel_root_placements_for_snapshot(&snapshot, &panel_nodes, &mut panel_last_sizes)
            {
                let _ = cx.layout_child_root(node, rect);
                laid_out.push(node);
            }

            for child in children {
                if laid_out.contains(&child) {
                    continue;
                }
                let size = panel_keys
                    .iter()
                    .zip(cx.children().iter())
                    .find_map(|(panel, node)| {
                        (*node == child)
                            .then(|| panel_last_sizes.get(panel).copied())
                            .flatten()
                    })
                    .unwrap_or(Size::new(fret_core::Px(0.0), fret_core::Px(0.0)));
                let _ = cx.layout_child(child, hidden_bounds(size));
            }

            cx.set_output(DockSpaceElementFrame::from_snapshot(
                &snapshot,
                panel_last_sizes,
                hover,
                tab_chrome_inputs,
                tab_detail_inputs,
                tab_widths,
                tab_scroll,
                complex_drop_overlay_inputs,
                floating_chrome_inputs,
                dock_drag_ghost,
                split_handle_inputs,
                viewport_surface_inputs,
            ));
        },
        move |cx| {
            let host_node = cx.node();
            keep_internal_drag_route_alive(cx.app(), window, host_node);
            publish_declarative_docking_diagnostics(cx.app(), window);
            if dock_dragging_affects_window(cx.app(), window) {
                cx.request_animation_frame();
            }

            let settings = cx
                .app()
                .global::<fret_runtime::DockingInteractionSettings>()
                .copied()
                .unwrap_or_default();
            let bounds = cx.bounds();
            let (_chrome, dock_bounds) = dock_space_regions(bounds);
            let snapshot = cx.app().global::<DockManager>().and_then(|dock| {
                DockSpaceLayoutSnapshot::build(
                    dock,
                    window,
                    dock_bounds,
                    settings.split_handle_gap,
                    settings.split_handle_hit_thickness,
                    &HashMap::new(),
                )
            });

            let Some(snapshot) = snapshot else {
                cx.set_output(DockSpaceElementFrame::empty(HashMap::new()));
                return;
            };

            let theme = cx.theme().snapshot();
            let tab_widths = declarative_tab_widths_for_layout(
                cx.app(),
                window,
                theme.clone(),
                &snapshot.layout_all,
            );
            let tab_scroll = declarative_tab_scroll_for_frame(
                cx.app(),
                window,
                theme.clone(),
                &snapshot.layout_all,
                &tab_widths,
                false,
            );
            declarative_sync_tab_scroll_for_window(
                cx.app(),
                window,
                &tab_scroll,
                snapshot.layout_all.keys().copied(),
            );
            let tab_overflow_menu = declarative_tab_overflow_menu_for_window(cx.app(), window);
            let tab_hover = declarative_tab_hover_for_window(cx.app(), window);
            let floating_hover = declarative_floating_hover_for_window(cx.app(), window);
            let pressed_floating_close =
                declarative_pressed_floating_close_for_window(cx.app(), window);
            let floating_chrome_inputs =
                floating_chrome_paint_inputs(&snapshot, pressed_floating_close, floating_hover);
            let dock_drag_ghost = dock_drag_ghost_for_window(cx.app(), window);
            let (
                hover,
                tab_chrome_inputs,
                tab_detail_inputs,
                complex_drop_overlay_inputs,
                split_handle_inputs,
                viewport_surface_inputs,
            ) = cx
                .app()
                .global::<DockManager>()
                .map(|dock| {
                    (
                        dock.hover.clone(),
                        tab_chrome_paint_inputs(
                            &dock.graph,
                            &snapshot.layout_all,
                            &tab_widths,
                            &tab_scroll,
                            tab_hover.tab,
                        ),
                        tab_detail_paint_inputs(
                            &dock.graph,
                            &snapshot.layout_all,
                            &tab_widths,
                            &tab_scroll,
                            tab_hover.tab,
                            tab_hover.tab_close,
                            tab_hover.overflow_button,
                            None,
                            tab_overflow_menu.clone(),
                        ),
                        complex_drop_overlay_paint_inputs(
                            theme.clone(),
                            dock.hover.clone(),
                            window,
                            &dock.graph,
                            &snapshot.layout_all,
                            settings.split_handle_gap,
                            settings.split_handle_hit_thickness,
                            &tab_scroll,
                            &tab_widths,
                        ),
                        split_handle_paint_inputs(&dock.graph, &snapshot.layout_all),
                        viewport_surface_paint_inputs(dock, window, &snapshot.layout_all),
                    )
                })
                .unwrap_or_default();
            sync_declarative_viewport_layouts(cx.app(), window, &snapshot);

            let panel_last_sizes = snapshot
                .active_panel_bounds
                .iter()
                .map(|(panel, rect)| (panel.clone(), rect.size))
                .collect();
            cx.set_output(DockSpaceElementFrame::from_snapshot(
                &snapshot,
                panel_last_sizes,
                hover,
                tab_chrome_inputs,
                tab_detail_inputs,
                tab_widths,
                tab_scroll,
                complex_drop_overlay_inputs,
                floating_chrome_inputs,
                dock_drag_ghost,
                split_handle_inputs,
                viewport_surface_inputs,
            ));
        },
        move |cx| {
            let host_node = cx.node();
            keep_internal_drag_route_alive(cx.app(), window, host_node);
            let Some(frame) = cx.output::<DockSpaceElementFrame>().cloned() else {
                return;
            };
            let panel_nodes = panel_nodes_for_window(cx.app(), window);
            let theme = cx.theme().snapshot();
            let scale_factor = cx.scale_factor();
            let overlay_hooks = cx
                .app()
                .global::<super::services::DockViewportOverlayHooksService>()
                .and_then(|svc| svc.hooks());

            let tab_hover = declarative_tab_hover_for_window(cx.app(), window);
            let tab_overflow_menu = declarative_tab_overflow_menu_for_window(cx.app(), window);
            let mut tab_chrome_inputs = frame.tab_chrome_inputs.clone();
            let mut tab_detail_inputs = frame.tab_detail_inputs.clone();
            apply_declarative_tab_interaction_paint_state(
                &frame,
                tab_hover,
                tab_overflow_menu,
                &mut tab_chrome_inputs,
                &mut tab_detail_inputs,
            );
            paint_tab_chrome_inputs(theme.clone(), &tab_chrome_inputs, cx.scene());
            let tab_detail_titles = declarative_tab_detail_titles(cx.app(), &frame);
            let (tab_titles, tab_close_glyph, tab_overflow_glyph) =
                prepare_declarative_tab_detail_paint(
                    tab_detail_titles,
                    cx.services(),
                    scale_factor,
                );
            let measured_tab_widths = declarative_tab_widths_from_prepared_titles(
                cx.app(),
                theme.clone(),
                &frame,
                &tab_titles,
                true,
            );
            cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| {
                    service.set_tab_widths_for_window(window, measured_tab_widths);
                },
            );
            paint_tab_detail_inputs(
                theme.clone(),
                &tab_detail_inputs,
                &tab_titles,
                Some(tab_close_glyph),
                Some(tab_overflow_glyph),
                None,
                None,
                cx.scene(),
            );
            for title in tab_titles.values() {
                cx.release_text_blob_on_next_paint(title.blob);
            }
            cx.release_text_blob_on_next_paint(tab_close_glyph.blob);
            cx.release_text_blob_on_next_paint(tab_overflow_glyph.blob);

            for (panel, rect) in &frame.paint_panel_bounds {
                if let Some(node) = panel_nodes.get(panel).copied() {
                    let bounds = cx.child_bounds(node).unwrap_or(*rect);
                    cx.paint_child(node, bounds);
                }
            }

            paint_viewport_surface_inputs(
                theme.clone(),
                window,
                &frame.viewport_surface_inputs,
                overlay_hooks.as_deref(),
                cx.scene(),
            );

            let floating_hover = declarative_floating_hover_for_window(cx.app(), window);
            let mut floating_chrome_inputs = frame.floating_chrome_inputs.clone();
            apply_declarative_floating_hover_paint_state(
                &frame,
                floating_hover,
                &mut floating_chrome_inputs,
            );
            paint_floating_chrome_inputs(
                theme.clone(),
                &floating_chrome_inputs,
                None,
                None,
                cx.scene(),
            );

            let drag_ghost = frame.dock_drag_ghost.as_ref().map(|ghost| {
                let title = drag_ghost_title(cx.app(), ghost);
                prepare_declarative_drag_ghost(cx.services(), ghost, &title, scale_factor)
            });
            if let Some(drag_ghost) = drag_ghost.as_ref() {
                paint_drag_payload_ghost(theme.clone(), Some(drag_ghost), false, cx.scene());
            }
            if let Some(drag_ghost) = drag_ghost {
                cx.release_text_blob_on_next_paint(drag_ghost.title.blob);
            }

            paint_basic_drop_overlay(
                theme.clone(),
                frame.hover.clone(),
                window,
                cx.bounds(),
                &frame.layout_all,
                None,
                cx.scene(),
            );
            if let Some((title, drag_source_tabs, tab_count)) =
                declarative_tab_insert_preview_title(cx.app(), window, &frame)
            {
                let prepared = prepare_declarative_tab_title(cx.services(), &title, scale_factor);
                paint_tab_insert_preview_title(
                    theme.clone(),
                    frame.hover.clone(),
                    &frame.layout_all,
                    tab_count,
                    &frame.tab_scroll,
                    &frame.tab_widths,
                    drag_source_tabs,
                    Some(&prepared),
                    false,
                    cx.scene(),
                );
                cx.release_text_blob_on_next_paint(prepared.blob);
            }
            paint_complex_drop_overlay_inputs(
                theme.clone(),
                &frame.complex_drop_overlay_inputs,
                cx.scene(),
            );

            let docking_interaction_settings = cx
                .app()
                .global::<fret_runtime::DockingInteractionSettings>()
                .copied()
                .unwrap_or_default();
            let font_size = theme.metric_token("font.size");
            let hint_font_size_inner = fret_core::Px(
                (font_size.0 * docking_interaction_settings.dock_hint_scale_inner.max(0.0))
                    .max(0.0),
            );
            let hint_font_size_outer = fret_core::Px(
                (font_size.0 * docking_interaction_settings.dock_hint_scale_outer.max(0.0))
                    .max(0.0),
            );
            paint_drop_hints(
                theme.clone(),
                frame.drop_hints,
                frame.hover.clone(),
                hint_font_size_inner,
                hint_font_size_outer,
                window,
                cx.bounds(),
                &frame.layout_all,
                cx.scene(),
            );

            paint_split_handle_inputs(
                theme,
                &frame.split_handle_inputs,
                None,
                frame.split_handle_gap,
                frame.split_handle_hit_thickness,
                scale_factor,
                cx.scene(),
            );
        },
        |_cx| children,
    );
    cx.managed_surface_on_command_availability_for(element.id, move |cx, command| {
        if command.as_str() != "dock.focus_requested_panel" {
            return fret_ui::CommandAvailability::NotHandled;
        }

        if cx
            .app()
            .global::<DockFocusRequestService>()
            .is_some_and(|service| service.has(window))
        {
            fret_ui::CommandAvailability::Available
        } else {
            fret_ui::CommandAvailability::NotHandled
        }
    });
    cx.managed_surface_on_command_for(element.id, move |cx, command| {
        if command.as_str() != "dock.focus_requested_panel" {
            return false;
        }

        let Some(panel) = cx.app().with_global_mut(
            DockFocusRequestService::default,
            |service: &mut DockFocusRequestService, _app| service.take(window),
        ) else {
            return false;
        };

        let panel_nodes = panel_nodes_for_window(cx.app(), window);
        if let Some(node) = panel_nodes.get(&panel).copied() {
            cx.request_focus(node);
        } else {
            cx.request_focus(cx.node());
        }
        cx.request_redraw();
        true
    });
    cx.managed_surface_on_event_for(element.id, move |cx, event| match event {
        fret_core::Event::InternalDrag(e)
            if matches!(
                e.kind,
                fret_core::InternalDragKind::Enter | fret_core::InternalDragKind::Over
            ) =>
        {
            let position = cx.pointer_position_window(event).unwrap_or(e.position);
            let bounds = cx.bounds();
            let theme = cx.theme().snapshot();
            let allow_tear_off = cx
                .app()
                .global::<fret_runtime::PlatformCapabilities>()
                .cloned()
                .unwrap_or_default()
                .ui
                .window_tear_off;
            let (effects, changed, invalidate_layout) = declarative_resolve_internal_drag_hover(
                cx.app(),
                window,
                e.pointer_id,
                bounds,
                theme,
                position,
                allow_tear_off,
                allow_multi_window_tear_off,
            );
            for effect in effects {
                cx.push_effect(effect);
            }
            if invalidate_layout {
                cx.invalidate_self(fret_ui::Invalidation::Layout);
            }
            if changed {
                cx.invalidate_self(fret_ui::Invalidation::Paint);
                cx.request_redraw();
            }
        }
        fret_core::Event::InternalDrag(e) if e.kind == fret_core::InternalDragKind::Drop => {
            let position = cx.pointer_position_window(event).unwrap_or(e.position);
            let bounds = cx.bounds();
            let theme = cx.theme().snapshot();
            let (effects, changed, invalidate_layout, end_drag) =
                declarative_resolve_internal_drag_drop(
                    cx.app(),
                    window,
                    e.pointer_id,
                    bounds,
                    theme,
                    position,
                    false,
                    false,
                );
            for effect in effects {
                cx.push_effect(effect);
            }
            if invalidate_layout {
                cx.invalidate_self(fret_ui::Invalidation::Layout);
            }
            if end_drag
                && cx.app().drag(e.pointer_id).is_some_and(|drag| {
                    drag.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                        || drag.kind == fret_runtime::DRAG_KIND_DOCK_TABS
                })
            {
                cx.app().cancel_drag(e.pointer_id);
            }
            if changed {
                cx.invalidate_self(fret_ui::Invalidation::Paint);
                cx.request_redraw();
            }
        }
        fret_core::Event::InternalDrag(e)
            if matches!(
                e.kind,
                fret_core::InternalDragKind::Leave | fret_core::InternalDragKind::Cancel
            ) =>
        {
            let hover_cleared = cx
                .app()
                .with_global_mut(DockManager::default, |dock, _app| {
                    dock.hover.take().is_some()
                });
            if hover_cleared {
                cx.request_redraw();
            }
        }
        fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button,
            modifiers,
            click_count,
            pointer_id,
            pointer_type,
            ..
        }) => {
            let theme = cx.theme().snapshot();
            let bounds = cx.bounds();
            if let Some(snapshot) = declarative_layout_snapshot_for_bounds(cx.app(), window, bounds)
            {
                let menu = declarative_tab_overflow_menu_for_window(cx.app(), window);
                if let Some(menu) = menu {
                    let (handled, next_menu, effects) =
                        declarative_handle_tab_overflow_menu_left_click(
                            cx.app(),
                            window,
                            menu,
                            &snapshot.layout_all,
                            theme.clone(),
                            *position,
                        );
                    cx.app().with_global_mut(
                        DeclarativeDockInteractionService::default,
                        |service, _app| service.set_tab_overflow_menu(window, next_menu),
                    );
                    for effect in effects {
                        cx.push_effect(effect);
                    }
                    cx.request_redraw();
                    if handled {
                        cx.stop_propagation();
                        return;
                    }
                } else {
                    let tab_widths = declarative_tab_widths_for_layout(
                        cx.app(),
                        window,
                        theme.clone(),
                        &snapshot.layout_all,
                    );
                    let tab_scroll = declarative_tab_scroll_for_frame(
                        cx.app(),
                        window,
                        theme.clone(),
                        &snapshot.layout_all,
                        &tab_widths,
                        false,
                    );
                    if let Some(menu) = declarative_open_tab_overflow_menu(
                        cx.app(),
                        window,
                        &snapshot.layout_all,
                        &tab_scroll,
                        theme.clone(),
                        *position,
                    ) {
                        cx.app().with_global_mut(
                            DeclarativeDockInteractionService::default,
                            |service, _app| service.set_tab_overflow_menu(window, Some(menu)),
                        );
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }
                }

                if let Some(floating) = declarative_hit_test_floating_close(&snapshot, *position) {
                    let pressed = DeclarativePressedFloatingClose {
                        pointer_id: *pointer_id,
                        floating,
                    };
                    cx.app().with_global_mut(
                        DeclarativeDockInteractionService::default,
                        |service, _app| service.begin_floating_close(window, pressed),
                    );
                    cx.push_effect(Effect::Dock(fret_core::DockOp::RaiseFloating {
                        window,
                        floating,
                    }));
                    cx.capture_pointer(cx.node());
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
                if let Some((floating, grab_offset, start_rect)) =
                    declarative_hit_test_floating_title_bar(&snapshot, *position)
                    && *button == fret_core::MouseButton::Left
                {
                    let drag = DeclarativeFloatingDrag {
                        pointer_id: *pointer_id,
                        floating,
                        grab_offset,
                        start_rect,
                        start: *position,
                        start_tick: cx.app().tick_id(),
                        activated: false,
                        dock_previews_enabled: false,
                    };
                    cx.app().with_global_mut(
                        DeclarativeDockInteractionService::default,
                        |service, _app| service.begin_floating_drag(window, drag),
                    );
                    cx.push_effect(Effect::Dock(fret_core::DockOp::RaiseFloating {
                        window,
                        floating,
                    }));
                    cx.capture_pointer(cx.node());
                    cx.set_cursor_icon(fret_core::CursorIcon::Default);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if let Some((handle, min_px)) =
                    declarative_split_handle_hit_for_position(cx.app(), &snapshot, *position)
                    && *button == fret_core::MouseButton::Left
                {
                    cx.app().with_global_mut(
                        DeclarativeDockInteractionService::default,
                        |service, _app| {
                            service.begin_divider_drag(
                                window,
                                DeclarativeDividerDrag {
                                    pointer_id: *pointer_id,
                                    handle,
                                    min_px,
                                },
                            );
                        },
                    );
                    cx.capture_pointer(cx.node());
                    cx.set_cursor_icon(declarative_split_handle_cursor(handle.axis));
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if matches!(
                    *button,
                    fret_core::MouseButton::Left
                        | fret_core::MouseButton::Right
                        | fret_core::MouseButton::Middle
                ) && let Some(hit) =
                    declarative_hit_test_active_viewport_panel(cx.app(), window, bounds, *position)
                {
                    let pixels_per_point = declarative_pixels_per_point(cx.app(), window);
                    if let Some(input) = viewport_input_from_hit(
                        window,
                        hit.clone(),
                        pixels_per_point,
                        *pointer_id,
                        *pointer_type,
                        *position,
                        fret_core::ViewportInputKind::PointerDown {
                            button: *button,
                            modifiers: *modifiers,
                            click_count: *click_count,
                        },
                    ) {
                        cx.push_effect(Effect::ViewportInput(input));
                    }
                    cx.app().with_global_mut(
                        DeclarativeDockInteractionService::default,
                        |service, _app| {
                            service.begin_viewport_capture(
                                window,
                                ViewportCaptureState {
                                    pointer_id: *pointer_id,
                                    hit,
                                    button: *button,
                                    start: *position,
                                    last: *position,
                                    moved: false,
                                },
                            );
                        },
                    );
                    cx.capture_pointer(cx.node());
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }
            if let Some((tabs, index, panel)) =
                declarative_hit_test_tab_close(cx.app(), window, bounds, theme.clone(), *position)
                && *button == fret_core::MouseButton::Left
            {
                let pressed = DeclarativePressedTabClose {
                    pointer_id: *pointer_id,
                    tabs,
                    index,
                    panel,
                    start: *position,
                };
                cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.begin_tab_close(window, pressed),
                );
                cx.capture_pointer(cx.node());
                cx.request_redraw();
                cx.stop_propagation();
                return;
            }

            if let Some((_tabs, _index, panel, grab_offset)) =
                declarative_hit_test_tab_content(cx.app(), window, bounds, theme.clone(), *position)
                && *button == fret_core::MouseButton::Left
                && declarative_panel_drag_allowed(cx.app(), window, &panel)
            {
                let pending = DeclarativePendingDockDrag {
                    pointer_id: *pointer_id,
                    start: *position,
                    panel,
                    grab_offset,
                    start_tick: cx.app().tick_id(),
                };
                cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.begin_pending_dock_drag(window, pending),
                );
                cx.capture_pointer(cx.node());
                cx.request_redraw();
                cx.stop_propagation();
                return;
            }

            if let Some((tabs, grab_offset)) =
                declarative_hit_test_tab_bar_empty_space(cx.app(), window, bounds, theme, *position)
                && *button == fret_core::MouseButton::Left
                && declarative_tabs_group_drag_allowed(cx.app(), window, tabs)
            {
                let pending = DeclarativePendingDockTabsDrag {
                    pointer_id: *pointer_id,
                    start: *position,
                    tabs,
                    grab_offset,
                    start_tick: cx.app().tick_id(),
                };
                cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.begin_pending_dock_tabs_drag(window, pending),
                );
                cx.capture_pointer(cx.node());
                cx.request_redraw();
                cx.stop_propagation();
            }
        }
        fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons,
            modifiers,
            pointer_id,
            pointer_type,
            ..
        }) => {
            let viewport_capture = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| service.viewport_capture(window, *pointer_id),
            );
            if let Some(mut capture) = viewport_capture {
                capture.last = *position;
                if !capture.moved && capture.button == fret_core::MouseButton::Right {
                    let settings = cx
                        .app()
                        .global::<fret_runtime::DockingInteractionSettings>()
                        .copied()
                        .unwrap_or_default();
                    let dx = position.x.0 - capture.start.x.0;
                    let dy = position.y.0 - capture.start.y.0;
                    let dist2 = dx * dx + dy * dy;
                    let threshold = settings.viewport_context_menu_drag_threshold.0.max(0.0);
                    capture.moved = dist2 >= threshold * threshold;
                }
                let input = viewport_input_from_hit_clamped(
                    window,
                    capture.hit.clone(),
                    declarative_pixels_per_point(cx.app(), window),
                    *pointer_id,
                    *pointer_type,
                    *position,
                    fret_core::ViewportInputKind::PointerMove {
                        buttons: *buttons,
                        modifiers: *modifiers,
                    },
                );
                cx.push_effect(Effect::ViewportInput(input));
                cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.begin_viewport_capture(window, capture),
                );
                cx.request_redraw();
                cx.stop_propagation();
                return;
            }

            let viewport_capture_exists = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| service.has_viewport_capture_for_window(window),
            );
            if viewport_capture_exists {
                return;
            }

            let divider_drag = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| {
                    if buttons.left {
                        service.divider_drag(window, *pointer_id)
                    } else {
                        service.take_divider_drag(window, *pointer_id)
                    }
                },
            );
            if let Some(divider_drag) = divider_drag {
                if !buttons.left {
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                cx.set_cursor_icon(declarative_split_handle_cursor(divider_drag.handle.axis));
                let settings = cx
                    .app()
                    .global::<fret_runtime::DockingInteractionSettings>()
                    .copied()
                    .unwrap_or_default();
                let changed = cx
                    .app()
                    .with_global_mut(DockManager::default, |dock, _app| {
                        let Some((children_len, fractions_now)) = dock
                            .graph
                            .node(divider_drag.handle.split)
                            .and_then(|node| match node {
                                fret_core::DockNode::Split {
                                    children,
                                    fractions,
                                    ..
                                } => Some((children.len(), fractions.clone())),
                                _ => None,
                            })
                        else {
                            return false;
                        };

                        let Some(next) = super::split_geometry::drag_update_adjacent_fractions(
                            divider_drag.handle.axis,
                            divider_drag.handle.bounds,
                            children_len,
                            &fractions_now,
                            divider_drag.handle.handle_ix,
                            settings.split_handle_gap,
                            settings.split_handle_hit_thickness,
                            &divider_drag.min_px,
                            divider_drag.handle.grab_offset,
                            *position,
                        ) else {
                            return false;
                        };

                        dock.graph
                            .update_split_fractions(divider_drag.handle.split, next)
                    });
                if changed {
                    cx.invalidate_self(fret_ui::Invalidation::Layout);
                    cx.request_redraw();
                }
                cx.stop_propagation();
                return;
            }

            let drag = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| service.take_floating_drag(window, *pointer_id),
            );
            if let Some(mut drag) = drag {
                if !buttons.left {
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if !drag.activated {
                    let settings = cx
                        .app()
                        .global::<fret_runtime::DockingInteractionSettings>()
                        .copied()
                        .unwrap_or_default();
                    let activation = fret_dnd::ActivationConstraint::Distance {
                        px: settings.tab_drag_threshold.0,
                    };
                    if activation.is_satisfied(
                        drag.start_tick.0,
                        cx.app().tick_id().0,
                        drag.start,
                        *position,
                    ) {
                        drag.activated = true;
                        drag.dock_previews_enabled =
                            settings.drag_inversion.wants_dock_previews(*modifiers);
                    }
                }

                let desired = Rect::new(
                    fret_core::Point::new(
                        fret_core::Px(position.x.0 - drag.grab_offset.x.0),
                        fret_core::Px(position.y.0 - drag.grab_offset.y.0),
                    ),
                    drag.start_rect.size,
                );
                let rect = clamp_declarative_floating_rect_to_bounds(desired, cx.bounds());
                cx.push_effect(Effect::Dock(fret_core::DockOp::SetFloatingRect {
                    window,
                    floating: drag.floating,
                    rect,
                }));
                if drag.activated {
                    let bounds = cx.bounds();
                    let theme = cx.theme().snapshot();
                    let hover = declarative_resolve_floating_title_bar_drag_target(
                        cx.app(),
                        window,
                        bounds,
                        theme,
                        drag.dock_previews_enabled,
                        *position,
                    );
                    cx.app()
                        .with_global_mut(DockManager::default, |dock, _app| dock.hover = hover);
                }
                cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.begin_floating_drag(window, drag),
                );
                cx.set_cursor_icon(fret_core::CursorIcon::Default);
                cx.request_redraw();
                cx.stop_propagation();
                return;
            }

            let pending = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| {
                    if buttons.left {
                        service.pending_dock_drag(window, *pointer_id)
                    } else {
                        service.take_pending_dock_drag(window, *pointer_id)
                    }
                },
            );
            if let Some(pending) = pending {
                if !buttons.left {
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                let settings = cx
                    .app()
                    .global::<fret_runtime::DockingInteractionSettings>()
                    .copied()
                    .unwrap_or_default();
                let activation = fret_dnd::ActivationConstraint::Distance {
                    px: settings.tab_drag_threshold.0,
                };
                if activation.is_satisfied(
                    pending.start_tick.0,
                    cx.app().tick_id().0,
                    pending.start,
                    *position,
                ) {
                    let pending = cx.app().with_global_mut(
                        DeclarativeDockInteractionService::default,
                        |service, _app| service.take_pending_dock_drag(window, *pointer_id),
                    );
                    if let Some(pending) = pending {
                        begin_declarative_panel_drag(
                            cx.app(),
                            window,
                            *pointer_id,
                            pending,
                            *position,
                            *modifiers,
                        );
                        cx.app()
                            .with_global_mut(DockManager::default, |dock, _app| dock.hover = None);
                        cx.release_pointer_capture();
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }
                }

                cx.request_redraw();
                cx.stop_propagation();
                return;
            }

            let pending = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| {
                    if buttons.left {
                        service.pending_dock_tabs_drag(window, *pointer_id)
                    } else {
                        service.take_pending_dock_tabs_drag(window, *pointer_id)
                    }
                },
            );
            if let Some(pending) = pending {
                if !buttons.left {
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                let settings = cx
                    .app()
                    .global::<fret_runtime::DockingInteractionSettings>()
                    .copied()
                    .unwrap_or_default();
                let activation = fret_dnd::ActivationConstraint::Distance {
                    px: settings.tab_drag_threshold.0,
                };
                if activation.is_satisfied(
                    pending.start_tick.0,
                    cx.app().tick_id().0,
                    pending.start,
                    *position,
                ) {
                    let pending = cx.app().with_global_mut(
                        DeclarativeDockInteractionService::default,
                        |service, _app| service.take_pending_dock_tabs_drag(window, *pointer_id),
                    );
                    if let Some(pending) = pending {
                        begin_declarative_tabs_group_drag(
                            cx.app(),
                            window,
                            *pointer_id,
                            pending,
                            *position,
                            *modifiers,
                        );
                        cx.app()
                            .with_global_mut(DockManager::default, |dock, _app| dock.hover = None);
                        cx.release_pointer_capture();
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }
                }

                cx.request_redraw();
                cx.stop_propagation();
                return;
            }

            let bounds = cx.bounds();
            let Some(snapshot) = declarative_layout_snapshot_for_bounds(cx.app(), window, bounds)
            else {
                return;
            };
            let split_cursor =
                declarative_split_handle_hit_for_position(cx.app(), &snapshot, *position)
                    .map(|(handle, _min_px)| declarative_split_handle_cursor(handle.axis));
            let floating_close = declarative_hit_test_floating_close(&snapshot, *position);
            let floating_title_bar = declarative_hit_test_floating_title_bar(&snapshot, *position)
                .map(|(floating, _grab_offset, _rect)| floating);
            let floating_hover = DeclarativeFloatingHover {
                close: floating_close,
                title_bar: floating_title_bar,
            };
            let theme = cx.theme().snapshot();
            let (hover, next_menu, pointer_cursor) = declarative_tab_hover_for_position(
                cx.app(),
                window,
                &snapshot.layout_all,
                theme,
                *position,
            );
            let changed = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| {
                    let floating_hover_changed = service.set_floating_hover(window, floating_hover);
                    let hover_changed = service.set_tab_hover(window, hover);
                    let menu_changed = !service.tab_overflow_menu_matches(window, &next_menu);
                    service.set_tab_overflow_menu(window, next_menu);
                    floating_hover_changed || hover_changed || menu_changed
                },
            );
            if let Some(cursor) = split_cursor {
                cx.set_cursor_icon(cursor);
            } else if floating_close.is_some() {
                cx.set_cursor_icon(fret_core::CursorIcon::Pointer);
            } else if floating_title_bar.is_some() {
                cx.set_cursor_icon(fret_core::CursorIcon::Default);
            } else if pointer_cursor {
                cx.set_cursor_icon(fret_core::CursorIcon::Pointer);
            }
            if changed {
                cx.request_redraw();
            }
        }
        fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position, delta, ..
        }) => {
            let bounds = cx.bounds();
            let Some(snapshot) = declarative_layout_snapshot_for_bounds(cx.app(), window, bounds)
            else {
                return;
            };
            let theme = cx.theme().snapshot();
            if let Some(menu) = declarative_tab_overflow_menu_for_window(cx.app(), window) {
                let (handled, next_menu) = declarative_handle_tab_overflow_menu_wheel(
                    cx.app(),
                    menu,
                    &snapshot.layout_all,
                    theme.clone(),
                    *position,
                    *delta,
                );
                if handled {
                    cx.app().with_global_mut(
                        DeclarativeDockInteractionService::default,
                        |service, _app| service.set_tab_overflow_menu(window, next_menu),
                    );
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }
            if let Some(next_scroll) = declarative_handle_tab_strip_wheel(
                cx.app(),
                window,
                &snapshot.layout_all,
                theme,
                *position,
                *delta,
            ) {
                declarative_sync_tab_scroll_for_window(
                    cx.app(),
                    window,
                    &next_scroll,
                    snapshot.layout_all.keys().copied(),
                );
                cx.request_redraw();
                cx.stop_propagation();
            }
        }
        fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button,
            modifiers,
            is_click,
            click_count,
            pointer_id,
            pointer_type,
            ..
        }) => {
            let viewport_capture = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| service.take_viewport_capture(window, *pointer_id),
            );
            if let Some(capture) = viewport_capture {
                if capture.button != *button {
                    cx.app().with_global_mut(
                        DeclarativeDockInteractionService::default,
                        |service, _app| service.begin_viewport_capture(window, capture),
                    );
                    return;
                }
                let suppress_context_menu = cx
                    .app()
                    .global::<fret_runtime::DockingInteractionSettings>()
                    .copied()
                    .unwrap_or_default()
                    .suppress_context_menu_during_viewport_capture
                    && capture.button == fret_core::MouseButton::Right
                    && capture.moved;
                let is_click = if suppress_context_menu {
                    false
                } else {
                    *is_click
                };
                let input = viewport_input_from_hit_clamped(
                    window,
                    capture.hit.clone(),
                    declarative_pixels_per_point(cx.app(), window),
                    *pointer_id,
                    *pointer_type,
                    *position,
                    fret_core::ViewportInputKind::PointerUp {
                        button: *button,
                        modifiers: *modifiers,
                        is_click,
                        click_count: *click_count,
                    },
                );
                cx.push_effect(Effect::ViewportInput(input));
                cx.app()
                    .with_global_mut(DockManager::default, |dock, _app| dock.hover = None);
                cx.release_pointer_capture();
                cx.request_redraw();
                if suppress_context_menu {
                    cx.stop_propagation();
                }
                return;
            }

            if *button == fret_core::MouseButton::Left {
                let pressed_floating = cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.take_floating_close(window, *pointer_id),
                );
                if let Some(pressed) = pressed_floating {
                    let bounds = cx.bounds();
                    let clicked =
                        declarative_layout_snapshot_for_bounds(cx.app(), window, bounds).and_then(
                            |snapshot| declarative_hit_test_floating_close(&snapshot, *position),
                        ) == Some(pressed.floating);
                    if clicked
                        && let Some(target_tabs) = cx
                            .app()
                            .global::<DockManager>()
                            .and_then(|dock| dock.graph.first_tabs_in_window(window))
                    {
                        cx.push_effect(Effect::Dock(fret_core::DockOp::MergeFloatingInto {
                            window,
                            floating: pressed.floating,
                            target_tabs,
                        }));
                    }
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }

            if *button == fret_core::MouseButton::Left {
                let floating_drag = cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.take_floating_drag(window, *pointer_id),
                );
                if let Some(drag) = floating_drag {
                    let bounds = cx.bounds();
                    let theme = cx.theme().snapshot();
                    if drag.activated
                        && drag.dock_previews_enabled
                        && let Some(DockDropTarget::Dock(target)) =
                            declarative_resolve_floating_title_bar_drag_target(
                                cx.app(),
                                window,
                                bounds,
                                theme,
                                drag.dock_previews_enabled,
                                *position,
                            )
                        && matches!(target.zone, fret_core::DropZone::Center)
                    {
                        cx.push_effect(Effect::Dock(fret_core::DockOp::MergeFloatingInto {
                            window,
                            floating: drag.floating,
                            target_tabs: target.tabs,
                        }));
                    }
                    cx.app()
                        .with_global_mut(DockManager::default, |dock, _app| dock.hover = None);
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }

            if *button == fret_core::MouseButton::Left {
                let divider_drag = cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.take_divider_drag(window, *pointer_id),
                );
                if let Some(divider_drag) = divider_drag {
                    let updates = cx
                        .app()
                        .with_global_mut(DockManager::default, |dock, _app| {
                            dock.graph
                                .node(divider_drag.handle.split)
                                .and_then(|node| match node {
                                    fret_core::DockNode::Split {
                                        children,
                                        fractions,
                                        ..
                                    } if children.len() >= 2
                                        && children.len() == fractions.len() =>
                                    {
                                        Some(vec![fret_core::SplitFractionsUpdate {
                                            split: divider_drag.handle.split,
                                            fractions: fractions.clone(),
                                        }])
                                    }
                                    _ => None,
                                })
                                .unwrap_or_default()
                        });
                    if !updates.is_empty() {
                        cx.push_effect(Effect::Dock(fret_core::DockOp::SetSplitFractionsMany {
                            updates,
                        }));
                    }
                    cx.release_pointer_capture();
                    cx.invalidate_self(fret_ui::Invalidation::Layout);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }

            if *button == fret_core::MouseButton::Left {
                let pending_drag = cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.take_pending_dock_drag(window, *pointer_id),
                );
                if pending_drag.is_some() {
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }

            if *button == fret_core::MouseButton::Left {
                let pending_tabs_drag = cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.take_pending_dock_tabs_drag(window, *pointer_id),
                );
                if pending_tabs_drag.is_some() {
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }

            let pressed = if *button == fret_core::MouseButton::Left {
                cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.take_tab_close(window, *pointer_id),
                )
            } else {
                None
            };
            let Some(pressed) = pressed else {
                return;
            };
            let theme = cx.theme().snapshot();
            let bounds = cx.bounds();
            let clicked =
                declarative_hit_test_tab_close(cx.app(), window, bounds, theme, *position)
                    .is_some_and(|(tabs, index, panel)| {
                        tabs == pressed.tabs && index == pressed.index && panel == pressed.panel
                    });
            let within_slop = fret_ui_headless::tab_strip_hit_test::pointer_move_within_slop(
                pressed.start,
                *position,
                super::consts::DOCK_TAB_CLOSE_CLICK_SLOP,
            );
            if clicked || within_slop {
                cx.push_effect(Effect::Dock(fret_core::DockOp::ClosePanel {
                    window,
                    panel: pressed.panel,
                }));
            }
            cx.release_pointer_capture();
            cx.request_redraw();
            cx.stop_propagation();
        }
        fret_core::Event::PointerCancel(cancel) => {
            let viewport_capture = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| service.take_viewport_capture(window, cancel.pointer_id),
            );
            if let Some(capture) = viewport_capture {
                let position = cancel.position.unwrap_or(capture.last);
                let input = viewport_input_from_hit_clamped(
                    window,
                    capture.hit,
                    declarative_pixels_per_point(cx.app(), window),
                    cancel.pointer_id,
                    cancel.pointer_type,
                    position,
                    fret_core::ViewportInputKind::PointerCancel {
                        buttons: cancel.buttons,
                        modifiers: cancel.modifiers,
                        reason: cancel.reason,
                    },
                );
                cx.push_effect(Effect::ViewportInput(input));
                cx.app()
                    .with_global_mut(DockManager::default, |dock, _app| dock.hover = None);
                cx.release_pointer_capture();
                cx.request_redraw();
                cx.stop_propagation();
                return;
            }

            let cleared_tab = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| {
                    service.take_tab_close(window, cancel.pointer_id).is_some()
                        || service
                            .take_pending_dock_drag(window, cancel.pointer_id)
                            .is_some()
                        || service
                            .take_pending_dock_tabs_drag(window, cancel.pointer_id)
                            .is_some()
                        || service
                            .take_divider_drag(window, cancel.pointer_id)
                            .is_some()
                },
            );
            let cleared_floating = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| {
                    service
                        .take_floating_close(window, cancel.pointer_id)
                        .is_some()
                        || service
                            .take_floating_drag(window, cancel.pointer_id)
                            .is_some()
                },
            );
            if cleared_tab || cleared_floating {
                cx.release_pointer_capture();
                cx.request_redraw();
                cx.stop_propagation();
            }
        }
        _ => {}
    });

    let mut semantics = fret_ui::element::SemanticsDecoration::default().role(SemanticsRole::Panel);
    if let Some(test_id) = options.test_id {
        semantics = semantics.test_id(test_id);
    }
    element = element.attach_semantics(semantics);
    element
}

/// Build a declarative dock-space host from the installed declarative panel registry.
///
/// Non-viewport panels without a registered declarative element receive the same generic missing-UI
/// placeholder as the retained registry path. Pure viewport panels may omit an element root.
pub fn dock_space_element_from_registry<H>(
    cx: &mut ElementContext<'_, H>,
    window: AppWindowId,
    options: DockSpaceElementOptions,
) -> AnyElement
where
    H: UiHost + 'static,
{
    let registry = cx
        .app
        .global::<DockPanelElementRegistryService<H>>()
        .and_then(|service| service.registry());
    let panels = collect_panels_for_window(cx.app, window);
    let mut elements = Vec::new();

    for (panel, is_viewport_panel) in panels {
        let element = registry
            .as_ref()
            .and_then(|registry| registry.render_panel(cx, window, &panel));
        let element = match (is_viewport_panel, element) {
            (_, Some(element)) => element,
            (true, None) => continue,
            (false, None) => missing_panel_element(cx, &panel),
        };
        elements.push(DockPanelElement::new(panel, element));
    }

    dock_space_element(cx, window, options, elements)
}

/// Mount a declarative dock-space host into an immediate-style writer.
#[cfg(feature = "imui")]
pub fn imui_dock_space_element<H: UiHost + 'static>(
    ui: &mut impl fret_authoring::UiWriter<H>,
    options: DockSpaceElementOptions,
) {
    let window = ui.with_cx_mut(|cx| cx.window);
    ui.mount(|cx| vec![dock_space_element_from_registry(cx, window, options)]);
}
