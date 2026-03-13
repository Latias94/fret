#[path = "selected_nodes/body.rs"]
mod body;
#[path = "selected_nodes/selection.rs"]
mod selection;

use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn collect_selected_nodes_render_data<H: UiHost>(
        &self,
        host: &H,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
        render_cull_rect: Option<Rect>,
        zoom: f32,
    ) -> RenderData {
        let selected_nodes = selection::selected_nodes(snapshot);
        if selected_nodes.is_empty() {
            return RenderData::default();
        }

        let presenter: &dyn NodeGraphPresenter = &*self.presenter;
        body::collect_selected_nodes_render_data(
            self,
            host,
            geom,
            render_cull_rect,
            zoom,
            presenter,
            selected_nodes,
        )
    }
}
