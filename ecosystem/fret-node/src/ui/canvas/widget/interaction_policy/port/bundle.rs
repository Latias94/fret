use super::super::super::*;

fn should_add_bundle_port_with_graph(
    graph: &Graph,
    from: PortId,
    bundle: &[PortId],
    candidate: PortId,
) -> bool {
    if candidate == from || bundle.contains(&candidate) {
        return false;
    }
    let Some(from_port) = graph.ports.get(&from) else {
        return false;
    };
    let Some(candidate_port) = graph.ports.get(&candidate) else {
        return false;
    };
    candidate_port.dir == from_port.dir
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super::super) fn should_add_bundle_port(
        graph: &Graph,
        from: PortId,
        bundle: &[PortId],
        candidate: PortId,
    ) -> bool {
        should_add_bundle_port_with_graph(graph, from, bundle, candidate)
    }
}
