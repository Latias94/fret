use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn finish_paint_root_pass<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        edge_anchor_target_id: Option<EdgeId>,
        edge_anchor_target: Option<(EdgeRouteKind, Point, Point, Color)>,
        zoom: f32,
        viewport_origin_x: f32,
        viewport_origin_y: f32,
        viewport_w: f32,
        viewport_h: f32,
    ) {
        self.paint_edge_focus_anchors(
            cx,
            snapshot,
            edge_anchor_target_id,
            edge_anchor_target,
            zoom,
        );
        self.paint_overlays(
            cx,
            snapshot,
            zoom,
            viewport_origin_x,
            viewport_origin_y,
            viewport_w,
            viewport_h,
        );
        self.prune_paint_caches(cx.services, snapshot);
        cx.scene.push(SceneOp::PopClip);
    }
}
