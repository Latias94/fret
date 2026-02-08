use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn geometry_key<H: UiHost>(
        &self,
        host: &H,
        snapshot: &ViewSnapshot,
    ) -> GeometryCacheKey {
        let zoom = snapshot.zoom;
        let node_origin = snapshot.interaction.node_origin.normalized();
        let graph_rev = self.graph.revision(host).unwrap_or(0);
        let presenter_rev = self.presenter.geometry_revision();
        let edge_types_rev = self.edge_types.as_ref().map(|t| t.revision()).unwrap_or(0);
        GeometryCacheKey {
            base: DerivedBaseKey {
                graph_rev,
                zoom_bits: zoom.to_bits(),
                node_origin_x_bits: node_origin.x.to_bits(),
                node_origin_y_bits: node_origin.y.to_bits(),
                draw_order: Self::draw_order_fingerprint(&snapshot.draw_order),
                presenter_rev,
                edge_types_rev,
            },
        }
    }
}
