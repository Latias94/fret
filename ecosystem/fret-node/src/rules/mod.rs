//! Constraint evaluation, connection planning, and diagnostics.
//!
//! This module is intentionally small in v1: the contracts are more important than the algorithms.

use serde::{Deserialize, Serialize};

use crate::core::{
    Edge, EdgeId, EdgeKind, Graph, NodeId, PortCapacity, PortDirection, PortId, PortKind, SymbolId,
};
use crate::ops::{EdgeEndpoints, GraphOp};

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    /// Informational note.
    Info,
    /// Warning.
    Warning,
    /// Error.
    Error,
}

/// Diagnostic target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DiagnosticTarget {
    /// Graph-level diagnostic.
    Graph,
    /// Node-level diagnostic.
    Node { id: NodeId },
    /// Port-level diagnostic.
    Port { id: PortId },
    /// Edge-level diagnostic.
    Edge { id: EdgeId },
    /// Symbol-level diagnostic.
    Symbol { id: SymbolId },
}

/// A diagnostic produced by validation or connection planning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Stable machine key (for filtering/suppression).
    pub key: String,
    /// Severity.
    pub severity: DiagnosticSeverity,
    /// Target element.
    pub target: DiagnosticTarget,
    /// Human-readable message.
    pub message: String,
    /// Optional suggested fixes expressed as graph ops.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixes: Vec<GraphOp>,
}

/// Connection decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectDecision {
    /// Accept the connection.
    Accept,
    /// Reject the connection.
    Reject,
}

/// Which endpoint of an existing edge is being reconnected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeEndpoint {
    /// The source endpoint (`edge.from`).
    From,
    /// The target endpoint (`edge.to`).
    To,
}

/// A rules-driven plan for connecting two ports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectPlan {
    /// Decision.
    pub decision: ConnectDecision,
    /// Diagnostics explaining the decision.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<Diagnostic>,
    /// Optional edits to apply if accepted (disconnect existing edges, insert conversion nodes, etc.).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ops: Vec<GraphOp>,
}

impl ConnectPlan {
    /// Creates an accepted plan with no side effects.
    pub fn accept() -> Self {
        Self {
            decision: ConnectDecision::Accept,
            diagnostics: Vec::new(),
            ops: Vec::new(),
        }
    }

    /// Creates a rejected plan with a single error diagnostic.
    pub fn reject(message: impl Into<String>) -> Self {
        Self {
            decision: ConnectDecision::Reject,
            diagnostics: vec![Diagnostic {
                key: "connect.rejected".to_string(),
                severity: DiagnosticSeverity::Error,
                target: DiagnosticTarget::Graph,
                message: message.into(),
                fixes: Vec::new(),
            }],
            ops: Vec::new(),
        }
    }
}

/// Plans connecting two ports.
///
/// This is a rules-driven decision point used by the UI interaction loop.
/// The returned ops are intended to be applied as part of a single transaction.
pub fn plan_connect(graph: &Graph, a: PortId, b: PortId) -> ConnectPlan {
    let Some(port_a) = graph.ports.get(&a) else {
        return ConnectPlan::reject(format!("missing port: {a:?}"));
    };
    let Some(port_b) = graph.ports.get(&b) else {
        return ConnectPlan::reject(format!("missing port: {b:?}"));
    };

    let (from_id, to_id, from, to) = match (port_a.dir, port_b.dir) {
        (PortDirection::Out, PortDirection::In) => (a, b, port_a, port_b),
        (PortDirection::In, PortDirection::Out) => (b, a, port_b, port_a),
        _ => {
            return ConnectPlan::reject("ports must have opposite directions (in/out)");
        }
    };

    if from.node == to.node {
        return ConnectPlan::reject("cannot connect ports on the same node");
    }

    if from.kind != to.kind {
        return ConnectPlan::reject(format!(
            "port kinds are incompatible: from={:?} to={:?}",
            from.kind, to.kind
        ));
    }

    let edge_kind = match (from.kind, to.kind) {
        (PortKind::Data, PortKind::Data) => EdgeKind::Data,
        (PortKind::Exec, PortKind::Exec) => EdgeKind::Exec,
        _ => {
            return ConnectPlan::reject("port kinds are incompatible");
        }
    };

    for edge in graph.edges.values() {
        if edge.kind == edge_kind && edge.from == from_id && edge.to == to_id {
            return ConnectPlan::accept();
        }
    }

    let mut ops: Vec<GraphOp> = Vec::new();

    if from.capacity == PortCapacity::Single {
        for (edge_id, edge) in graph.edges.iter() {
            if edge.kind == edge_kind && edge.from == from_id {
                ops.push(GraphOp::RemoveEdge {
                    id: *edge_id,
                    edge: edge.clone(),
                });
            }
        }
    }

    if to.capacity == PortCapacity::Single {
        for (edge_id, edge) in graph.edges.iter() {
            if edge.kind == edge_kind && edge.to == to_id {
                ops.push(GraphOp::RemoveEdge {
                    id: *edge_id,
                    edge: edge.clone(),
                });
            }
        }
    }

    ops.push(GraphOp::AddEdge {
        id: EdgeId::new(),
        edge: Edge {
            kind: edge_kind,
            from: from_id,
            to: to_id,
        },
    });

    ConnectPlan {
        decision: ConnectDecision::Accept,
        diagnostics: Vec::new(),
        ops,
    }
}

