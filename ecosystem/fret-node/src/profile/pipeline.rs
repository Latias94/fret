//! Edit pipeline for profiles (apply -> concretize -> validate).
//!
//! This is the headless counterpart of an editor interaction loop. UI code can drive edits by
//! producing `GraphTransaction`s, then passing them through this pipeline to get deterministic
//! derived edits (dynamic ports, autofixes) and diagnostics.

use crate::core::Graph;
use crate::ops::{ApplyError, GraphOp, GraphTransaction, apply_transaction};
use crate::rules::{ConnectDecision, ConnectPlan, Diagnostic, DiagnosticSeverity};

use super::GraphProfile;

#[derive(Debug, thiserror::Error)]
pub enum ApplyPipelineError {
    #[error("failed to apply transaction ops")]
    Apply(#[from] ApplyError),
    #[error("concretization did not converge within bound={bound}")]
    ConcretizeNonConvergent { bound: usize },
    #[error("transaction rejected by diagnostics: {message}")]
    Rejected {
        message: String,
        diagnostics: Vec<Diagnostic>,
    },
}

impl ApplyPipelineError {
    pub fn diagnostics(&self) -> Option<&[Diagnostic]> {
        match self {
            Self::Rejected { diagnostics, .. } => Some(diagnostics),
            _ => None,
        }
    }
}

/// Applies a transaction and runs profile-driven concretization/validation to a fixed point.
///
/// Returns the committed transaction (original ops + derived concretization ops).
pub fn apply_transaction_with_profile(
    graph: &mut Graph,
    profile: &mut dyn GraphProfile,
    tx: &GraphTransaction,
) -> Result<GraphTransaction, ApplyPipelineError> {
    let mut committed = GraphTransaction {
        label: tx.label.clone(),
        ops: Vec::new(),
    };

    apply_transaction(graph, tx)?;
    committed.ops.extend(tx.ops.clone());

    let bound = profile.concretize_bound();
    for _ in 0..bound {
        let derived_ops: Vec<GraphOp> = profile.concretize(graph);
        if derived_ops.is_empty() {
            let diagnostics = profile.validate_graph(graph);
            let rejected = diagnostics
                .iter()
                .any(|d| d.severity == DiagnosticSeverity::Error);
            if rejected {
                let message = diagnostics
                    .first()
                    .map(|d| d.message.clone())
                    .unwrap_or_else(|| "transaction rejected".to_string());
                return Err(ApplyPipelineError::Rejected {
                    message,
                    diagnostics,
                });
            }
            return Ok(committed);
        }

        let derived_tx = GraphTransaction {
            label: None,
            ops: derived_ops.clone(),
        };
        apply_transaction(graph, &derived_tx)?;
        committed.ops.extend(derived_ops);
    }

    Err(ApplyPipelineError::ConcretizeNonConvergent { bound })
}

/// Helper for UI loops: apply a `ConnectPlan` as a transaction via the profile pipeline.
pub fn apply_connect_plan_with_profile(
    graph: &mut Graph,
    profile: &mut dyn GraphProfile,
    plan: &ConnectPlan,
) -> Result<GraphTransaction, ApplyPipelineError> {
    match plan.decision {
        ConnectDecision::Accept => {
            let tx = GraphTransaction {
                label: None,
                ops: plan.ops.clone(),
            };
            apply_transaction_with_profile(graph, profile, &tx)
        }
        ConnectDecision::Reject => {
            let message = plan
                .diagnostics
                .first()
                .map(|d| d.message.clone())
                .unwrap_or_else(|| "connect rejected".to_string());
            Err(ApplyPipelineError::Rejected {
                message,
                diagnostics: plan.diagnostics.clone(),
            })
        }
    }
}
