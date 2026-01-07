//! Simple built-in profiles.

use crate::core::{Graph, PortId};
use crate::rules::{ConnectPlan, Diagnostic, DiagnosticSeverity, plan_connect_typed};
use crate::types::{DefaultTypeCompatibility, TypeCompatibility, TypeDesc};

use super::GraphProfile;

/// A permissive dataflow profile:
/// - allows both data and exec edges,
/// - uses `Port::ty` as the source of truth for typing,
/// - enforces a small default compatibility table for data edges when both sides have types.
#[derive(Debug, Default, Clone)]
pub struct DataflowProfile {
    compat: DefaultTypeCompatibility,
}

impl DataflowProfile {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_compat(mut self, compat: DefaultTypeCompatibility) -> Self {
        self.compat = compat;
        self
    }

    pub fn compat_mut(&mut self) -> &mut dyn TypeCompatibility {
        &mut self.compat
    }
}

impl GraphProfile for DataflowProfile {
    fn type_of_port(&mut self, graph: &Graph, port: PortId) -> Option<TypeDesc> {
        graph.ports.get(&port).and_then(|p| p.ty.clone())
    }

    fn plan_connect(&mut self, graph: &Graph, a: PortId, b: PortId) -> ConnectPlan {
        plan_connect_typed(
            graph,
            a,
            b,
            |g, p| g.ports.get(&p).and_then(|p| p.ty.clone()),
            &mut self.compat,
        )
    }

    fn validate_graph(&mut self, graph: &Graph) -> Vec<Diagnostic> {
        let report = crate::core::validate_graph(graph);
        report
            .errors
            .into_iter()
            .map(|err| Diagnostic {
                key: "graph.invalid".to_string(),
                severity: DiagnosticSeverity::Error,
                target: crate::rules::DiagnosticTarget::Graph,
                message: err.to_string(),
                fixes: Vec::new(),
            })
            .collect()
    }
}
