use crate::ui::canvas::widget::paint_root::cache_plan::PaintRootCachePlan;
use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn paint_root_cached_pass<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        plan: &PaintRootCachePlan,
        cache_rect: Rect,
        render_cull_rect: Option<Rect>,
        viewport_rect: Rect,
        viewport_w: f32,
        viewport_h: f32,
        zoom: f32,
        view_interacting: bool,
        viewport_origin_x: f32,
        viewport_origin_y: f32,
    ) {
        self.paint_root_groups_cached_path(
            cx,
            snapshot,
            &plan.geom,
            &plan.index,
            cache_rect,
            render_cull_rect,
            zoom,
            plan.base_key,
            plan.style_key,
            plan.nodes_cache_tile_size_canvas,
        );

        let (edge_anchor_target_id, edge_anchor_target) = self.paint_root_edges_cached_path(
            cx,
            snapshot,
            &plan.geom,
            &plan.index,
            plan.hovered_edge,
            cache_rect,
            plan.edges_cache_rect,
            render_cull_rect,
            viewport_rect,
            viewport_w,
            viewport_h,
            zoom,
            view_interacting,
            plan.base_key,
            plan.style_key,
            plan.edges_cache_tile_size_canvas,
        );

        self.paint_root_nodes_cached_path(
            cx,
            snapshot,
            &plan.geom,
            &plan.index,
            cache_rect,
            render_cull_rect,
            zoom,
            plan.base_key,
            plan.style_key,
            plan.nodes_cache_tile_size_canvas,
        );

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
