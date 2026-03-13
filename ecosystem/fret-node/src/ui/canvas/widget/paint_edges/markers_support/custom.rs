use crate::ui::canvas::widget::*;
use fret_core::PathId;
use fret_core::scene::DashPatternV1;

#[allow(clippy::too_many_arguments)]
pub(super) fn custom_marker_paths_budgeted<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    services: &mut dyn fret_core::UiServices,
    from: Point,
    to: Point,
    start_tangent: Point,
    end_tangent: Point,
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
            let (path, skipped_by_budget) = canvas
                .paint_cache
                .edge_end_marker_path_budgeted_with_tangent(
                    services,
                    to,
                    end_tangent,
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
        let (path, skipped_by_budget) = canvas
            .paint_cache
            .edge_end_marker_path_budgeted_with_tangent(
                services,
                to,
                end_tangent,
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
            let (path, skipped_by_budget) = canvas
                .paint_cache
                .edge_start_marker_path_budgeted_with_tangent(
                    services,
                    from,
                    start_tangent,
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
        let (path, skipped_by_budget) = canvas
            .paint_cache
            .edge_start_marker_path_budgeted_with_tangent(
                services,
                from,
                start_tangent,
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
pub(super) fn custom_wire_path<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    services: &mut dyn fret_core::UiServices,
    cache_key: u64,
    commands: &[fret_core::PathCommand],
    zoom: f32,
    scale_factor: f32,
    width: f32,
    dash: Option<DashPatternV1>,
) -> Option<PathId> {
    canvas.paint_cache.wire_path_from_commands(
        services,
        cache_key,
        commands,
        zoom,
        scale_factor,
        width,
        dash,
    )
}
