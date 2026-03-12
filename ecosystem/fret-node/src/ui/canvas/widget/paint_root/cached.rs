use crate::ui::canvas::widget::*;

#[path = "cached_edges/mod.rs"]
mod cached_edges;
#[path = "cached_groups.rs"]
mod cached_groups;
#[path = "cached_nodes.rs"]
mod cached_nodes;
#[path = "cached_pass.rs"]
mod cached_pass;
#[path = "static_cache.rs"]
mod static_cache;
#[path = "static_layer.rs"]
mod static_layer;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn paint_root<H: UiHost>(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(&self.graph, Invalidation::Paint);
        cx.observe_model(&self.view_state, Invalidation::Paint);
        let snapshot = self.sync_view_state(cx.app);

        let view_interacting = self.view_interacting();
        let zoom = snapshot.zoom;
        let frame = self.prepare_paint_root_frame(cx, &snapshot, view_interacting);
        let viewport_rect = frame.viewport_rect;
        let viewport_w = frame.viewport_w;
        let viewport_h = frame.viewport_h;
        let viewport_origin_x = frame.viewport_origin_x;
        let viewport_origin_y = frame.viewport_origin_y;
        let render_cull_rect = frame.render_cull_rect;

        let plan = self.prepare_paint_root_cache_plan(
            cx,
            &snapshot,
            viewport_rect,
            viewport_w,
            viewport_h,
        );

        if let Some(cache_rect) = plan.nodes_cache_rect {
            self.paint_root_cached_pass(
                cx,
                &snapshot,
                &plan,
                cache_rect,
                render_cull_rect,
                viewport_rect,
                viewport_w,
                viewport_h,
                zoom,
                view_interacting,
                viewport_origin_x,
                viewport_origin_y,
            );
            return;
        }

        self.paint_root_immediate_path(
            cx,
            &snapshot,
            plan.geom,
            plan.index,
            plan.hovered_edge,
            render_cull_rect,
            view_interacting,
            zoom,
            viewport_origin_x,
            viewport_origin_y,
            viewport_w,
            viewport_h,
        );
    }
}
