use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn paint_root_immediate_path<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: Arc<CanvasGeometry>,
        index: Arc<CanvasSpatialDerived>,
        hovered_edge: Option<EdgeId>,
        render_cull_rect: Option<Rect>,
        view_interacting: bool,
        zoom: f32,
        viewport_origin_x: f32,
        viewport_origin_y: f32,
        viewport_w: f32,
        viewport_h: f32,
    ) {
        let (edge_anchor_target_id, edge_anchor_target) = self.paint_root_immediate_pass(
            cx,
            snapshot,
            &geom,
            &index,
            hovered_edge,
            render_cull_rect,
            view_interacting,
            zoom,
        );
        self.finish_paint_root_pass(
            cx,
            snapshot,
            edge_anchor_target_id,
            edge_anchor_target,
            zoom,
            viewport_origin_x,
            viewport_origin_y,
            viewport_w,
            viewport_h,
        );
    }
}
