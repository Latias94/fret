use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn yank_edges_from_port(
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

    pub(super) fn yank_reconnectable_edges_from_port<H: UiHost>(
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

    pub(super) fn edge_reconnectable_flags(
        edge: &crate::core::Edge,
        interaction: &NodeGraphInteractionState,
    ) -> (bool, bool) {
        match edge.reconnectable {
            Some(crate::core::EdgeReconnectable::Bool(false)) => (false, false),
            Some(crate::core::EdgeReconnectable::Bool(true)) => (true, true),
            Some(crate::core::EdgeReconnectable::Endpoint(
                crate::core::EdgeReconnectableEndpoint::Source,
            )) => (true, false),
            Some(crate::core::EdgeReconnectable::Endpoint(
                crate::core::EdgeReconnectableEndpoint::Target,
            )) => (false, true),
            None => {
                let allow = interaction.edges_reconnectable;
                (allow, allow)
            }
        }
    }

    pub(super) fn edge_endpoint_is_reconnectable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        edge: EdgeId,
        endpoint: EdgeEndpoint,
    ) -> bool {
        let Some(edge) = graph.edges.get(&edge) else {
            return false;
        };
        let (allow_source, allow_target) = Self::edge_reconnectable_flags(edge, interaction);
        match endpoint {
            EdgeEndpoint::From => allow_source,
            EdgeEndpoint::To => allow_target,
        }
    }
}
