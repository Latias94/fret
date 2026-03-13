use super::super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn yank_edges_from_port(
        graph: &Graph,
        port: PortId,
    ) -> Vec<(EdgeId, EdgeEndpoint, PortId)> {
        let Some(p) = graph.ports.get(&port) else {
            return Vec::new();
        };

        let mut out: Vec<(EdgeId, EdgeEndpoint, PortId)> = Vec::new();
        match p.dir {
            PortDirection::Out => {
                for (edge_id, edge) in &graph.edges {
                    if edge.from == port {
                        out.push((*edge_id, EdgeEndpoint::From, edge.to));
                    }
                }
            }
            PortDirection::In => {
                for (edge_id, edge) in &graph.edges {
                    if edge.to == port {
                        out.push((*edge_id, EdgeEndpoint::To, edge.from));
                    }
                }
            }
        }
        out
    }

    pub(in super::super) fn yank_reconnectable_edges_from_port<H: UiHost>(
        &self,
        host: &mut H,
        interaction: &NodeGraphInteractionState,
        port: PortId,
    ) -> Vec<(EdgeId, EdgeEndpoint, PortId)> {
        if let Some(store) = self.store.as_ref() {
            if let Ok(out) = store.read_ref(host, |s| {
                use crate::runtime::lookups::ConnectionSide;

                let graph = s.graph();
                let Some(p) = graph.ports.get(&port) else {
                    return Vec::new();
                };

                let node = p.node;
                let side = ConnectionSide::from_port_dir(p.dir);
                let mut out: Vec<(EdgeId, EdgeEndpoint, PortId)> = Vec::new();

                let Some(conns) = s.lookups().connections_for_port(node, side, port) else {
                    return out;
                };

                for (edge_id, conn) in conns {
                    let (endpoint, fixed) = match p.dir {
                        PortDirection::Out => (EdgeEndpoint::From, conn.target_port),
                        PortDirection::In => (EdgeEndpoint::To, conn.source_port),
                    };
                    if Self::edge_endpoint_is_reconnectable(graph, interaction, *edge_id, endpoint)
                    {
                        out.push((*edge_id, endpoint, fixed));
                    }
                }

                out.sort_by_key(|(edge_id, _, _)| *edge_id);
                out
            }) {
                return out;
            }
        }

        self.graph
            .read_ref(host, |graph| {
                let mut edges = Self::yank_edges_from_port(graph, port);
                edges.retain(|(edge_id, endpoint, _fixed)| {
                    Self::edge_endpoint_is_reconnectable(graph, interaction, *edge_id, *endpoint)
                });
                edges
            })
            .ok()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests;
