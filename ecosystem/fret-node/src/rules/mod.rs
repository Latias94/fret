//! Constraint evaluation, connection planning, and diagnostics.
//!
//! This module is intentionally small in v1: the contracts are more important than the algorithms.

use serde::{Deserialize, Serialize};

use crate::core::{EdgeId, NodeId, PortId, SymbolId};
use crate::ops::GraphOp;

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
