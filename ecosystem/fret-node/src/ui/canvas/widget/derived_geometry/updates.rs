use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn canvas_derived<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
    ) -> (Arc<CanvasGeometry>, Arc<CanvasSpatialIndex>) {
        let (geom, index) = self.ensure_canvas_derived_base(host, snapshot);
        let node_drag = self.interaction.node_drag.clone();
        let group_drag = self.interaction.group_drag.clone();
        let node_resize = self.interaction.node_resize.clone();

        if let Some(drag) = node_drag.as_ref() {
            if let Some((geom, index)) = self.drag_preview_derived(
                host,
                snapshot,
                DragPreviewKind::NodeDrag,
                drag.preview_rev,
                &drag.current_nodes,
            ) {
                return (geom, index);
            }
        } else if let Some(drag) = group_drag.as_ref() {
            if let Some((geom, index)) = self.drag_preview_derived(
                host,
                snapshot,
                DragPreviewKind::GroupDrag,
                drag.preview_rev,
                &drag.current_nodes,
            ) {
                return (geom, index);
            }
        } else if let Some(resize) = node_resize.as_ref() {
            if let Some((geom, index)) = self.node_resize_preview_derived(
                host,
                snapshot,
                resize.preview_rev,
                resize.node,
                resize.current_node_pos,
                resize.current_size_opt,
            ) {
                return (geom, index);
            }
        } else {
            self.geometry.drag_preview = None;
        }

        (geom, index)
    }

    pub(in super::super) fn update_ports_for_node_rect_change(
        geom: &mut CanvasGeometry,
        index: &mut CanvasSpatialIndex,
        node_id: GraphNodeId,
        prev_rect: Rect,
        next_rect: Rect,
        ports: &[PortId],
    ) {
        let eps = 1.0e-3;
        let prev_w = prev_rect.size.width.0;
        let next_w = next_rect.size.width.0;

        for port_id in ports {
            let Some(handle) = geom.ports.get_mut(port_id) else {
                continue;
            };
            if handle.node != node_id {
                continue;
            }

            let local_x = handle.center.x.0 - prev_rect.origin.x.0;
            let local_y = handle.center.y.0 - prev_rect.origin.y.0;
            let mut next_local_x = local_x;
            match handle.dir {
                PortDirection::In => {
                    if (local_x - 0.0).abs() <= eps {
                        next_local_x = 0.0;
                    }
                }
                PortDirection::Out => {
                    if (local_x - prev_w).abs() <= eps {
                        next_local_x = next_w;
                    }
                }
            }

            let center = Point::new(
                Px(next_rect.origin.x.0 + next_local_x),
                Px(next_rect.origin.y.0 + local_y),
            );
            let half_w = 0.5 * handle.bounds.size.width.0;
            let half_h = 0.5 * handle.bounds.size.height.0;
            let bounds = Rect::new(
                Point::new(Px(center.x.0 - half_w), Px(center.y.0 - half_h)),
                handle.bounds.size,
            );
            handle.center = center;
            handle.bounds = bounds;
            index.update_port_rect(*port_id, bounds);
        }
    }

    pub(in super::super) fn update_edges_for_ports(
        geom: &mut CanvasGeometry,
        index: &mut CanvasSpatialIndex,
        zoom: f32,
        ports: &[PortId],
        resolve_edges: impl FnOnce(&[EdgeId]) -> Vec<(EdgeId, PortId, PortId)>,
    ) {
        let mut edge_ids: Vec<EdgeId> = Vec::new();
        let edge_ids = Self::collect_edges_for_ports_sorted_dedup(index, ports, &mut edge_ids);
        if edge_ids.is_empty() {
            return;
        }

        let endpoints = resolve_edges(edge_ids);
        for (edge_id, from, to) in endpoints {
            let Some(p0) = geom.port_center(from) else {
                continue;
            };
            let Some(p1) = geom.port_center(to) else {
                continue;
            };
            let rect = index.edge_aabb(p0, p1, zoom);
            index.update_edge_rect(edge_id, rect);
        }
    }

    fn collect_edges_for_ports_sorted_dedup<'a>(
        index: &CanvasSpatialIndex,
        ports: &[PortId],
        out: &'a mut Vec<EdgeId>,
    ) -> &'a [EdgeId] {
        out.clear();
        for port in ports {
            if let Some(edges) = index.edges_for_port(*port) {
                out.extend(edges.iter().copied());
            }
        }

        out.sort_unstable();
        out.dedup();
        out.as_slice()
    }
}
