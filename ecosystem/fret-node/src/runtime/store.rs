//! Headless runtime store (B-layer) for node graphs.
//!
//! This is the ergonomic "single entry point" that B-layer consumers want:
//! - authoritative `Graph` (serializable document),
//! - per-user/per-project `NodeGraphViewState` (pan/zoom/selection),
//! - undo/redo history (`GraphHistory`),
//! - dispatch methods that return `NodeGraphChanges` (XyFlow-style change events).

use crate::core::Graph;
use crate::io::NodeGraphViewState;
use crate::ops::{GraphHistory, GraphTransaction, apply_transaction};
use crate::profile::{ApplyPipelineError, GraphProfile, apply_transaction_with_profile};
use crate::runtime::changes::{ChangesToTransactionError, NodeGraphChanges};

/// Dispatch outcome for store actions.
#[derive(Debug, Clone)]
pub struct DispatchOutcome {
    /// The transaction that was committed (includes any derived ops when using a profile pipeline).
    pub committed: GraphTransaction,
    /// XyFlow-style change events derived from `committed`.
    pub changes: NodeGraphChanges,
}

#[derive(Debug, thiserror::Error)]
pub enum DispatchError {
    #[error(transparent)]
    Apply(#[from] ApplyPipelineError),
    #[error(transparent)]
    Changes(#[from] ChangesToTransactionError),
}

/// Minimal B-layer store.
///
/// This is intentionally headless-safe and does not depend on `fret-ui`.
pub struct NodeGraphStore {
    graph: Graph,
    view_state: NodeGraphViewState,
    history: GraphHistory,
    profile: Option<Box<dyn GraphProfile>>,
}

impl std::fmt::Debug for NodeGraphStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeGraphStore")
            .field("graph_id", &self.graph.graph_id)
            .field("node_count", &self.graph.nodes.len())
            .field("edge_count", &self.graph.edges.len())
            .field("undo_len", &self.history.undo_len())
            .field("redo_len", &self.history.redo_len())
            .field("has_profile", &self.profile.is_some())
            .finish()
    }
}

impl NodeGraphStore {
    /// Creates a store without a profile pipeline (raw ops apply + undo/redo).
    pub fn new(graph: Graph, mut view_state: NodeGraphViewState) -> Self {
        view_state.sanitize_for_graph(&graph);
        Self {
            graph,
            view_state,
            history: GraphHistory::default(),
            profile: None,
        }
    }

    /// Creates a store with a profile pipeline (apply -> concretize -> validate).
    pub fn with_profile(
        graph: Graph,
        mut view_state: NodeGraphViewState,
        profile: Box<dyn GraphProfile>,
    ) -> Self {
        view_state.sanitize_for_graph(&graph);
        Self {
            graph,
            view_state,
            history: GraphHistory::default(),
            profile: Some(profile),
        }
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    pub fn view_state(&self) -> &NodeGraphViewState {
        &self.view_state
    }

    pub fn view_state_mut(&mut self) -> &mut NodeGraphViewState {
        &mut self.view_state
    }

    pub fn history(&self) -> &GraphHistory {
        &self.history
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Applies a transaction and records it in history.
    ///
    /// This mirrors the UI loop contract: the store applies edits to a scratch graph first and only
    /// commits on success (so rejected profile validations do not partially mutate state).
    pub fn dispatch_transaction(
        &mut self,
        tx: &GraphTransaction,
    ) -> Result<DispatchOutcome, DispatchError> {
        let mut scratch = self.graph.clone();
        let committed = self.apply_to_graph(&mut scratch, tx)?;
        self.graph = scratch;
        self.history.record(committed.clone());
        let changes = NodeGraphChanges::from_transaction(&committed);
        Ok(DispatchOutcome { committed, changes })
    }

    /// Applies XyFlow-style changes by converting them to a reversible transaction.
    pub fn dispatch_changes(
        &mut self,
        changes: &NodeGraphChanges,
    ) -> Result<DispatchOutcome, DispatchError> {
        let tx = changes.to_transaction(&self.graph)?;
        self.dispatch_transaction(&tx)
    }

    /// Undoes the last committed transaction.
    pub fn undo(&mut self) -> Result<Option<DispatchOutcome>, DispatchError> {
        let mut scratch = self.graph.clone();
        let mut committed: Option<GraphTransaction> = None;

        let mut history = std::mem::take(&mut self.history);
        let did = history.undo(|tx| -> Result<GraphTransaction, ApplyPipelineError> {
            let committed_tx = self.apply_to_graph(&mut scratch, tx)?;
            committed = Some(committed_tx.clone());
            Ok(committed_tx)
        });
        self.history = history;
        let did = did?;

        if !did {
            return Ok(None);
        }

        let committed = committed.unwrap_or_else(GraphTransaction::new);
        let changes = NodeGraphChanges::from_transaction(&committed);
        self.graph = scratch;
        Ok(Some(DispatchOutcome { committed, changes }))
    }

    /// Redoes the last undone transaction.
    pub fn redo(&mut self) -> Result<Option<DispatchOutcome>, DispatchError> {
        let mut scratch = self.graph.clone();
        let mut committed: Option<GraphTransaction> = None;

        let mut history = std::mem::take(&mut self.history);
        let did = history.redo(|tx| -> Result<GraphTransaction, ApplyPipelineError> {
            let committed_tx = self.apply_to_graph(&mut scratch, tx)?;
            committed = Some(committed_tx.clone());
            Ok(committed_tx)
        });
        self.history = history;
        let did = did?;

        if !did {
            return Ok(None);
        }

        let committed = committed.unwrap_or_else(GraphTransaction::new);
        let changes = NodeGraphChanges::from_transaction(&committed);
        self.graph = scratch;
        Ok(Some(DispatchOutcome { committed, changes }))
    }

    fn apply_to_graph(
        &mut self,
        graph: &mut Graph,
        tx: &GraphTransaction,
    ) -> Result<GraphTransaction, ApplyPipelineError> {
        if let Some(profile) = self.profile.as_deref_mut() {
            apply_transaction_with_profile(graph, profile, tx)
        } else {
            apply_transaction(graph, tx)?;
            Ok(GraphTransaction {
                label: tx.label.clone(),
                ops: tx.ops.clone(),
            })
        }
    }
}
