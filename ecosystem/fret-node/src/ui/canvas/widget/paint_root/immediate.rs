use crate::ui::canvas::widget::paint_render_data::RenderData;
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
        // Fallback: immediate-mode paint (no static scene replay cache).
        let render: RenderData = self.collect_render_data(
            &*cx.app,
            snapshot,
            geom.clone(),
            index.clone(),
            render_cull_rect,
            zoom,
            hovered_edge,
            true,
            true,
            true,
        );

        let edge_anchor_target_id = self.resolve_edge_anchor_target_id(cx, snapshot);
        let edge_anchor_target =
            self.resolve_edge_anchor_target_from_render(&render, edge_anchor_target_id);

        self.paint_groups_static(cx.scene, cx.services, cx.scale_factor, &render.groups, zoom);
        self.paint_groups_selected_overlay(cx.scene, &render.groups, zoom);
        self.paint_edges(cx, snapshot, &render, &geom, zoom, view_interacting);
        self.paint_nodes_static(cx.scene, cx.services, cx.scale_factor, &render, zoom);
        if snapshot.interaction.elevate_nodes_on_select {
            let render_selected = self.collect_selected_nodes_render_data(
                &*cx.app,
                snapshot,
                &geom,
                render_cull_rect,
                zoom,
            );
            if !render_selected.nodes.is_empty() {
                self.paint_nodes_static(
                    cx.scene,
                    cx.services,
                    cx.scale_factor,
                    &render_selected,
                    zoom,
                );
            }
        }
        self.paint_nodes_dynamic_from_geometry(cx, snapshot, &geom, zoom);
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
