use std::sync::Arc;

use fret_canvas::view::PanZoom2D;
use fret_core::{Point, Px, Rect};

use crate::io::NodeGraphViewState;

use super::{
    DerivedGeometryCacheState, EdgePaintCacheState, GridPaintCacheState, NodeDragState,
    canvas_viewport_rect, node_drag_contains, rect_union, rects_intersect,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct EdgePaintDiagnostics {
    pub(super) total: u32,
    pub(super) drawn: u32,
    pub(super) culled: u32,
    pub(super) dragged: u32,
    pub(super) missing_ports: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct PortalDiagnostics {
    pub(super) fit_count: u64,
    pub(super) fit_pending: bool,
    pub(super) union_width: f32,
    pub(super) union_height: f32,
    pub(super) bounds_entries: usize,
    pub(super) disabled: bool,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SurfaceSemanticsParams<'a> {
    pub(super) panning: bool,
    pub(super) marquee_active: bool,
    pub(super) node_drag_armed: bool,
    pub(super) node_dragging: bool,
    pub(super) hovered: bool,
    pub(super) selected_nodes_len: usize,
    pub(super) grid_cached: bool,
    pub(super) geom_cached: bool,
    pub(super) nodes_cached: bool,
    pub(super) edges_cached: bool,
    pub(super) grid_rebuilds: u64,
    pub(super) geom_rebuilds: u64,
    pub(super) nodes_rebuilds: u64,
    pub(super) edges_rebuilds: u64,
    pub(super) edges: EdgePaintDiagnostics,
    pub(super) paint_overrides_rev: u64,
    pub(super) view_state: &'a NodeGraphViewState,
    pub(super) portal: PortalDiagnostics,
}

pub(super) fn collect_edge_paint_diagnostics(
    edges_cache: &EdgePaintCacheState,
    grid_cache: &GridPaintCacheState,
    derived_cache: &DerivedGeometryCacheState,
    view_state: &NodeGraphViewState,
    cull_margin_screen_px: f32,
    node_drag: Option<&NodeDragState>,
) -> EdgePaintDiagnostics {
    edges_cache
        .draws
        .as_deref()
        .map(|draws| {
            let mut total: u32 = draws.len() as u32;
            let mut drawn: u32 = 0;
            let mut culled: u32 = 0;
            let mut dragged: u32 = 0;
            let mut missing_ports: u32 = 0;

            let bounds = grid_cache.bounds;
            let view = PanZoom2D {
                pan: Point::new(Px(view_state.pan.x), Px(view_state.pan.y)),
                zoom: view_state.zoom,
            };
            let Some(cull) = canvas_viewport_rect(bounds, view, cull_margin_screen_px) else {
                return EdgePaintDiagnostics {
                    total,
                    drawn,
                    culled,
                    dragged,
                    missing_ports,
                };
            };

            let drag_active = node_drag.is_some_and(NodeDragState::is_active);
            let geom = derived_cache.geom.as_deref();

            for draw in draws.iter() {
                let mut affected_by_drag = false;
                if drag_active {
                    let from_node = geom
                        .and_then(|geometry| geometry.ports.get(&draw.from))
                        .map(|handle| handle.node);
                    let to_node = geom
                        .and_then(|geometry| geometry.ports.get(&draw.to))
                        .map(|handle| handle.node);
                    affected_by_drag = from_node.is_some_and(|id| {
                        node_drag.is_some_and(|drag| node_drag_contains(drag, id))
                    }) || to_node.is_some_and(|id| {
                        node_drag.is_some_and(|drag| node_drag_contains(drag, id))
                    });
                }

                if affected_by_drag {
                    dragged += 1;
                    let ok_from = geom
                        .and_then(|geometry| geometry.port_center(draw.from))
                        .is_some();
                    let ok_to = geom
                        .and_then(|geometry| geometry.port_center(draw.to))
                        .is_some();
                    if ok_from && ok_to {
                        drawn += 1;
                    } else {
                        missing_ports += 1;
                    }
                    continue;
                }

                if !rects_intersect(cull, draw.bbox) {
                    culled += 1;
                } else {
                    drawn += 1;
                }
            }

            if draws.len() > (u32::MAX as usize) {
                total = u32::MAX;
            }

            EdgePaintDiagnostics {
                total,
                drawn,
                culled,
                dragged,
                missing_ports,
            }
        })
        .unwrap_or_default()
}

pub(super) fn collect_portal_diagnostics(
    portal_bounds_store: &super::PortalBoundsStore,
    portals_disabled: bool,
) -> PortalDiagnostics {
    let mut union: Option<Rect> = None;
    for rect in portal_bounds_store.nodes_canvas_bounds.values().copied() {
        union = Some(match union {
            Some(prev) => rect_union(prev, rect),
            None => rect,
        });
    }

    let (union_width, union_height) = union
        .map(|rect| (rect.size.width.0, rect.size.height.0))
        .unwrap_or((0.0, 0.0));

    PortalDiagnostics {
        fit_count: portal_bounds_store.fit_to_portals_count,
        fit_pending: portal_bounds_store.pending_fit_to_portals,
        union_width,
        union_height,
        bounds_entries: portal_bounds_store.nodes_canvas_bounds.len(),
        disabled: portals_disabled,
    }
}

pub(super) fn build_surface_semantics_value(params: SurfaceSemanticsParams<'_>) -> Arc<str> {
    let edges_paint_ok =
        params.edges.total > 0 && params.edges.drawn > 0 && params.edges.missing_ports == 0;

    Arc::from(format!(
        "panning {}; marquee_active:{}; node_drag_armed:{}; node_dragging:{}; hovered_node:{}; selected_nodes:{}; grid_cached:{}; grid_rebuilds:{}; geom_cached:{}; geom_rebuilds:{}; nodes_cached:{}; nodes_rebuilds:{}; edges_cached:{}; edges_rebuilds:{}; edges_paint_total:{}; edges_paint_drawn:{}; edges_paint_culled:{}; edges_paint_dragged:{}; edges_paint_missing_ports:{}; edges_paint_ok:{}; paint_overrides_rev:{}; view_pan:{:.2},{:.2}; view_zoom:{:.4}; portal_fit_count:{}; portal_fit_pending:{}; portal_union_wh:{:.2}x{:.2}; portal_bounds_entries:{}; portals_disabled:{};",
        params.panning,
        params.marquee_active,
        params.node_drag_armed,
        params.node_dragging,
        params.hovered,
        params.selected_nodes_len,
        params.grid_cached,
        params.grid_rebuilds,
        params.geom_cached,
        params.geom_rebuilds,
        params.nodes_cached,
        params.nodes_rebuilds,
        params.edges_cached,
        params.edges_rebuilds,
        params.edges.total,
        params.edges.drawn,
        params.edges.culled,
        params.edges.dragged,
        params.edges.missing_ports,
        edges_paint_ok,
        params.paint_overrides_rev,
        params.view_state.pan.x,
        params.view_state.pan.y,
        params.view_state.zoom,
        params.portal.fit_count,
        params.portal.fit_pending,
        params.portal.union_width,
        params.portal.union_height,
        params.portal.bounds_entries,
        params.portal.disabled,
    ))
}
