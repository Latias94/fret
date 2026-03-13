#[path = "build_state/init.rs"]
mod init;
#[path = "build_state/ops.rs"]
mod ops;
#[path = "build_state/step.rs"]
mod step;

use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn init_edges_build_state<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialDerived>,
        clip_rect: Rect,
        cull_rect: Rect,
        zoom: f32,
    ) -> EdgesBuildState {
        init::init_edges_build_state(
            self, host, snapshot, geom, index, clip_rect, cull_rect, zoom,
        )
    }

    pub(super) fn init_edge_labels_build_state<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialDerived>,
        key: u64,
        clip_rect: Rect,
        cull_rect: Rect,
        zoom: f32,
    ) -> EdgeLabelsBuildState {
        init::init_edge_labels_build_state(
            self, host, snapshot, geom, index, key, clip_rect, cull_rect, zoom,
        )
    }

    pub(super) fn paint_edges_build_state_step<H: UiHost>(
        &mut self,
        tmp: &mut fret_core::Scene,
        host: &H,
        services: &mut dyn fret_core::UiServices,
        zoom: f32,
        scale_factor: f32,
        state: &mut EdgesBuildState,
        wire_budget: &mut WorkBudget,
        marker_budget: &mut WorkBudget,
    ) -> bool {
        step::paint_edges_build_state_step(
            self,
            tmp,
            host,
            services,
            zoom,
            scale_factor,
            state,
            wire_budget,
            marker_budget,
        )
    }

    pub(super) fn paint_edge_labels_build_state_step<H: UiHost>(
        &mut self,
        tmp: &mut fret_core::Scene,
        host: &H,
        services: &mut dyn fret_core::UiServices,
        scale_factor: f32,
        zoom: f32,
        bezier_steps: usize,
        state: &mut EdgeLabelsBuildState,
        budget: &mut WorkBudget,
    ) -> bool {
        step::paint_edge_labels_build_state_step(
            self,
            tmp,
            host,
            services,
            scale_factor,
            zoom,
            bezier_steps,
            state,
            budget,
        )
    }
}
