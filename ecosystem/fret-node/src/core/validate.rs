use std::collections::{BTreeMap, BTreeSet};

use crate::core::{EdgeId, Graph, NodeId, PortCapacity, PortDirection, PortId, PortKind};

#[derive(Debug, thiserror::Error)]
pub enum GraphValidationError {
    #[error("graph version mismatch: expected={expected} found={found}")]
    UnsupportedGraphVersion { expected: u32, found: u32 },

    #[error("port references missing node: port={port:?} node={node:?}")]
    PortMissingNode { port: PortId, node: NodeId },

    #[error("node ports list references missing port: node={node:?} port={port:?}")]
    NodePortsMissingPort { node: NodeId, port: PortId },

    #[error(
        "node ports list references port owned by another node: node={node:?} port={port:?} owner={owner:?}"
    )]
    NodePortsWrongOwner {
        node: NodeId,
        port: PortId,
        owner: NodeId,
    },

    #[error("node ports list contains duplicates: node={node:?} port={port:?}")]
    NodePortsDuplicate { node: NodeId, port: PortId },

    #[error("edge references missing port: edge={edge:?} port={port:?}")]
    EdgeMissingPort { edge: EdgeId, port: PortId },

    #[error("edge connects ports on the same node: edge={edge:?} node={node:?}")]
    EdgeSameNode { edge: EdgeId, node: NodeId },

    #[error(
        "edge port directions are invalid: edge={edge:?} from_dir={from_dir:?} to_dir={to_dir:?}"
    )]
    EdgeInvalidDirection {
        edge: EdgeId,
        from_dir: PortDirection,
        to_dir: PortDirection,
    },

    #[error(
        "edge port kinds are incompatible: edge={edge:?} from_kind={from_kind:?} to_kind={to_kind:?}"
    )]
    EdgeKindMismatch {
        edge: EdgeId,
        from_kind: PortKind,
        to_kind: PortKind,
    },

    #[error("edge duplicates an existing connection: edge={edge:?}")]
    DuplicateEdge { edge: EdgeId },

    #[error("port capacity exceeded: port={port:?} capacity={capacity:?} count={count}")]
    PortCapacityExceeded {
        port: PortId,
        capacity: PortCapacity,
        count: usize,
    },
}

#[derive(Debug, Default)]
pub struct GraphValidationReport {
    pub errors: Vec<GraphValidationError>,
}

impl GraphValidationReport {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

pub fn validate_graph(graph: &Graph) -> GraphValidationReport {
    let mut report = GraphValidationReport::default();

    if graph.graph_version != crate::core::model::GRAPH_VERSION {
        report
            .errors
            .push(GraphValidationError::UnsupportedGraphVersion {
                expected: crate::core::model::GRAPH_VERSION,
                found: graph.graph_version,
            });
        return report;
    }

    for (port_id, port) in &graph.ports {
        if !graph.nodes.contains_key(&port.node) {
            report.errors.push(GraphValidationError::PortMissingNode {
                port: *port_id,
                node: port.node,
            });
        }
    }

    for (node_id, node) in &graph.nodes {
        let mut seen: BTreeSet<PortId> = BTreeSet::new();
        for port_id in &node.ports {
            if !seen.insert(*port_id) {
                report
                    .errors
                    .push(GraphValidationError::NodePortsDuplicate {
                        node: *node_id,
                        port: *port_id,
                    });
                continue;
            }
            let Some(port) = graph.ports.get(port_id) else {
                report
                    .errors
                    .push(GraphValidationError::NodePortsMissingPort {
                        node: *node_id,
                        port: *port_id,
                    });
                continue;
            };
            if port.node != *node_id {
                report
                    .errors
                    .push(GraphValidationError::NodePortsWrongOwner {
                        node: *node_id,
                        port: *port_id,
                        owner: port.node,
                    });
            }
        }
    }

    let mut edge_pairs: BTreeSet<(PortKind, PortId, PortId)> = BTreeSet::new();
    let mut incident_counts: BTreeMap<PortId, usize> = BTreeMap::new();

    for (edge_id, edge) in &graph.edges {
        let Some(from) = graph.ports.get(&edge.from) else {
            report.errors.push(GraphValidationError::EdgeMissingPort {
                edge: *edge_id,
                port: edge.from,
            });
            continue;
        };
        let Some(to) = graph.ports.get(&edge.to) else {
            report.errors.push(GraphValidationError::EdgeMissingPort {
                edge: *edge_id,
                port: edge.to,
            });
            continue;
        };

        if from.node == to.node {
            report.errors.push(GraphValidationError::EdgeSameNode {
                edge: *edge_id,
                node: from.node,
            });
        }

        if from.dir != PortDirection::Out || to.dir != PortDirection::In {
            report
                .errors
                .push(GraphValidationError::EdgeInvalidDirection {
                    edge: *edge_id,
                    from_dir: from.dir,
                    to_dir: to.dir,
                });
        }

        if from.kind != to.kind {
            report.errors.push(GraphValidationError::EdgeKindMismatch {
                edge: *edge_id,
                from_kind: from.kind,
                to_kind: to.kind,
            });
        }

        if !edge_pairs.insert((from.kind, edge.from, edge.to)) {
            report
                .errors
                .push(GraphValidationError::DuplicateEdge { edge: *edge_id });
        }

        *incident_counts.entry(edge.from).or_insert(0) += 1;
        *incident_counts.entry(edge.to).or_insert(0) += 1;
    }

    for (port_id, count) in incident_counts {
        let Some(port) = graph.ports.get(&port_id) else {
            continue;
        };
        if port.capacity == PortCapacity::Single && count > 1 {
            report
                .errors
                .push(GraphValidationError::PortCapacityExceeded {
                    port: port_id,
                    capacity: port.capacity,
                    count,
                });
        }
    }

    report
}
