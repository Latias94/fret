#[path = "collect/body.rs"]
mod body;
#[path = "collect/selection.rs"]
mod selection;

use super::*;
impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn collect_render_data<H: UiHost>(
        &self,
        host: &H,
        snapshot: &ViewSnapshot,
        geom: Arc<CanvasGeometry>,
        index: Arc<CanvasSpatialDerived>,
        render_cull_rect: Option<Rect>,
        zoom: f32,
        hovered_edge: Option<EdgeId>,
        include_groups: bool,
        include_nodes: bool,
        include_edges: bool,
    ) -> RenderData {
        let selections = selection::collect_render_selections(snapshot);
        let presenter: &dyn NodeGraphPresenter = &*self.presenter;
        body::collect_render_data(
            self,
            host,
            snapshot,
            geom,
            index,
            render_cull_rect,
            zoom,
            hovered_edge,
            include_groups,
            include_nodes,
            include_edges,
            presenter,
            selections,
        )
    }
}
