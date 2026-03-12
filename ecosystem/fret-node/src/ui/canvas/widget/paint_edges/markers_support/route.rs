use crate::ui::canvas::widget::*;
use fret_core::PathId;
use fret_core::scene::DashPatternV1;

#[allow(clippy::too_many_arguments)]
pub(super) fn route_marker_paths_budgeted<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    services: &mut dyn fret_core::UiServices,
    route: EdgeRouteKind,
    from: Point,
    to: Point,
    zoom: f32,
    scale_factor: f32,
    start_marker: Option<&crate::ui::presenter::EdgeMarker>,
    end_marker: Option<&crate::ui::presenter::EdgeMarker>,
    marker_budget: &mut WorkBudget,
    stop_on_marker_skip: bool,
) -> Result<(Option<PathId>, Option<PathId>, u32), u32> {
    let mut marker_skipped: u32 = 0;
    let pin_radius = canvas.style.geometry.pin_radius;

    let end_path = if stop_on_marker_skip {
        if let Some(marker) = end_marker {
            let (path, skipped_by_budget) = canvas.paint_cache.edge_end_marker_path_budgeted(
                services,
                route,
                from,
                to,
                zoom,
                scale_factor,
                marker,
                pin_radius,
                marker_budget,
            );
            if skipped_by_budget {
                return Err(1);
            }
            path
        } else {
            None
        }
    } else if let Some(marker) = end_marker {
        let (path, skipped_by_budget) = canvas.paint_cache.edge_end_marker_path_budgeted(
            services,
            route,
            from,
            to,
            zoom,
            scale_factor,
            marker,
            pin_radius,
            marker_budget,
        );
        if skipped_by_budget {
            marker_skipped = marker_skipped.saturating_add(1);
        }
        path
    } else {
        None
    };

    let start_path = if stop_on_marker_skip {
        if let Some(marker) = start_marker {
            let (path, skipped_by_budget) = canvas.paint_cache.edge_start_marker_path_budgeted(
                services,
                route,
                from,
                to,
                zoom,
                scale_factor,
                marker,
                pin_radius,
                marker_budget,
            );
            if skipped_by_budget {
                return Err(1);
            }
            path
        } else {
            None
        }
    } else if let Some(marker) = start_marker {
        let (path, skipped_by_budget) = canvas.paint_cache.edge_start_marker_path_budgeted(
            services,
            route,
            from,
            to,
            zoom,
            scale_factor,
            marker,
            pin_radius,
            marker_budget,
        );
        if skipped_by_budget {
            marker_skipped = marker_skipped.saturating_add(1);
        }
        path
    } else {
        None
    };

    Ok((start_path, end_path, marker_skipped))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn route_wire_path<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    services: &mut dyn fret_core::UiServices,
    route: EdgeRouteKind,
    from: Point,
    to: Point,
    zoom: f32,
    scale_factor: f32,
    width: f32,
    dash: Option<DashPatternV1>,
) -> Option<PathId> {
    canvas
        .paint_cache
        .wire_path(services, route, from, to, zoom, scale_factor, width, dash)
}
