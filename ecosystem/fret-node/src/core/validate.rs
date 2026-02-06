use std::collections::{BTreeMap, BTreeSet};

use crate::core::{
    EdgeId, EdgeKind, Graph, GraphId, GroupId, NodeId, PortCapacity, PortId, PortKind,
    SubgraphNodeError, SymbolId, SymbolRefNodeError, subgraph_target_graph_id,
    symbol_ref_target_symbol_id,
};

#[derive(Debug, thiserror::Error)]
pub enum GraphValidationError {
    #[error("graph version mismatch: expected={expected} found={found}")]
    UnsupportedGraphVersion { expected: u32, found: u32 },

    #[error("port references missing node: port={port:?} node={node:?}")]
    PortMissingNode { port: PortId, node: NodeId },

    #[error("node parent references missing group: node={node:?} group={group:?}")]
    NodeParentMissingGroup { node: NodeId, group: GroupId },

    #[error("node has invalid size: node={node:?} width={width} height={height}")]
    NodeInvalidSize {
        node: NodeId,
        width: f32,
        height: f32,
    },

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

    #[error(
        "edge port kinds are incompatible: edge={edge:?} from_kind={from_kind:?} to_kind={to_kind:?}"
    )]
    EdgeKindMismatch {
        edge: EdgeId,
        from_kind: PortKind,
        to_kind: PortKind,
    },

    #[error(
        "edge kind does not match port kind: edge={edge:?} edge_kind={edge_kind:?} port_kind={port_kind:?}"
    )]
    EdgeKindPortKindMismatch {
        edge: EdgeId,
        edge_kind: EdgeKind,
        port_kind: PortKind,
    },

    #[error("edge duplicates an existing connection: edge={edge:?}")]
    DuplicateEdge { edge: EdgeId },

    #[error("port capacity exceeded: port={port:?} capacity={capacity:?} count={count}")]
    PortCapacityExceeded {
        port: PortId,
        capacity: PortCapacity,
        count: usize,
    },

    #[error("subgraph node missing graph_id: node={node:?}")]
    SubgraphNodeMissingGraphId { node: NodeId },

    #[error("subgraph node graph_id is not a string: node={node:?}")]
    SubgraphNodeGraphIdNotString { node: NodeId },

    #[error("subgraph node graph_id is not a valid uuid: node={node:?} value={value:?}")]
    SubgraphNodeInvalidGraphId { node: NodeId, value: String },

    #[error(
        "subgraph node target graph is not declared in imports: node={node:?} graph_id={graph_id}"
    )]
    SubgraphTargetNotImported { node: NodeId, graph_id: GraphId },

    #[error("symbol ref node missing symbol_id: node={node:?}")]
    SymbolRefNodeMissingSymbolId { node: NodeId },

    #[error("symbol ref node symbol_id is not a string: node={node:?}")]
    SymbolRefNodeSymbolIdNotString { node: NodeId },

    #[error("symbol ref node symbol_id is not a valid uuid: node={node:?} value={value:?}")]
    SymbolRefNodeInvalidSymbolId { node: NodeId, value: String },

    #[error(
        "symbol ref node target symbol is not declared in symbols: node={node:?} symbol_id={symbol_id:?}"
    )]
    SymbolRefTargetNotDeclared { node: NodeId, symbol_id: SymbolId },
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
    validate_graph_structural(graph)
}

/// Validates a graph for structural consistency (contract-level invariants).
///
/// This intentionally does **not** enforce editor policies such as connection direction.
/// Direction, cycle policy, and domain-specific semantics belong in profiles/rules.
pub fn validate_graph_structural(graph: &Graph) -> GraphValidationReport {
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
        if let Some(group) = node.parent
            && !graph.groups.contains_key(&group)
        {
            report
                .errors
                .push(GraphValidationError::NodeParentMissingGroup {
                    node: *node_id,
                    group,
                });
        }

        if let Some(size) = node.size {
            if !size.width.is_finite()
                || !size.height.is_finite()
                || size.width <= 0.0
                || size.height <= 0.0
            {
                report.errors.push(GraphValidationError::NodeInvalidSize {
                    node: *node_id,
                    width: size.width,
                    height: size.height,
                });
            }
        }

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

    for (node_id, node) in &graph.nodes {
        match subgraph_target_graph_id(*node_id, node) {
            Ok(Some(target)) => {
                if !graph.imports.contains_key(&target) {
                    report
                        .errors
                        .push(GraphValidationError::SubgraphTargetNotImported {
                            node: *node_id,
                            graph_id: target,
                        });
                }
            }
            Ok(None) => {}
            Err(err) => match err {
                SubgraphNodeError::MissingGraphId { node } => {
                    report
                        .errors
                        .push(GraphValidationError::SubgraphNodeMissingGraphId { node });
                }
                SubgraphNodeError::GraphIdNotString { node } => {
                    report
                        .errors
                        .push(GraphValidationError::SubgraphNodeGraphIdNotString { node });
                }
                SubgraphNodeError::InvalidGraphId { node, value } => {
                    report
                        .errors
                        .push(GraphValidationError::SubgraphNodeInvalidGraphId { node, value });
                }
            },
        }

        match symbol_ref_target_symbol_id(*node_id, node) {
            Ok(Some(target)) => {
                if !graph.symbols.contains_key(&target) {
                    report
                        .errors
                        .push(GraphValidationError::SymbolRefTargetNotDeclared {
                            node: *node_id,
                            symbol_id: target,
                        });
                }
            }
            Ok(None) => {}
            Err(err) => match err {
                SymbolRefNodeError::MissingSymbolId { node } => {
                    report
                        .errors
                        .push(GraphValidationError::SymbolRefNodeMissingSymbolId { node });
                }
                SymbolRefNodeError::SymbolIdNotString { node } => {
                    report
                        .errors
                        .push(GraphValidationError::SymbolRefNodeSymbolIdNotString { node });
                }
                SymbolRefNodeError::InvalidSymbolId { node, value } => {
                    report
                        .errors
                        .push(GraphValidationError::SymbolRefNodeInvalidSymbolId { node, value });
                }
            },
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

        if from.kind != to.kind {
            report.errors.push(GraphValidationError::EdgeKindMismatch {
                edge: *edge_id,
                from_kind: from.kind,
                to_kind: to.kind,
            });
        } else {
            let expected = match from.kind {
                PortKind::Data => EdgeKind::Data,
                PortKind::Exec => EdgeKind::Exec,
            };
            if edge.kind != expected {
                report
                    .errors
                    .push(GraphValidationError::EdgeKindPortKindMismatch {
                        edge: *edge_id,
                        edge_kind: edge.kind,
                        port_kind: from.kind,
                    });
            }
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
