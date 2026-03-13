use super::*;

impl CanvasPortEdgeAdjacency {
    pub(crate) fn empty() -> Self {
        Self {
            edges_by_port: HashMap::new(),
        }
    }

    pub(crate) fn build(graph: &Graph) -> Self {
        let mut edges_by_port: HashMap<PortId, Vec<EdgeId>> = HashMap::new();
        for (&edge_id, edge) in &graph.edges {
            if edge.from == edge.to {
                edges_by_port.entry(edge.from).or_default().push(edge_id);
            } else {
                edges_by_port.entry(edge.from).or_default().push(edge_id);
                edges_by_port.entry(edge.to).or_default().push(edge_id);
            }
        }
        Self { edges_by_port }
    }

    pub(crate) fn edges_for_port(&self, port: PortId) -> Option<&[EdgeId]> {
        self.edges_by_port.get(&port).map(|edges| edges.as_slice())
    }
}
