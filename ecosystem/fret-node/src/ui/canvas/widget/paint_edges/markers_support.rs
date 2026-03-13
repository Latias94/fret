#[path = "markers_support/custom.rs"]
mod custom;
#[path = "markers_support/paint.rs"]
mod paint;
#[path = "markers_support/route.rs"]
mod route;

use crate::ui::canvas::widget::*;
use fret_core::PathId;
use fret_core::scene::{DashPatternV1, PaintBindingV1};

use super::markers::WireHighlightPaint;

pub(super) fn marker_paint_binding_for_wire(paint: PaintBindingV1, color: Color) -> PaintBindingV1 {
    paint::marker_paint_binding_for_wire(paint, color)
}

pub(super) fn push_marker_path(
    scene: &mut fret_core::Scene,
    path: Option<PathId>,
    paint: PaintBindingV1,
) {
    paint::push_marker_path(scene, path, paint);
}

#[allow(clippy::too_many_arguments)]
pub(super) fn push_wire_highlight_path(
    scene: &mut fret_core::Scene,
    path: Option<PathId>,
    highlight: Option<WireHighlightPaint>,
) {
    paint::push_wire_highlight_path(scene, path, highlight);
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
    route::route_marker_paths_budgeted(
        canvas,
        services,
        route,
        from,
        to,
        zoom,
        scale_factor,
        start_marker,
        end_marker,
        marker_budget,
        stop_on_marker_skip,
    )
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
    custom::custom_marker_paths_budgeted(
        canvas,
        services,
        from,
        to,
        start_tangent,
        end_tangent,
        zoom,
        scale_factor,
        start_marker,
        end_marker,
        marker_budget,
        stop_on_marker_skip,
    )
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
    route::route_wire_path(
        canvas,
        services,
        route,
        from,
        to,
        zoom,
        scale_factor,
        width,
        dash,
    )
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
    custom::custom_wire_path(
        canvas,
        services,
        cache_key,
        commands,
        zoom,
        scale_factor,
        width,
        dash,
    )
}
