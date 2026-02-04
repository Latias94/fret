use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn group_rect_with_preview(
        &self,
        group_id: crate::core::GroupId,
        base: crate::core::CanvasRect,
    ) -> crate::core::CanvasRect {
        if let Some(resize) = self
            .interaction
            .group_resize
            .as_ref()
            .filter(|r| r.group == group_id)
        {
            return resize.current_rect;
        }
        if let Some(drag) = self
            .interaction
            .group_drag
            .as_ref()
            .filter(|d| d.group == group_id)
        {
            return drag.current_rect;
        }
        if let Some(rect) = self.interaction.node_resize.as_ref().and_then(|r| {
            r.current_groups
                .iter()
                .find(|(id, _)| *id == group_id)
                .map(|(_, rect)| *rect)
        }) {
            return rect;
        }
        if let Some(rect) = self.interaction.node_drag.as_ref().and_then(|d| {
            d.current_groups
                .iter()
                .find(|(id, _)| *id == group_id)
                .map(|(_, r)| *r)
        }) {
            return rect;
        }
        base
    }

    pub(in super::super) fn canvas_geometry<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
    ) -> Arc<CanvasGeometry> {
        self.ensure_canvas_derived_base(host, snapshot).0
    }

    pub(in super::super) fn ensure_canvas_derived_base<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
    ) -> (Arc<CanvasGeometry>, Arc<CanvasSpatialIndex>) {
        let geom_key = self.geometry_key(host, snapshot);
        let geom = self.ensure_canvas_geometry_cache(host, snapshot, geom_key);
        let index = self.ensure_spatial_index_cache(host, snapshot, geom_key, &geom);
        (geom, index)
    }

    fn ensure_canvas_geometry_cache<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
        geom_key: GeometryCacheKey,
    ) -> Arc<CanvasGeometry> {
        if self.geometry.geom_key != Some(geom_key) {
            self.geometry.geom_key = Some(geom_key);
            self.geometry.index_key = None;
            self.geometry.drag_preview = None;

            let zoom = snapshot.zoom;
            let node_origin = snapshot.interaction.node_origin.normalized();
            let style = self.style.clone();
            let draw_order = snapshot.draw_order.clone();
            let graph = self.graph.clone();
            let presenter = &mut *self.presenter;

            let geom = graph
                .read_ref(host, |graph| {
                    CanvasGeometry::build_with_presenter(
                        graph,
                        &draw_order,
                        &style,
                        zoom,
                        node_origin,
                        presenter,
                    )
                })
                .ok()
                .unwrap_or_default();
            self.geometry.geom = Arc::new(geom);
        }

        self.geometry.geom.clone()
    }
}
