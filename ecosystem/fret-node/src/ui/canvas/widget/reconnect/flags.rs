use super::super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn edge_reconnectable_flags(
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

    pub(in super::super) fn edge_endpoint_is_reconnectable(
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

#[cfg(test)]
mod tests;
