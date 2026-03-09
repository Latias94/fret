use super::*;

impl CanvasSpatialDerived {
    pub(crate) fn empty() -> Self {
        Self {
            index: CanvasSpatialIndex::empty(1.0),
            port_edges: CanvasPortEdgeAdjacency::empty(),
            edge_aabb_pad_canvas: 0.0,
        }
    }

    pub(crate) fn build(
        graph: &Graph,
        geom: &CanvasGeometry,
        zoom: f32,
        max_hit_pad_canvas: f32,
        cell_size_canvas: f32,
    ) -> Self {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        let cell_size = if cell_size_canvas.is_finite() && cell_size_canvas > 0.0 {
            cell_size_canvas
        } else {
            (256.0 / zoom).max(16.0 / zoom).max(1.0)
        };

        let pad = max_hit_pad_canvas.max(0.0);
        let mut index = CanvasSpatialIndex::build_from_geometry(graph, geom, cell_size);
        for (&edge_id, edge) in &graph.edges {
            let Some(from) = geom.port_center(edge.from) else {
                continue;
            };
            let Some(to) = geom.port_center(edge.to) else {
                continue;
            };
            let rect = canvas_wires::wire_aabb(from, to, zoom, pad);
            index.update_edge_rect(edge_id, rect);
        }

        Self {
            index,
            port_edges: CanvasPortEdgeAdjacency::build(graph),
            edge_aabb_pad_canvas: pad,
        }
    }

    pub(crate) fn edge_aabb(&self, from: Point, to: Point, zoom: f32) -> Rect {
        canvas_wires::wire_aabb(from, to, zoom, self.edge_aabb_pad_canvas)
    }

    pub(crate) fn edges_for_port(&self, port: PortId) -> Option<&[EdgeId]> {
        self.port_edges.edges_for_port(port)
    }

    pub(crate) fn query_ports_sorted_dedup<'a>(
        &self,
        pos: Point,
        radius: f32,
        out: &'a mut Vec<PortId>,
    ) -> &'a [PortId] {
        self.index.query_ports_sorted_dedup(pos, radius, out)
    }

    pub(crate) fn query_edges_sorted_dedup<'a>(
        &self,
        pos: Point,
        radius: f32,
        out: &'a mut Vec<EdgeId>,
    ) -> &'a [EdgeId] {
        self.index.query_edges_sorted_dedup(pos, radius, out)
    }

    pub(crate) fn query_edges_in_rect(&self, rect: Rect, out: &mut Vec<EdgeId>) {
        self.index.query_edges_in_rect(rect, out);
    }

    pub(crate) fn query_nodes_in_rect(&self, rect: Rect, out: &mut Vec<NodeId>) {
        self.index.query_nodes_in_rect(rect, out);
    }

    pub(crate) fn update_node_rect(&mut self, node: NodeId, rect: Rect) {
        self.index.update_node_rect(node, rect);
    }

    pub(crate) fn update_port_rect(&mut self, port: PortId, rect: Rect) {
        self.index.update_port_rect(port, rect);
    }

    pub(crate) fn update_edge_rect(&mut self, edge: EdgeId, rect: Rect) {
        self.index.update_edge_rect(edge, rect);
    }
}
