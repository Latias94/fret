use crate::core::{Edge, EdgeId, Graph, NodeId, Port, PortId};
use crate::ops::{GraphOp, GraphTransaction};

/// Builder helpers for constructing safe, reversible edit operations.
///
/// These helpers:
/// - snapshot the removed data needed for undo,
/// - produce ops in a valid apply order,
/// - use deterministic ordering for stability.
pub trait GraphOpBuilderExt {
    /// Builds a `RemoveEdge` op for an existing edge.
    fn build_remove_edge_op(&self, id: EdgeId) -> Option<GraphOp>;

    /// Builds a `RemovePort` op for an existing port, including incident edges.
    fn build_remove_port_op(&self, id: PortId) -> Option<GraphOp>;

    /// Builds ops that disconnect all edges incident to the port.
    ///
    /// The ops are returned in deterministic order.
    fn build_disconnect_port_ops(&self, id: PortId) -> Option<Vec<GraphOp>>;

    /// Builds a transaction that removes a port (and its incident edges).
    fn build_remove_port_tx(
        &self,
        id: PortId,
        label: impl Into<String>,
    ) -> Option<GraphTransaction>;

    /// Builds a `RemoveNode` op for an existing node, including owned ports and incident edges.
    fn build_remove_node_op(&self, id: NodeId) -> Option<GraphOp>;

    /// Builds a transaction that removes a node (and its ports/edges).
    fn build_remove_node_tx(
        &self,
        id: NodeId,
        label: impl Into<String>,
    ) -> Option<GraphTransaction>;
}

impl GraphOpBuilderExt for Graph {
    fn build_remove_edge_op(&self, id: EdgeId) -> Option<GraphOp> {
        let edge = self.edges.get(&id)?.clone();
        Some(GraphOp::RemoveEdge { id, edge })
    }

    fn build_remove_port_op(&self, id: PortId) -> Option<GraphOp> {
        let port = self.ports.get(&id)?.clone();

        let mut edges: Vec<(EdgeId, Edge)> = self
            .edges
            .iter()
            .filter_map(|(edge_id, edge)| {
                if edge.from == id || edge.to == id {
                    Some((*edge_id, edge.clone()))
                } else {
                    None
                }
            })
            .collect();

        edges.sort_by_key(|(edge_id, _)| *edge_id);

        Some(GraphOp::RemovePort { id, port, edges })
    }

    fn build_disconnect_port_ops(&self, id: PortId) -> Option<Vec<GraphOp>> {
        self.ports.get(&id)?;

        let mut ops: Vec<GraphOp> = self
            .edges
            .iter()
            .filter_map(|(edge_id, edge)| {
                if edge.from == id || edge.to == id {
                    Some(GraphOp::RemoveEdge {
                        id: *edge_id,
                        edge: edge.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        ops.sort_by_key(|op| match op {
            GraphOp::RemoveEdge { id, .. } => *id,
            _ => unreachable!(),
        });

        Some(ops)
    }

    fn build_remove_port_tx(
        &self,
        id: PortId,
        label: impl Into<String>,
    ) -> Option<GraphTransaction> {
        let op = self.build_remove_port_op(id)?;
        Some(GraphTransaction {
            label: Some(label.into()),
            ops: vec![op],
        })
    }

    fn build_remove_node_op(&self, id: NodeId) -> Option<GraphOp> {
        let node = self.nodes.get(&id)?.clone();

        let mut ports: Vec<(PortId, Port)> = self
            .ports
            .iter()
            .filter_map(|(port_id, port)| {
                if port.node == id {
                    Some((*port_id, port.clone()))
                } else {
                    None
                }
            })
            .collect();
        ports.sort_by_key(|(port_id, _)| *port_id);

        let port_ids: std::collections::BTreeSet<PortId> =
            ports.iter().map(|(port_id, _)| *port_id).collect();

        let mut edges: Vec<(EdgeId, Edge)> = self
            .edges
            .iter()
            .filter_map(|(edge_id, edge)| {
                if port_ids.contains(&edge.from) || port_ids.contains(&edge.to) {
                    Some((*edge_id, edge.clone()))
                } else {
                    None
                }
            })
            .collect();
        edges.sort_by_key(|(edge_id, _)| *edge_id);

        Some(GraphOp::RemoveNode {
            id,
            node,
            ports,
            edges,
        })
    }

    fn build_remove_node_tx(
        &self,
        id: NodeId,
        label: impl Into<String>,
    ) -> Option<GraphTransaction> {
        let op = self.build_remove_node_op(id)?;
        Some(GraphTransaction {
            label: Some(label.into()),
            ops: vec![op],
        })
    }
}
