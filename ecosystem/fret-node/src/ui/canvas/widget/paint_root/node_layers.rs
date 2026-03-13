use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_root_node_overlay_layers<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        render_cull_rect: Option<Rect>,
        zoom: f32,
    ) {
        if snapshot.interaction.elevate_nodes_on_select {
            let render_selected = self.collect_selected_nodes_render_data(
                &*cx.app,
                snapshot,
                geom,
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

        self.paint_nodes_dynamic_from_geometry(cx, snapshot, geom, zoom);
    }
}
