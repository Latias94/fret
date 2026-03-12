#[path = "cached_budgeted/labels.rs"]
mod labels;
#[path = "cached_budgeted/wires.rs"]
mod wires;

use crate::ui::canvas::widget::paint_render_data::EdgeRender;
use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn paint_edges_cached_budgeted<H: UiHost>(
        &mut self,
        tmp: &mut fret_core::Scene,
        host: &H,
        services: &mut dyn fret_core::UiServices,
        edges: &[EdgeRender],
        zoom: f32,
        scale_factor: f32,
        next_edge: usize,
        wire_budget: &mut WorkBudget,
        marker_budget: &mut WorkBudget,
    ) -> (usize, bool) {
        wires::paint_edges_cached_budgeted(
            self,
            tmp,
            host,
            services,
            edges,
            zoom,
            scale_factor,
            next_edge,
            wire_budget,
            marker_budget,
        )
    }

    pub(in super::super) fn paint_edge_labels_static_budgeted_cached<H: UiHost>(
        &mut self,
        scene: &mut fret_core::Scene,
        host: &H,
        services: &mut dyn fret_core::UiServices,
        scale_factor: f32,
        edges: &[EdgeRender],
        bezier_steps: usize,
        zoom: f32,
        start_edge: usize,
        budget: &mut WorkBudget,
    ) -> (usize, bool) {
        labels::paint_edge_labels_static_budgeted_cached(
            self,
            scene,
            host,
            services,
            scale_factor,
            edges,
            bezier_steps,
            zoom,
            start_edge,
            budget,
        )
    }

    pub(in super::super) fn paint_edge_labels_static_budgeted(
        &mut self,
        scene: &mut fret_core::Scene,
        services: &mut dyn fret_core::UiServices,
        scale_factor: f32,
        edges: &[EdgeRender],
        custom_paths: Option<&HashMap<EdgeId, crate::ui::edge_types::EdgeCustomPath>>,
        bezier_steps: usize,
        zoom: f32,
        start_edge: usize,
        budget: &mut WorkBudget,
    ) -> (usize, bool) {
        labels::paint_edge_labels_static_budgeted(
            self,
            scene,
            services,
            scale_factor,
            edges,
            custom_paths,
            bezier_steps,
            zoom,
            start_edge,
            budget,
        )
    }
}