/// Plans reconnecting one endpoint of an existing edge to a new port.
///
/// This is used for "yank and reattach" workflows where edge identity should be preserved.
pub fn plan_reconnect_edge(
    graph: &Graph,
    edge_id: EdgeId,
    endpoint: EdgeEndpoint,
    new_port: PortId,
) -> ConnectPlan {
    let Some(edge) = graph.edges.get(&edge_id) else {
        return ConnectPlan::reject(format!("missing edge: {edge_id:?}"));
    };

    let old = EdgeEndpoints {
        from: edge.from,
        to: edge.to,
    };

    let (candidate_from, candidate_to) = match endpoint {
        EdgeEndpoint::From => (new_port, edge.to),
        EdgeEndpoint::To => (edge.from, new_port),
    };

    if candidate_from == old.from && candidate_to == old.to {
        return ConnectPlan::accept();
    }

    let Some(from) = graph.ports.get(&candidate_from) else {
        return ConnectPlan::reject(format!("missing port: {candidate_from:?}"));
    };
    let Some(to) = graph.ports.get(&candidate_to) else {
        return ConnectPlan::reject(format!("missing port: {candidate_to:?}"));
    };

    if from.dir != PortDirection::Out || to.dir != PortDirection::In {
        return ConnectPlan::reject("ports must be out -> in for reconnection");
    }

    if from.node == to.node {
        return ConnectPlan::reject("cannot connect ports on the same node");
    }

    if from.kind != to.kind {
        return ConnectPlan::reject(format!(
            "port kinds are incompatible: from={:?} to={:?}",
            from.kind, to.kind
        ));
    }

    let expected_edge_kind = match (from.kind, to.kind) {
        (PortKind::Data, PortKind::Data) => EdgeKind::Data,
        (PortKind::Exec, PortKind::Exec) => EdgeKind::Exec,
        _ => {
            return ConnectPlan::reject("port kinds are incompatible");
        }
    };

    if edge.kind != expected_edge_kind {
        return ConnectPlan::reject(format!(
            "edge kind is incompatible with ports: edge={:?} expected={:?}",
            edge.kind, expected_edge_kind
        ));
    }

    for (other_id, other) in &graph.edges {
        if *other_id == edge_id {
            continue;
        }
        if other.kind == edge.kind && other.from == candidate_from && other.to == candidate_to {
            return ConnectPlan::reject("duplicate connection already exists");
        }
    }

    let mut ops: Vec<GraphOp> = Vec::new();

    if from.capacity == PortCapacity::Single {
        for (other_id, other) in &graph.edges {
            if *other_id == edge_id {
                continue;
            }
            if other.kind == edge.kind && other.from == candidate_from {
                ops.push(GraphOp::RemoveEdge {
                    id: *other_id,
                    edge: other.clone(),
                });
            }
        }
    }

    if to.capacity == PortCapacity::Single {
        for (other_id, other) in &graph.edges {
            if *other_id == edge_id {
                continue;
            }
            if other.kind == edge.kind && other.to == candidate_to {
                ops.push(GraphOp::RemoveEdge {
                    id: *other_id,
                    edge: other.clone(),
                });
            }
        }
    }

    ops.push(GraphOp::SetEdgeEndpoints {
        id: edge_id,
        from: old,
        to: EdgeEndpoints {
            from: candidate_from,
            to: candidate_to,
        },
    });

    ConnectPlan {
        decision: ConnectDecision::Accept,
        diagnostics: Vec::new(),
        ops,
    }
}

#[cfg(test)]
mod tests;
