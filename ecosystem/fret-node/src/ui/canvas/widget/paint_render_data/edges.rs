use super::*;
use fret_core::scene::PaintBindingV1;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    fn candidate_edge_ids_for_render(
        &self,
        graph: &Graph,
        index: &CanvasSpatialDerived,
        cull: Option<Rect>,
    ) -> Vec<EdgeId> {
        let mut edge_ids: Vec<EdgeId> = Vec::new();
        if let Some(c) = cull {
            index.query_edges_in_rect(c, &mut edge_ids);
        } else {
            edge_ids.extend(graph.edges.keys().copied());
        }
        edge_ids
    }

    fn resolve_edge_render_hint(
        &self,
        graph: &Graph,
        edge_id: EdgeId,
        selected: bool,
        hovered: bool,
    ) -> EdgeRenderHint {
        let hint = EdgePathContext::new(&self.style, &*self.presenter, self.edge_types.as_ref())
            .edge_render_hint_normalized(graph, edge_id);
        if let Some(skin) = self.skin.as_ref() {
            skin.edge_render_hint(graph, edge_id, &self.style, &hint, selected, hovered)
                .normalized()
        } else {
            hint
        }
    }

    fn apply_edge_paint_override(
        &self,
        edge_id: EdgeId,
        mut hint: EdgeRenderHint,
    ) -> (
        EdgeRenderHint,
        Option<crate::ui::paint_overrides::EdgePaintOverrideV1>,
    ) {
        let paint_override = self
            .paint_overrides
            .as_ref()
            .and_then(|o| o.edge_paint_override(edge_id));
        if let Some(ov) = paint_override {
            if let Some(dash) = ov.dash {
                hint.dash = Some(dash);
            }
            if let Some(w) = ov.stroke_width_mul {
                hint.width_mul = hint.width_mul * w;
            }
        }
        (hint.normalized(), paint_override)
    }

    fn edge_cull_pad(
        &self,
        snapshot: &ViewSnapshot,
        edge_id: EdgeId,
        hint: &EdgeRenderHint,
        zoom: f32,
    ) -> f32 {
        let interaction_width_px = self
            .geometry_overrides
            .as_ref()
            .and_then(|o| o.edge_geometry_override(edge_id).interaction_width_px)
            .unwrap_or(snapshot.interaction.edge_interaction_width);
        (interaction_width_px
            .max(
                self.style.geometry.wire_width
                    * hint.width_mul
                    * self.style.paint.wire_width_selected_mul,
            )
            .max(
                self.style.geometry.wire_width
                    * hint.width_mul
                    * self.style.paint.wire_width_hover_mul,
            ))
            / zoom
    }

    fn edge_intersects_cull(
        &self,
        graph: &Graph,
        edge_id: EdgeId,
        hint: &EdgeRenderHint,
        from: Point,
        to: Point,
        cull: Rect,
        snapshot: &ViewSnapshot,
        zoom: f32,
    ) -> bool {
        let pad = self.edge_cull_pad(snapshot, edge_id, hint, zoom);
        let bounds =
            if let Some(custom) = self.edge_custom_path(graph, edge_id, hint, from, to, zoom) {
                path_bounds_rect(&custom.commands)
                    .map(|r| inflate_rect(r, pad))
                    .unwrap_or_else(|| edge_bounds_rect(hint.route, from, to, zoom, pad))
            } else {
                edge_bounds_rect(hint.route, from, to, zoom, pad)
            };
        rects_intersect(bounds, cull)
    }

    fn edge_rank_for_render(&self, geom: &CanvasGeometry, edge: &Edge) -> u32 {
        geom.ports
            .get(&edge.from)
            .and_then(|p| geom.node_rank.get(&p.node).copied())
            .unwrap_or(0)
            .max(
                geom.ports
                    .get(&edge.to)
                    .and_then(|p| geom.node_rank.get(&p.node).copied())
                    .unwrap_or(0),
            )
    }

    pub(in super::super) fn collect_edge_render_data(
        &self,
        graph: &Graph,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
        index: &CanvasSpatialDerived,
        presenter: &dyn NodeGraphPresenter,
        selected_edges: &HashSet<EdgeId>,
        hovered_edge: Option<EdgeId>,
        cull: Option<Rect>,
        zoom: f32,
        out: &mut RenderData,
    ) {
        out.metrics.edge_total = graph.edges.len();
        let edge_ids = self.candidate_edge_ids_for_render(graph, index, cull);
        out.metrics.edge_candidates = edge_ids.len();

        for edge_id in edge_ids {
            let Some(edge) = graph.edges.get(&edge_id) else {
                continue;
            };
            if self
                .interaction
                .wire_drag
                .as_ref()
                .is_some_and(|w| Self::wire_drag_suppresses_edge(&w.kind, edge_id))
            {
                continue;
            }
            use std::collections::hash_map::Entry;

            let from = match out.port_centers.entry(edge.from) {
                Entry::Occupied(v) => *v.get(),
                Entry::Vacant(v) => {
                    let Some(center) = geom.port_center(edge.from) else {
                        continue;
                    };
                    *v.insert(center)
                }
            };
            let to = match out.port_centers.entry(edge.to) {
                Entry::Occupied(v) => *v.get(),
                Entry::Vacant(v) => {
                    let Some(center) = geom.port_center(edge.to) else {
                        continue;
                    };
                    *v.insert(center)
                }
            };

            let selected = selected_edges.contains(&edge_id);
            let hovered = hovered_edge == Some(edge_id);
            let hint = self.resolve_edge_render_hint(graph, edge_id, selected, hovered);
            let (hint, paint_override) = self.apply_edge_paint_override(edge_id, hint);

            if cull.is_some_and(|c| {
                !self.edge_intersects_cull(graph, edge_id, &hint, from, to, c, snapshot, zoom)
            }) {
                continue;
            }

            let mut color = presenter.edge_color(graph, edge_id, &self.style);
            if let Some(override_color) = hint.color {
                color = override_color;
            }

            let mut paint: PaintBindingV1 = color.into();
            if let Some(ov) = paint_override
                && let Some(p) = ov.stroke_paint
            {
                paint = p;
            }

            out.edges.push(EdgeRender {
                id: edge_id,
                rank: self.edge_rank_for_render(geom, edge),
                from,
                to,
                color,
                paint,
                hint,
                selected,
                hovered,
            });
            out.metrics.edge_visible = out.metrics.edge_visible.saturating_add(1);
        }

        out.edges
            .sort_unstable_by(|a, b| a.rank.cmp(&b.rank).then_with(|| a.id.cmp(&b.id)));
    }
}
