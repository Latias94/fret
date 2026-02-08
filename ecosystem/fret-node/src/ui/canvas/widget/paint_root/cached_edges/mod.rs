use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

mod build_state;
mod keys;
mod single_rect;
mod tile_path;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_root_edges_cached_path<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialDerived>,
        hovered_edge: Option<EdgeId>,
        cache_rect: Rect,
        edges_cache_rect: Option<Rect>,
        render_cull_rect: Option<Rect>,
        viewport_rect: Rect,
        viewport_w: f32,
        viewport_h: f32,
        zoom: f32,
        view_interacting: bool,
        base_key: DerivedBaseKey,
        style_key: u64,
        edges_cache_tile_size_canvas: f32,
    ) -> (Option<EdgeId>, Option<(EdgeRouteKind, Point, Point, Color)>) {
        let replay_delta = Point::new(Px(0.0), Px(0.0));

        // --- Edges (static + overlays) ---
        let edges_cache_allowed = self.interaction.pending_wire_drag.is_none()
            && self.interaction.wire_drag.is_none()
            && self.interaction.suspended_wire_drag.is_none()
            && self.interaction.pending_edge_insert_drag.is_none()
            && self.interaction.edge_insert_drag.is_none()
            && self.interaction.edge_drag.is_none()
            && self.interaction.pending_insert_node_drag.is_none()
            && self.interaction.insert_node_drag_preview.is_none();

        let edge_anchor_target_id = self
            .interaction
            .focused_edge
            .or_else(|| (snapshot.selected_edges.len() == 1).then(|| snapshot.selected_edges[0]))
            .filter(|edge_id| {
                self.graph
                    .read_ref(cx.app, |g| {
                        let edge = g.edges.get(edge_id)?;
                        let (allow_source, allow_target) =
                            Self::edge_reconnectable_flags(edge, &snapshot.interaction);
                        Some(allow_source || allow_target)
                    })
                    .ok()
                    .flatten()
                    .unwrap_or(false)
            });
        let edge_anchor_target: Option<(EdgeRouteKind, Point, Point, Color)> =
            edge_anchor_target_id.and_then(|edge_id| {
                self.graph
                    .read_ref(cx.app, |g| {
                        let edge = g.edges.get(&edge_id)?;
                        let from = geom.port_center(edge.from)?;
                        let to = geom.port_center(edge.to)?;
                        let hint = EdgePathContext::new(
                            &self.style,
                            &*self.presenter,
                            self.edge_types.as_ref(),
                        )
                        .edge_render_hint_normalized(g, edge_id);
                        let mut color = self.presenter.edge_color(g, edge_id, &self.style);
                        if let Some(override_color) = hint.color {
                            color = override_color;
                        }
                        Some((hint.route, from, to, color))
                    })
                    .ok()
                    .flatten()
            });

        if edges_cache_allowed {
            if edges_cache_tile_size_canvas.is_finite()
                && (edges_cache_tile_size_canvas < viewport_w
                    || edges_cache_tile_size_canvas < viewport_h)
            {
                self.paint_root_edges_cached_path_tiled(
                    cx,
                    snapshot,
                    geom,
                    index,
                    hovered_edge,
                    render_cull_rect,
                    viewport_rect,
                    zoom,
                    view_interacting,
                    base_key,
                    style_key,
                    edges_cache_tile_size_canvas,
                    replay_delta,
                );
            } else {
                self.paint_root_edges_cached_path_single_rect(
                    cx,
                    snapshot,
                    geom,
                    index,
                    hovered_edge,
                    cache_rect,
                    edges_cache_rect,
                    render_cull_rect,
                    zoom,
                    view_interacting,
                    base_key,
                    style_key,
                    edges_cache_tile_size_canvas,
                    replay_delta,
                );
            }
        } else {
            self.edges_build_states.clear();
            self.edge_labels_build_states.clear();
            self.edge_labels_build_state = None;
            let render_edges: RenderData = self.collect_render_data(
                &*cx.app,
                snapshot,
                Arc::clone(geom),
                Arc::clone(index),
                render_cull_rect,
                zoom,
                hovered_edge,
                false,
                false,
                true,
            );
            self.paint_edges(cx, snapshot, &render_edges, geom, zoom, view_interacting);
        }

        (edge_anchor_target_id, edge_anchor_target)
    }
}
