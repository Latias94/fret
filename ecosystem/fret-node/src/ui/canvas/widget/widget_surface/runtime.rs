#[path = "runtime/edge.rs"]
mod edge;
#[path = "runtime/interaction.rs"]
mod interaction;
#[path = "runtime/render.rs"]
mod render;

use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn compute_render_cull_rect(
        &self,
        snapshot: &ViewSnapshot,
        bounds: Rect,
    ) -> Option<Rect> {
        render::compute_render_cull_rect(self, snapshot, bounds)
    }

    #[cfg(test)]
    pub(in super::super) fn debug_derived_build_counters(
        &self,
    ) -> crate::ui::canvas::state::DerivedBuildCounters {
        render::debug_derived_build_counters(self)
    }

    #[cfg(test)]
    pub(in super::super) fn debug_render_metrics_for_bounds<H: UiHost>(
        &mut self,
        host: &mut H,
        bounds: Rect,
    ) -> paint_render_data::RenderMetrics {
        render::debug_render_metrics_for_bounds(self, host, bounds)
    }

    pub(in super::super) fn view_interacting(&self) -> bool {
        interaction::view_interacting(&self.interaction)
    }

    pub(in super::super) fn edge_render_hint(
        &self,
        graph: &Graph,
        edge_id: EdgeId,
    ) -> EdgeRenderHint {
        edge::edge_render_hint(self, graph, edge_id)
    }

    pub(in super::super) fn edge_custom_path(
        &self,
        graph: &Graph,
        edge_id: EdgeId,
        hint: &EdgeRenderHint,
        from: Point,
        to: Point,
        zoom: f32,
    ) -> Option<crate::ui::edge_types::EdgeCustomPath> {
        edge::edge_custom_path(self, graph, edge_id, hint, from, to, zoom)
    }
}
