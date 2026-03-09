use crate::ui::canvas::widget::*;
use fret_core::PathId;
use fret_core::scene::{DashPatternV1, PaintBindingV1, PaintEvalSpaceV1};

use super::markers::WireHighlightPaint;

pub(super) fn marker_paint_binding_for_wire(paint: PaintBindingV1, color: Color) -> PaintBindingV1 {
    if paint.eval_space == PaintEvalSpaceV1::StrokeS01 {
        color.into()
    } else {
        paint
    }
}

pub(super) fn push_marker_path(
    scene: &mut fret_core::Scene,
    path: Option<PathId>,
    paint: PaintBindingV1,
) {
    if let Some(path) = path {
        scene.push(SceneOp::Path {
            order: DrawOrder(2),
            origin: Point::new(Px(0.0), Px(0.0)),
            path,
            paint,
        });
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn push_wire_highlight_path(
    scene: &mut fret_core::Scene,
    path: Option<PathId>,
    highlight: Option<WireHighlightPaint>,
) {
    if let Some(highlight) = highlight
        && highlight.width.is_finite()
        && highlight.width > 1.0e-3
        && highlight.color.a > 0.0
    {
        push_marker_path(scene, path, highlight.color.into());
    }
}

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
