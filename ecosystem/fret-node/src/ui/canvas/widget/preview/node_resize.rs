use crate::ui::canvas::geometry::node_rect_origin_from_anchor;
use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn node_resize_preview_derived<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
        preview_rev: u64,
        node_id: GraphNodeId,
        pos: CanvasPoint,
        size_opt_px: Option<CanvasSize>,
    ) -> Option<(Arc<CanvasGeometry>, Arc<CanvasSpatialIndex>)> {
        let base_index_key = self.geometry.index_key?;
        let base_geom = self.geometry.geom.clone();
        let base_index = self.geometry.index.clone();

        let zoom = snapshot.zoom.max(1.0e-6);
        let base_rect = base_geom.nodes.get(&node_id)?.rect;
        let size = size_opt_px
            .map(|s| {
                Size::new(
                    Px((s.width / zoom).max(0.0)),
                    Px((s.height / zoom).max(0.0)),
                )
            })
            .unwrap_or(base_rect.size);
        let node_origin = snapshot.interaction.node_origin.normalized();
        let size_canvas = crate::core::CanvasSize {
            width: size.width.0,
            height: size.height.0,
        };
        let next_origin = node_rect_origin_from_anchor(pos, size_canvas, node_origin);
        let next_rect = Rect::new(Point::new(Px(next_origin.x), Px(next_origin.y)), size);

        let rebuild = self
            .geometry
            .drag_preview_rebuild_needed(DragPreviewKind::NodeResize, base_index_key);
        if rebuild {
            let node_ports = self
                .graph
                .read_ref(host, |g| {
                    let mut out: HashMap<GraphNodeId, Vec<PortId>> = HashMap::new();
                    if let Some(node) = g.nodes.get(&node_id) {
                        out.insert(node_id, node.ports.clone());
                    }
                    out
                })
                .ok()
                .unwrap_or_default();

            let mut geom = (*base_geom).clone();
            let mut index = (*base_index).clone();
            let mut node_positions: HashMap<GraphNodeId, CanvasPoint> = HashMap::new();
            let mut node_rects: HashMap<GraphNodeId, Rect> = HashMap::new();
            node_positions.insert(node_id, pos);
            node_rects.insert(node_id, next_rect);

            if let Some(node_geom) = geom.nodes.get_mut(&node_id) {
                let prev_rect = node_geom.rect;
                if prev_rect != next_rect {
                    node_geom.rect = next_rect;
                    index.update_node_rect(node_id, next_rect);
                    Self::update_ports_for_node_rect_change(
                        &mut geom,
                        &mut index,
                        node_id,
                        prev_rect,
                        next_rect,
                        node_ports
                            .get(&node_id)
                            .map(|v| v.as_slice())
                            .unwrap_or(&[]),
                    );
                }
            }

            Self::update_edges_for_ports(
                &mut geom,
                &mut index,
                snapshot.zoom,
                node_ports
                    .get(&node_id)
                    .map(|v| v.as_slice())
                    .unwrap_or(&[]),
                |edge_ids| {
                    self.graph
                        .read_ref(host, |g| {
                            edge_ids
                                .iter()
                                .filter_map(|edge_id| {
                                    g.edges.get(edge_id).map(|e| (*edge_id, e.from, e.to))
                                })
                                .collect::<Vec<_>>()
                        })
                        .ok()
                        .unwrap_or_default()
                },
            );

            self.geometry.set_drag_preview(DragPreviewCache {
                kind: DragPreviewKind::NodeResize,
                base_index_key,
                preview_rev,
                geom: Arc::new(geom),
                index: Arc::new(index),
                node_positions,
                node_rects,
                node_ports,
            });
        }

        let graph_model = self.graph.clone();
        self.geometry
            .drag_preview_outputs_for_rev(preview_rev, |meta, geom_mut, index_mut| {
                let prev_rect = meta.node_rects.get(&node_id).copied().unwrap_or(base_rect);
                if prev_rect != next_rect {
                    if let Some(node_geom) = geom_mut.nodes.get_mut(&node_id) {
                        node_geom.rect = next_rect;
                        index_mut.update_node_rect(node_id, next_rect);
                        Self::update_ports_for_node_rect_change(
                            geom_mut,
                            index_mut,
                            node_id,
                            prev_rect,
                            next_rect,
                            super::ports_for_node(meta.node_ports, node_id),
                        );
                    }

                    Self::update_edges_for_ports(
                        geom_mut,
                        index_mut,
                        snapshot.zoom,
                        super::ports_for_node(meta.node_ports, node_id),
                        |edge_ids| {
                            super::resolve_edge_endpoints_from_model(host, &graph_model, edge_ids)
                        },
                    );
                }

                super::set_preview_node_position_and_rect(meta, node_id, pos, next_rect);
            })
    }
}
