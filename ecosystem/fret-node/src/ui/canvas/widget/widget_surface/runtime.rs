use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn compute_render_cull_rect(
        &self,
        snapshot: &ViewSnapshot,
        bounds: Rect,
    ) -> Option<Rect> {
        if !snapshot.interaction.only_render_visible_elements {
            return None;
        }

        let zoom = snapshot.zoom;
        if !zoom.is_finite() || zoom <= 1.0e-6 {
            return None;
        }

        let viewport = Self::viewport_from_pan_zoom(bounds, snapshot.pan, zoom);
        let viewport_rect = viewport.visible_canvas_rect();
        let viewport_w = viewport_rect.size.width.0;
        let viewport_h = viewport_rect.size.height.0;
        let margin_screen = self.style.paint.render_cull_margin_px;

        if !margin_screen.is_finite()
            || margin_screen <= 0.0
            || !viewport_w.is_finite()
            || !viewport_h.is_finite()
            || viewport_w <= 0.0
            || viewport_h <= 0.0
        {
            return None;
        }

        let margin = margin_screen / zoom;
        Some(inflate_rect(viewport_rect, margin))
    }

    #[cfg(test)]
    pub(in super::super) fn debug_derived_build_counters(
        &self,
    ) -> crate::ui::canvas::state::DerivedBuildCounters {
        self.geometry.counters
    }

    #[cfg(test)]
    pub(in super::super) fn debug_render_metrics_for_bounds<H: UiHost>(
        &mut self,
        host: &mut H,
        bounds: Rect,
    ) -> paint_render_data::RenderMetrics {
        let snapshot = self.sync_view_state(host);
        let zoom = snapshot.zoom;
        if !zoom.is_finite() || zoom <= 1.0e-6 {
            return paint_render_data::RenderMetrics::default();
        }

        let render_cull_rect = self.compute_render_cull_rect(&snapshot, bounds);
        let (geom, index) = self.canvas_derived(host, &snapshot);
        self.collect_render_data(
            host,
            &snapshot,
            geom,
            index,
            render_cull_rect,
            zoom,
            None,
            true,
            true,
            true,
        )
        .metrics
    }

    pub(in super::super) fn view_interacting(&self) -> bool {
        self.interaction.viewport_move_debounce.is_some()
            || self.interaction.panning
            || self.interaction.pan_inertia.is_some()
            || self.interaction.viewport_animation.is_some()
            || self.interaction.pending_marquee.is_some()
            || self.interaction.marquee.is_some()
            || self.interaction.pending_node_drag.is_some()
            || self.interaction.node_drag.is_some()
            || self.interaction.pending_group_drag.is_some()
            || self.interaction.group_drag.is_some()
            || self.interaction.pending_group_resize.is_some()
            || self.interaction.group_resize.is_some()
            || self.interaction.pending_node_resize.is_some()
            || self.interaction.node_resize.is_some()
            || self.interaction.pending_wire_drag.is_some()
            || self.interaction.wire_drag.is_some()
            || self.interaction.suspended_wire_drag.is_some()
            || self.interaction.pending_edge_insert_drag.is_some()
            || self.interaction.edge_insert_drag.is_some()
            || self.interaction.edge_drag.is_some()
            || self.interaction.pending_insert_node_drag.is_some()
            || self.interaction.insert_node_drag_preview.is_some()
            || self.interaction.context_menu.is_some()
            || self.interaction.searcher.is_some()
    }

    pub(in super::super) fn edge_render_hint(
        &self,
        graph: &Graph,
        edge_id: EdgeId,
    ) -> EdgeRenderHint {
        EdgePathContext::new(&self.style, &*self.presenter, self.edge_types.as_ref())
            .edge_render_hint(graph, edge_id)
    }

    pub(in super::super) fn edge_custom_path(
        &self,
        graph: &Graph,
        edge_id: EdgeId,
        hint: &EdgeRenderHint,
        from: Point,
        to: Point,
        zoom: f32,
    ) -> Option<crate::ui::edge_types::EdgeCustomPath> {
        EdgePathContext::new(&self.style, &*self.presenter, self.edge_types.as_ref())
            .edge_custom_path(graph, edge_id, hint, from, to, zoom)
    }
}
