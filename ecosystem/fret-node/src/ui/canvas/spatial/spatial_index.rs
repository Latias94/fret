use super::*;

impl CanvasSpatialIndex {
    pub(crate) fn empty(cell_size_canvas: f32) -> Self {
        let cell_size_canvas = if cell_size_canvas.is_finite() && cell_size_canvas > 0.0 {
            cell_size_canvas
        } else {
            1.0
        };
        Self {
            nodes: DefaultIndexWithBackrefs::new(cell_size_canvas),
            ports: DefaultIndexWithBackrefs::new(cell_size_canvas),
            edges: DefaultIndexWithBackrefs::new(cell_size_canvas),
        }
    }

    pub(crate) fn build_from_geometry(
        graph: &Graph,
        geom: &CanvasGeometry,
        cell_size_canvas: f32,
    ) -> Self {
        let mut index = Self::empty(cell_size_canvas);

        for node_id in geom.order.iter().copied() {
            let Some(node_geom) = geom.nodes.get(&node_id) else {
                continue;
            };
            index.nodes.insert_rect(node_id, node_geom.rect);
        }

        for node_id in geom.order.iter().copied() {
            let Some(node) = graph.nodes.get(&node_id) else {
                continue;
            };
            for port_id in node.ports.iter().copied() {
                let Some(handle) = geom.ports.get(&port_id) else {
                    continue;
                };
                index.ports.insert_rect(port_id, handle.bounds);
            }
        }

        index
    }

    pub(crate) fn query_ports_sorted_dedup<'a>(
        &self,
        pos: Point,
        radius: f32,
        out: &'a mut Vec<PortId>,
    ) -> &'a [PortId] {
        self.ports.query_radius_sorted_dedup(pos, radius, out)
    }

    pub(crate) fn query_edges_sorted_dedup<'a>(
        &self,
        pos: Point,
        radius: f32,
        out: &'a mut Vec<EdgeId>,
    ) -> &'a [EdgeId] {
        self.edges.query_radius_sorted_dedup(pos, radius, out)
    }

    pub(crate) fn query_edges_in_rect(&self, rect: Rect, out: &mut Vec<EdgeId>) {
        let _ = self.edges.query_rect_sorted_dedup(rect, out);
    }

    pub(crate) fn query_nodes_in_rect(&self, rect: Rect, out: &mut Vec<NodeId>) {
        let _ = self.nodes.query_rect_sorted_dedup(rect, out);
    }

    pub(crate) fn update_node_rect(&mut self, node: NodeId, rect: Rect) {
        self.nodes.update_rect(node, rect);
    }

    pub(crate) fn update_port_rect(&mut self, port: PortId, rect: Rect) {
        self.ports.update_rect(port, rect);
    }

    pub(crate) fn update_edge_rect(&mut self, edge: EdgeId, rect: Rect) {
        self.edges.update_rect(edge, rect);
    }
}
