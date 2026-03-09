use crate::ui::canvas::widget::*;
use fret_core::scene::DashPatternV1;
use fret_core::scene::PaintBindingV1;

use super::markers_support::{
    custom_marker_paths_budgeted, custom_wire_path, marker_paint_binding_for_wire,
    push_marker_path, push_wire_highlight_path, route_marker_paths_budgeted, route_wire_path,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct WireHighlightPaint {
    pub width: f32,
    pub color: Color,
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn push_edge_wire_and_markers_budgeted(
        &mut self,
        scene: &mut fret_core::Scene,
        services: &mut dyn fret_core::UiServices,
        zoom: f32,
        scale_factor: f32,
        route: EdgeRouteKind,
        from: Point,
        to: Point,
        paint: PaintBindingV1,
        color: Color,
        width: f32,
        dash: Option<DashPatternV1>,
        highlight: Option<WireHighlightPaint>,
        start_marker: Option<&crate::ui::presenter::EdgeMarker>,
        end_marker: Option<&crate::ui::presenter::EdgeMarker>,
        wire_budget: &mut WorkBudget,
        marker_budget: &mut WorkBudget,
        stop_on_marker_skip: bool,
    ) -> (bool, u32) {
        if !wire_budget.try_consume(1) {
            return (true, 0);
        }

        let marker_paint = marker_paint_binding_for_wire(paint, color);
        let (start_path, end_path, marker_skipped) = match route_marker_paths_budgeted(
            self,
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
        ) {
            Ok(result) => result,
            Err(skipped) => return (true, skipped),
        };

        let wire_path = route_wire_path(
            self,
            services,
            route,
            from,
            to,
            zoom,
            scale_factor,
            width,
            dash,
        );
        push_marker_path(scene, wire_path, paint);
        let highlight_path = highlight.and_then(|value| {
            route_wire_path(
                self,
                services,
                route,
                from,
                to,
                zoom,
                scale_factor,
                value.width,
                dash,
            )
        });
        push_wire_highlight_path(scene, highlight_path, highlight);
        push_marker_path(scene, end_path, marker_paint);
        push_marker_path(scene, start_path, marker_paint);

        (false, marker_skipped)
    }

    pub(super) fn push_edge_custom_wire_and_markers_budgeted(
        &mut self,
        scene: &mut fret_core::Scene,
        services: &mut dyn fret_core::UiServices,
        cache_key: u64,
        commands: &[fret_core::PathCommand],
        start_tangent: Point,
        end_tangent: Point,
        zoom: f32,
        scale_factor: f32,
        from: Point,
        to: Point,
        paint: PaintBindingV1,
        color: Color,
        width: f32,
        dash: Option<DashPatternV1>,
        highlight: Option<WireHighlightPaint>,
        start_marker: Option<&crate::ui::presenter::EdgeMarker>,
        end_marker: Option<&crate::ui::presenter::EdgeMarker>,
        wire_budget: &mut WorkBudget,
        marker_budget: &mut WorkBudget,
        stop_on_marker_skip: bool,
    ) -> (bool, u32) {
        if !wire_budget.try_consume(1) {
            return (true, 0);
        }

        let marker_paint = marker_paint_binding_for_wire(paint, color);
        let (start_path, end_path, marker_skipped) = match custom_marker_paths_budgeted(
            self,
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
        ) {
            Ok(result) => result,
            Err(skipped) => return (true, skipped),
        };

        let wire_path = custom_wire_path(
            self,
            services,
            cache_key,
            commands,
            zoom,
            scale_factor,
            width,
            dash,
        );
        push_marker_path(scene, wire_path, paint);
        let highlight_path = highlight.and_then(|value| {
            custom_wire_path(
                self,
                services,
                cache_key,
                commands,
                zoom,
                scale_factor,
                value.width,
                dash,
            )
        });
        push_wire_highlight_path(scene, highlight_path, highlight);
        push_marker_path(scene, end_path, marker_paint);
        push_marker_path(scene, start_path, marker_paint);

        (false, marker_skipped)
    }
}
