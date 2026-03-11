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
mod tests {
    use super::*;
    use crate::core::{
        Edge, EdgeKind, GraphId, NodeId as GraphNodeId, Port, PortCapacity, PortDirection, PortKey,
        PortKind,
    };
    use serde_json::Value;

    fn sample_graph() -> (Graph, PortId, PortId, PortId, EdgeId, EdgeId) {
        let graph_id = GraphId::from_u128(1);
        let source_node = GraphNodeId::from_u128(10);
        let target_node = GraphNodeId::from_u128(11);
        let other_node = GraphNodeId::from_u128(12);
        let source_port = PortId::from_u128(20);
        let target_port = PortId::from_u128(21);
        let other_port = PortId::from_u128(22);
        let outgoing_edge = EdgeId::from_u128(30);
        let incoming_edge = EdgeId::from_u128(31);

        let mut graph = Graph::new(graph_id);
        graph.ports.insert(
            source_port,
            Port {
                node: source_node,
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
        graph.ports.insert(
            target_port,
            Port {
                node: target_node,
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
        graph.ports.insert(
            other_port,
            Port {
                node: other_node,
                key: PortKey::new("other"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
        graph.edges.insert(
            outgoing_edge,
            Edge {
                kind: EdgeKind::Data,
                from: source_port,
                to: target_port,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
        graph.edges.insert(
            incoming_edge,
            Edge {
                kind: EdgeKind::Data,
                from: source_port,
                to: other_port,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );

        (
            graph,
            source_port,
            target_port,
            other_port,
            outgoing_edge,
            incoming_edge,
        )
    }

    #[test]
    fn yank_edges_from_out_port_marks_from_endpoint_and_fixed_peer() {
        let (graph, source_port, target_port, other_port, outgoing_edge, incoming_edge) =
            sample_graph();

        let edges = NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::yank_edges_from_port(
            &graph,
            source_port,
        );

        assert_eq!(
            edges,
            vec![
                (outgoing_edge, EdgeEndpoint::From, target_port),
                (incoming_edge, EdgeEndpoint::From, other_port),
            ]
        );
    }

    #[test]
    fn yank_edges_from_in_port_marks_to_endpoint_and_fixed_peer() {
        let (graph, source_port, target_port, _other_port, outgoing_edge, _incoming_edge) =
            sample_graph();

        let edges = NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::yank_edges_from_port(
            &graph,
            target_port,
        );

        assert_eq!(edges, vec![(outgoing_edge, EdgeEndpoint::To, source_port)]);
    }
}
