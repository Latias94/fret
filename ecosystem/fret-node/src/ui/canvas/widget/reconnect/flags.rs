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
mod tests {
    use super::*;
    use crate::core::{
        Edge, EdgeKind, EdgeReconnectable, EdgeReconnectableEndpoint, GraphId, PortId,
    };

    fn edge(reconnectable: Option<EdgeReconnectable>) -> Edge {
        Edge {
            kind: EdgeKind::Data,
            from: PortId::from_u128(1),
            to: PortId::from_u128(2),
            selectable: None,
            deletable: None,
            reconnectable,
        }
    }

    #[test]
    fn edge_reconnectable_flags_respect_edge_override_or_interaction_default() {
        let mut interaction = NodeGraphInteractionState::default();
        interaction.edges_reconnectable = false;

        assert_eq!(
            NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::edge_reconnectable_flags(
                &edge(None),
                &interaction,
            ),
            (false, false)
        );
        assert_eq!(
            NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::edge_reconnectable_flags(
                &edge(Some(EdgeReconnectable::Bool(true))),
                &interaction,
            ),
            (true, true)
        );
        assert_eq!(
            NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::edge_reconnectable_flags(
                &edge(Some(EdgeReconnectable::Endpoint(
                    EdgeReconnectableEndpoint::Source,
                ))),
                &interaction,
            ),
            (true, false)
        );
    }

    #[test]
    fn edge_endpoint_is_reconnectable_checks_endpoint_specific_flags() {
        let mut interaction = NodeGraphInteractionState::default();
        interaction.edges_reconnectable = false;

        let mut graph = Graph::new(GraphId::from_u128(1));
        let edge_id = EdgeId::from_u128(3);
        graph.edges.insert(
            edge_id,
            edge(Some(EdgeReconnectable::Endpoint(
                EdgeReconnectableEndpoint::Target,
            ))),
        );

        assert!(
            !NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::edge_endpoint_is_reconnectable(
                &graph,
                &interaction,
                edge_id,
                EdgeEndpoint::From,
            )
        );
        assert!(
            NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::edge_endpoint_is_reconnectable(
                &graph,
                &interaction,
                edge_id,
                EdgeEndpoint::To,
            )
        );
    }
}
