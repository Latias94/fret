use crate::ui::canvas::widget::*;

#[path = "cached_edges/mod.rs"]
mod cached_edges;
#[path = "cached_groups.rs"]
mod cached_groups;
#[path = "cached_nodes.rs"]
mod cached_nodes;

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
            self.paint_root_groups_cached_path(
                cx,
                &snapshot,
                &plan.geom,
                &plan.index,
                cache_rect,
                render_cull_rect,
                zoom,
                plan.base_key,
                plan.style_key,
                plan.nodes_cache_tile_size_canvas,
            );

            // --- Edges (static + overlays) ---
            let (edge_anchor_target_id, edge_anchor_target) = self.paint_root_edges_cached_path(
                cx,
                &snapshot,
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
                &snapshot,
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
                &snapshot,
                edge_anchor_target_id,
                edge_anchor_target,
                zoom,
            );
            self.paint_overlays(
                cx,
                &snapshot,
                zoom,
                viewport_origin_x,
                viewport_origin_y,
                viewport_w,
                viewport_h,
            );

            self.prune_paint_caches(cx.services, &snapshot);

            cx.scene.push(SceneOp::PopClip);
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
