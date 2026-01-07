//! Domain specializations for node graphs.
//!
//! A profile selects:
//! - how port types are resolved (schema vs inferred vs domain-owned),
//! - which compatibility rules apply,
//! - how concretization (dynamic ports) is scheduled,
//! - how validation diagnostics are produced.
//!
//! This module is intentionally headless (no `fret-ui` dependency).

mod pipeline;
mod simple;

use crate::core::{EdgeKind, Graph, PortId};
use crate::rules::{ConnectPlan, Diagnostic};
use crate::types::TypeDesc;

pub use pipeline::{
    ApplyPipelineError, apply_connect_plan_with_profile, apply_transaction_with_profile,
};
pub use simple::DataflowProfile;

/// Profile hooks for typed graphs and domain specialization.
pub trait GraphProfile {
    /// Returns the current type of a port.
    ///
    /// Default implementations may read `Port::ty` and/or derive from node payloads.
    fn type_of_port(&mut self, graph: &Graph, port: PortId) -> Option<TypeDesc>;

    /// Plans connecting two ports under this profile.
    ///
    /// Implementations should call into `crate::rules` and then enforce extra constraints
    /// (type compatibility, cycle policy, exec/data policy, etc.).
    fn plan_connect(&mut self, graph: &Graph, a: PortId, b: PortId) -> ConnectPlan;

    /// Validates a graph and returns diagnostics.
    fn validate_graph(&mut self, graph: &Graph) -> Vec<Diagnostic>;

    /// Whether the profile allows cycles for the given edge kind.
    fn allow_cycles(&self, _edge_kind: EdgeKind) -> bool {
        true
    }

    /// Maximum number of concretization iterations per edit.
    ///
    /// This bounds fixed-point scheduling to prevent oscillation.
    fn concretize_bound(&self) -> usize {
        8
    }

    /// Runs concretization for dynamic ports and returns additional ops to apply.
    ///
    /// The returned ops must be deterministic and undoable.
    fn concretize(&mut self, _graph: &Graph) -> Vec<crate::ops::GraphOp> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests;
