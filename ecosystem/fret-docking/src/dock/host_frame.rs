// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::consts::{DOCK_FLOATING_BORDER, DOCK_FLOATING_CLOSE_SIZE, DOCK_FLOATING_TITLE_H};
use super::layout::{
    compute_layout_map_with_split_fractions_overrides, hidden_bounds, split_tab_bar,
};
use super::manager::DockManager;
use fret_core::{
    DockGraph, DockNode, DockNodeId, NodeId, PanelKey, Point, Px, Rect, RenderTargetId, Size,
    ViewportMapping,
};
use fret_runtime::DragKindId;
use fret_ui::UiHost;

pub(super) fn begin_cross_window_dock_drag<H: UiHost, T: std::any::Any>(
    app: &mut H,
    pointer_id: fret_core::PointerId,
    kind: DragKindId,
    source_window: fret_core::AppWindowId,
    start: Point,
    position: Point,
    payload: T,
    follow_window: Option<fret_core::AppWindowId>,
    grab_offset: Point,
) {
    fret_runtime::DragHost::begin_cross_window_drag_with_kind(
        app,
        pointer_id,
        kind,
        source_window,
        start,
        payload,
    );
    if let Some(drag) = fret_runtime::DragHost::drag_mut(app, pointer_id) {
        drag.follow_window = follow_window;
        drag.position = position;
        drag.dragging = true;
        drag.cursor_grab_offset = Some(grab_offset);
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct FloatingChrome {
    pub(super) outer: Rect,
    pub(super) title_bar: Rect,
    pub(super) close_button: Rect,
    pub(super) inner: Rect,
}

pub(super) fn floating_chrome(outer: Rect) -> FloatingChrome {
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

#[derive(Clone)]
pub(super) struct DockSpaceFloatingLayoutSnapshot {
    pub(super) floating: fret_core::DockFloatingWindow,
    pub(super) chrome: FloatingChrome,
    pub(super) layout: HashMap<DockNodeId, Rect>,
}

#[derive(Clone)]
pub(super) struct DockSpaceLayoutSnapshot {
    pub(super) dock_bounds: Rect,
    pub(super) split_handle_gap: Px,
    pub(super) split_handle_hit_thickness: Px,
    pub(super) root: Option<DockNodeId>,
    pub(super) root_layout: HashMap<DockNodeId, Rect>,
    pub(super) floating_layouts: Vec<DockSpaceFloatingLayoutSnapshot>,
    pub(super) layout_all: HashMap<DockNodeId, Rect>,
    pub(super) active_panel_bounds: HashMap<PanelKey, Rect>,
    pub(super) paint_panel_bounds: Vec<(PanelKey, Rect)>,
    pub(super) viewport_layouts: Vec<(RenderTargetId, super::DockViewportLayout)>,
}

fn push_active_panel_content_bounds_in_graph_order(
    graph: &DockGraph,
    node: DockNodeId,
    layout: &HashMap<DockNodeId, Rect>,
    visited: &mut HashSet<DockNodeId>,
    out: &mut Vec<(PanelKey, Rect)>,
) {
    if !visited.insert(node) {
        return;
    }

    let Some(dock_node) = graph.node(node) else {
        return;
    };
    match dock_node {
        DockNode::Tabs { tabs, active } => {
            let Some(&rect) = layout.get(&node) else {
                return;
            };
            let (_tab_bar, content) = split_tab_bar(rect);
            if let Some(panel) = tabs.get(*active) {
                out.push((panel.clone(), content));
            }
        }
        DockNode::Split { children, .. } => {
            for &child in children {
                push_active_panel_content_bounds_in_graph_order(graph, child, layout, visited, out);
            }
        }
        DockNode::Floating { child } => {
            push_active_panel_content_bounds_in_graph_order(graph, *child, layout, visited, out);
        }
    }
}

fn active_panel_content_bounds_in_graph_order(
    graph: &DockGraph,
    root: Option<DockNodeId>,
    root_layout: &HashMap<DockNodeId, Rect>,
    floating_layouts: &[DockSpaceFloatingLayoutSnapshot],
) -> Vec<(PanelKey, Rect)> {
    let mut out = Vec::new();
    if let Some(root) = root {
        push_active_panel_content_bounds_in_graph_order(
            graph,
            root,
            root_layout,
            &mut HashSet::new(),
            &mut out,
        );
    }
    for floating in floating_layouts {
        push_active_panel_content_bounds_in_graph_order(
            graph,
            floating.floating.floating,
            &floating.layout,
            &mut HashSet::new(),
            &mut out,
        );
    }
    out
}

impl DockSpaceLayoutSnapshot {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn build(
        dock: &DockManager,
        window: fret_core::AppWindowId,
        dock_bounds: Rect,
        split_handle_gap: Px,
        split_handle_hit_thickness: Px,
        split_overrides: &HashMap<DockNodeId, Arc<[f32]>>,
    ) -> Option<Self> {
        let root = dock.workspace.graph.window_root(window);
        let has_floatings = !dock.workspace.graph.floating_windows(window).is_empty();
        if root.is_none() && !has_floatings {
            return None;
        }

        let root_layout = root
            .map(|root| {
                compute_layout_map_with_split_fractions_overrides(
                    &dock.workspace.graph,
                    root,
                    dock_bounds,
                    split_handle_gap,
                    split_handle_hit_thickness,
                    split_overrides,
                )
            })
            .unwrap_or_default();

        let mut layout_all = root_layout.clone();
        let mut floating_layouts = Vec::new();
        for floating in dock.workspace.graph.floating_windows(window) {
            let chrome = floating_chrome(floating.rect);
            let layout = compute_layout_map_with_split_fractions_overrides(
                &dock.workspace.graph,
                floating.floating,
                chrome.inner,
                split_handle_gap,
                split_handle_hit_thickness,
                split_overrides,
            );
            for (&node, &rect) in &layout {
                layout_all.insert(node, rect);
            }
            floating_layouts.push(DockSpaceFloatingLayoutSnapshot {
                floating: *floating,
                chrome,
                layout,
            });
        }

        let mut active_panel_bounds = HashMap::new();
        let paint_panel_bounds = active_panel_content_bounds_in_graph_order(
            &dock.workspace.graph,
            root,
            &root_layout,
            &floating_layouts,
        );
        for (panel, rect) in &paint_panel_bounds {
            active_panel_bounds.insert(panel.clone(), *rect);
        }

        let mut viewport_layouts = Vec::new();
        for (&node_id, &rect) in &layout_all {
            let (_tab_bar, content) = split_tab_bar(rect);
            let viewport = (|| {
                let DockNode::Tabs { tabs, active } = dock.workspace.graph.node(node_id)?.clone()
                else {
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
                viewport_layouts.push((
                    viewport.target,
                    super::DockViewportLayout {
                        content_rect: content,
                        mapping,
                        draw_rect: mapping.map().draw_rect,
                    },
                ));
            }
        }

        Some(Self {
            dock_bounds,
            split_handle_gap,
            split_handle_hit_thickness,
            root,
            root_layout,
            floating_layouts,
            layout_all,
            active_panel_bounds,
            paint_panel_bounds,
            viewport_layouts,
        })
    }
}

pub(super) fn panel_root_placements_for_snapshot(
    snapshot: &DockSpaceLayoutSnapshot,
    panel_nodes: &HashMap<PanelKey, NodeId>,
    panel_last_sizes: &mut HashMap<PanelKey, Size>,
) -> Vec<(PanelKey, NodeId, Rect)> {
    let mut placements = Vec::with_capacity(panel_nodes.len());
    for (panel, node) in panel_nodes {
        let bounds = match snapshot.active_panel_bounds.get(panel).copied() {
            Some(rect) => {
                panel_last_sizes.insert(panel.clone(), rect.size);
                rect
            }
            None => hidden_bounds(
                panel_last_sizes
                    .get(panel)
                    .copied()
                    .unwrap_or(Size::new(Px(0.0), Px(0.0))),
            ),
        };
        placements.push((panel.clone(), *node, bounds));
    }
    placements
}
