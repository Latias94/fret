use std::collections::HashMap;

use fret_core::{AppWindowId, PanelKey, Rect, Size};
use fret_ui::UiHost;

use super::super::hit_test::{hit_test_split_handle, hit_test_tab};
use super::super::host_frame::DockSpaceLayoutSnapshot;
use super::super::layout::{dock_space_regions, split_tab_bar};
use super::super::manager::DockManager;
use super::super::services::DockingPolicyService;
use super::super::tab_overflow::tab_overflow_button_rect;
use super::super::types::DividerDragState;
use super::super::viewport::{ViewportHit, hit_test_active_viewport_panel};
use super::tab_metrics::{
    declarative_tab_bar_geometry, declarative_tab_scroll_for_frame,
    declarative_tab_widths_for_layout,
};

pub(super) fn declarative_hit_test_tab_close<H: UiHost>(
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
        &dock.workspace.graph,
        &snapshot.layout_all,
        &tab_scroll,
        &tab_widths,
        theme,
        position,
    )
    .and_then(|(tabs, index, panel, close)| close.then_some((tabs, index, panel)))
}

pub(super) fn declarative_hit_test_tab_content<H: UiHost>(
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
        &dock.workspace.graph,
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
    let (tab_bar, _content) = split_tab_bar(tabs_rect);
    let tab_count = match dock.workspace.graph.node(tabs) {
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

pub(super) fn declarative_hit_test_tab_bar_empty_space<H: UiHost>(
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
        &dock.workspace.graph,
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
        let Some(fret_core::DockNode::Tabs { tabs, .. }) = dock.workspace.graph.node(node) else {
            continue;
        };
        if tabs.is_empty() || !rect.contains(position) {
            continue;
        }
        let (tab_bar, _content) = split_tab_bar(rect);
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

pub(super) fn declarative_layout_snapshot_for_bounds<H: UiHost>(
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
    docking_policy: Option<&dyn super::super::DockingPolicy>,
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
        .then(super::super::default_viewport_min_content_size)
}

fn declarative_node_min_size(
    docking_policy: Option<&dyn super::super::DockingPolicy>,
    dock: &DockManager,
    node: fret_core::DockNodeId,
    split_handle_gap: fret_core::Px,
) -> Size {
    let Some(node) = dock.workspace.graph.node(node) else {
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
            min_h = min_h.max(0.0) + super::super::consts::DOCK_TAB_H.0.max(0.0);
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
    docking_policy: Option<&dyn super::super::DockingPolicy>,
    dock: &DockManager,
    split: fret_core::DockNodeId,
    axis: fret_core::Axis,
    split_handle_gap: fret_core::Px,
) -> Vec<fret_core::Px> {
    let Some(fret_core::DockNode::Split { children, .. }) = dock.workspace.graph.node(split) else {
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

pub(super) fn declarative_split_handle_hit_for_position<H: UiHost>(
    app: &H,
    snapshot: &DockSpaceLayoutSnapshot,
    position: fret_core::Point,
) -> Option<(DividerDragState, Vec<fret_core::Px>)> {
    let dock = app.global::<DockManager>()?;
    let policy = app
        .global::<DockingPolicyService>()
        .and_then(|service| service.policy());
    let handle = hit_test_split_handle(
        &dock.workspace.graph,
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

pub(super) fn declarative_split_handle_cursor(axis: fret_core::Axis) -> fret_core::CursorIcon {
    match axis {
        fret_core::Axis::Horizontal => fret_core::CursorIcon::ColResize,
        fret_core::Axis::Vertical => fret_core::CursorIcon::RowResize,
    }
}

pub(super) fn declarative_pixels_per_point<H: UiHost>(app: &H, window: AppWindowId) -> f32 {
    app.global::<fret_core::WindowMetricsService>()
        .and_then(|svc| svc.scale_factor(window))
        .unwrap_or(1.0)
}

pub(super) fn declarative_hit_test_active_viewport_panel<H: UiHost>(
    app: &H,
    window: AppWindowId,
    bounds: Rect,
    position: fret_core::Point,
) -> Option<ViewportHit> {
    let dock = app.global::<DockManager>()?;
    let snapshot = declarative_layout_snapshot_for_bounds(app, window, bounds)?;
    hit_test_active_viewport_panel(
        &dock.workspace.graph,
        dock.panels(),
        &snapshot.layout_all,
        position,
    )
}
