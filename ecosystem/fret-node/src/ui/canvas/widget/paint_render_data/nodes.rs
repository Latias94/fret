#[path = "nodes/append.rs"]
mod append;
#[path = "nodes/overhead.rs"]
mod overhead;
#[path = "nodes/ports.rs"]
mod ports;
#[path = "nodes/visible.rs"]
mod visible;

use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn visible_node_ids_for_render(
        &self,
        geom: &CanvasGeometry,
        index: &CanvasSpatialDerived,
        cull: Option<Rect>,
    ) -> (usize, Vec<GraphNodeId>) {
        visible::visible_node_ids_for_render(geom, index, cull)
    }

    pub(in super::super) fn node_render_label_overhead(&self) -> f32 {
        overhead::node_render_label_overhead(self)
    }

    pub(in super::super) fn append_node_render_data(
        &self,
        graph: &Graph,
        geom: &CanvasGeometry,
        presenter: &dyn NodeGraphPresenter,
        out: &mut RenderData,
        node: GraphNodeId,
        is_selected: bool,
        zoom: f32,
        label_overhead: f32,
    ) {
        append::append_node_render_data(
            self,
            graph,
            geom,
            presenter,
            out,
            node,
            is_selected,
            zoom,
            label_overhead,
        );
    }
}
