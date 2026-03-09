use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_root_immediate_pass<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialDerived>,
        hovered_edge: Option<EdgeId>,
        render_cull_rect: Option<Rect>,
        view_interacting: bool,
        zoom: f32,
    ) -> (Option<EdgeId>, Option<(EdgeRouteKind, Point, Point, Color)>) {
        let render: RenderData = self.collect_render_data(
            &*cx.app,
            snapshot,
            Arc::clone(geom),
            Arc::clone(index),
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
        self.paint_edges(cx, snapshot, &render, geom, zoom, view_interacting);
        self.paint_nodes_static(cx.scene, cx.services, cx.scale_factor, &render, zoom);
        self.paint_root_node_overlay_layers(cx, snapshot, geom, render_cull_rect, zoom);

        (edge_anchor_target_id, edge_anchor_target)
    }
}
