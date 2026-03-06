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
use crate::rules::{Diagnostic, DiagnosticSeverity, DiagnosticTarget};
use crate::runtime::changes::{ChangesToTransactionError, NodeGraphChanges};
use crate::runtime::events::{
    NodeGraphStoreEvent, NodeGraphStoreSnapshot, SubscriptionToken, ViewChange,
};
use crate::runtime::lookups::NodeGraphLookups;
use crate::runtime::middleware::NodeGraphStoreMiddleware;

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
    middleware: Option<Box<dyn NodeGraphStoreMiddleware>>,
    lookups: NodeGraphLookups,

    next_subscription: u64,
    event_subscriptions: Vec<(
        SubscriptionToken,
        Box<dyn for<'a> FnMut(NodeGraphStoreEvent<'a>)>,
    )>,
    selector_subscriptions: Vec<SelectorSubscription>,
}

struct SelectorSubscription {
    token: SubscriptionToken,
    compute: Box<dyn for<'a> Fn(NodeGraphStoreSnapshot<'a>) -> Box<dyn std::any::Any>>,
    equals: Box<dyn Fn(&dyn std::any::Any, &dyn std::any::Any) -> bool>,
    callback: Box<dyn FnMut(&dyn std::any::Any, &dyn std::any::Any)>,
    last: Box<dyn std::any::Any>,
}

impl std::fmt::Debug for NodeGraphStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeGraphStore")
            .field("graph_id", &self.graph.graph_id)
            .field("node_count", &self.graph.nodes.len())
            .field("edge_count", &self.graph.edges.len())
            .field("lookup_node_count", &self.lookups.node_lookup.len())
            .field("lookup_edge_count", &self.lookups.edge_lookup.len())
            .field("undo_len", &self.history.undo_len())
            .field("redo_len", &self.history.redo_len())
            .field("has_profile", &self.profile.is_some())
            .field("event_subscription_count", &self.event_subscriptions.len())
            .field(
                "selector_subscription_count",
                &self.selector_subscriptions.len(),
            )
            .finish()
    }
}

impl NodeGraphStore {
    /// Creates a store without a profile pipeline (raw ops apply + undo/redo).
    pub fn new(graph: Graph, mut view_state: NodeGraphViewState) -> Self {
        view_state.sanitize_for_graph(&graph);
        let mut lookups = NodeGraphLookups::default();
        lookups.rebuild_from(&graph);
        Self {
            graph,
            view_state,
            history: GraphHistory::default(),
            profile: None,
            middleware: None,
            lookups,
            next_subscription: 1,
            event_subscriptions: Vec::new(),
            selector_subscriptions: Vec::new(),
        }
    }

    /// Creates a store with a profile pipeline (apply -> concretize -> validate).
    pub fn with_profile(
        graph: Graph,
        mut view_state: NodeGraphViewState,
        profile: Box<dyn GraphProfile>,
    ) -> Self {
        view_state.sanitize_for_graph(&graph);
        let mut lookups = NodeGraphLookups::default();
        lookups.rebuild_from(&graph);
        Self {
            graph,
            view_state,
            history: GraphHistory::default(),
            profile: Some(profile),
            middleware: None,
            lookups,
            next_subscription: 1,
            event_subscriptions: Vec::new(),
            selector_subscriptions: Vec::new(),
        }
    }

    pub fn with_middleware(mut self, middleware: impl NodeGraphStoreMiddleware) -> Self {
        self.middleware = Some(Box::new(middleware));
        self
    }

    /// Subscribes to store events (graph commits + view-state changes).
    ///
    /// This is the minimal B-layer equivalent of XyFlow's store subscriptions.
    pub fn subscribe(
        &mut self,
        f: impl for<'a> FnMut(NodeGraphStoreEvent<'a>) + 'static,
    ) -> SubscriptionToken {
        let token = SubscriptionToken::new(self.next_subscription);
        self.next_subscription = self.next_subscription.saturating_add(1).max(1);
        self.event_subscriptions.push((token, Box::new(f)));
        token
    }

    /// Subscribes to a derived projection of store state and only fires when the derived value
    /// changes (by `PartialEq`).
    ///
    /// This is the B-layer "selector subscription" pattern used by XyFlow.
    pub fn subscribe_selector<T>(
        &mut self,
        selector: impl for<'a> Fn(NodeGraphStoreSnapshot<'a>) -> T + 'static,
        mut on_change: impl FnMut(&T) + 'static,
    ) -> SubscriptionToken
    where
        T: Clone + PartialEq + 'static,
    {
        self.subscribe_selector_diff(selector, move |_prev, next| on_change(next))
    }

    /// Subscribes to a derived projection and receives both the previous and next values.
    pub fn subscribe_selector_diff<T>(
        &mut self,
        selector: impl for<'a> Fn(NodeGraphStoreSnapshot<'a>) -> T + 'static,
        mut on_change: impl FnMut(&T, &T) + 'static,
    ) -> SubscriptionToken
    where
        T: Clone + PartialEq + 'static,
    {
        let token = SubscriptionToken::new(self.next_subscription);
        self.next_subscription = self.next_subscription.saturating_add(1).max(1);

        let snapshot = NodeGraphStoreSnapshot {
            graph: &self.graph,
            view_state: &self.view_state,
            history: &self.history,
        };
        let initial = selector(snapshot);

        self.selector_subscriptions.push(SelectorSubscription {
            token,
            compute: Box::new(move |snapshot| {
                Box::new(selector(snapshot)) as Box<dyn std::any::Any>
            }),
            equals: Box::new(|a, b| {
                let a = a.downcast_ref::<T>().expect("selector type mismatch");
                let b = b.downcast_ref::<T>().expect("selector type mismatch");
                a == b
            }),
            callback: Box::new(move |prev, next| {
                let prev = prev.downcast_ref::<T>().expect("selector type mismatch");
                let next = next.downcast_ref::<T>().expect("selector type mismatch");
                on_change(prev, next);
            }),
            last: Box::new(initial),
        });

        token
    }

    /// Removes a subscription.
    pub fn unsubscribe(&mut self, token: SubscriptionToken) -> bool {
        let mut removed = false;

        let before = self.event_subscriptions.len();
        self.event_subscriptions.retain(|(t, _)| *t != token);
        removed |= before != self.event_subscriptions.len();

        let before = self.selector_subscriptions.len();
        self.selector_subscriptions.retain(|s| s.token != token);
        removed |= before != self.selector_subscriptions.len();

        removed
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    pub fn lookups(&self) -> &NodeGraphLookups {
        &self.lookups
    }

    /// Replaces the entire graph document.
    ///
    /// This is a controlled-mode helper: callers that own graph state can swap the document
    /// without going through transactions (e.g. loading a file, switching tabs).
    ///
    /// Note: this does not emit `NodeGraphChanges` today. Consumers should treat this as a full
    /// reset and re-render. Selection is sanitized against the new graph.
    pub fn replace_graph(&mut self, graph: Graph) {
        self.graph = graph;
        self.view_state.sanitize_for_graph(&self.graph);
        self.lookups.rebuild_from(&self.graph);
        self.notify_selectors();
    }

    pub fn view_state(&self) -> &NodeGraphViewState {
        &self.view_state
    }

    pub fn view_state_mut(&mut self) -> &mut NodeGraphViewState {
        &mut self.view_state
    }

    /// Replaces the full view-state payload.
    ///
    /// This is the controlled-mode counterpart of `set_viewport`/`set_selection`.
    pub fn replace_view_state(&mut self, mut view_state: NodeGraphViewState) {
        view_state.sanitize_for_graph(&self.graph);
        let before = self.view_state.clone();
        if view_state_eq(&before, &view_state) {
            return;
        }

        self.view_state = view_state;
        let after = self.view_state.clone();

        let changes = collect_view_projection_changes(&before, &after);

        if !changes.is_empty() {
            self.emit(NodeGraphStoreEvent::ViewChanged {
                before: &before,
                after: &after,
                changes: &changes,
            });
        }
        self.notify_selectors();
    }

    /// Mutates view-state in place and emits derived `ViewChange` events.
    pub fn update_view_state(&mut self, f: impl FnOnce(&mut NodeGraphViewState)) {
        let before = self.view_state.clone();
        f(&mut self.view_state);
        self.view_state.sanitize_for_graph(&self.graph);
        let after = self.view_state.clone();

        if view_state_eq(&before, &after) {
            return;
        }

        let changes = collect_view_projection_changes(&before, &after);

        if !changes.is_empty() {
            self.emit(NodeGraphStoreEvent::ViewChanged {
                before: &before,
                after: &after,
                changes: &changes,
            });
        }
        self.notify_selectors();
    }

    /// Sets the viewport (pan/zoom) and notifies subscribers.
    pub fn set_viewport(&mut self, pan: crate::core::CanvasPoint, zoom: f32) {
        let z = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        let before = self.view_state.clone();
        if self.view_state.pan == pan && self.view_state.zoom == z {
            return;
        }

        self.view_state.pan = pan;
        self.view_state.zoom = z;
        let after = self.view_state.clone();

        let changes = [ViewChange::Viewport { pan, zoom: z }];
        self.emit(NodeGraphStoreEvent::ViewChanged {
            before: &before,
            after: &after,
            changes: &changes,
        });
        self.notify_selectors();
    }

    /// Sets selection state and notifies subscribers.
    pub fn set_selection(
        &mut self,
        nodes: Vec<crate::core::NodeId>,
        edges: Vec<crate::core::EdgeId>,
        groups: Vec<crate::core::GroupId>,
    ) {
        let before = self.view_state.clone();

        self.view_state.selected_nodes = nodes;
        self.view_state.selected_edges = edges;
        self.view_state.selected_groups = groups;
        self.view_state.sanitize_for_graph(&self.graph);
        let after = self.view_state.clone();

        if before.selected_nodes == after.selected_nodes
            && before.selected_edges == after.selected_edges
            && before.selected_groups == after.selected_groups
        {
            return;
        }

        let changes = [ViewChange::Selection {
            nodes: after.selected_nodes.clone(),
            edges: after.selected_edges.clone(),
            groups: after.selected_groups.clone(),
        }];
        self.emit(NodeGraphStoreEvent::ViewChanged {
            before: &before,
            after: &after,
            changes: &changes,
        });
        self.notify_selectors();
    }

    pub fn history(&self) -> &GraphHistory {
        &self.history
    }

    pub fn clear_history(&mut self) {
        self.history = GraphHistory::default();
        self.notify_selectors();
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
        let mut tx = crate::ops::normalize_transaction(tx.clone());
        if tx.is_empty() {
            return Ok(DispatchOutcome {
                committed: tx,
                changes: NodeGraphChanges::default(),
            });
        }
        if let Some((key, message)) = crate::ops::find_non_finite_in_tx(&tx) {
            return Err(DispatchError::Apply(Self::reject_tx(key, message)));
        }
        if let Some((key, message)) = crate::ops::find_invalid_size_in_tx(&tx) {
            return Err(DispatchError::Apply(Self::reject_tx(key, message)));
        }

        if let Some(middleware) = self.middleware.as_deref_mut() {
            let snapshot = NodeGraphStoreSnapshot {
                graph: &self.graph,
                view_state: &self.view_state,
                history: &self.history,
            };
            middleware.before_dispatch(snapshot, &mut tx)?;
        }
        tx = crate::ops::normalize_transaction(tx);
        if tx.is_empty() {
            return Ok(DispatchOutcome {
                committed: tx,
                changes: NodeGraphChanges::default(),
            });
        }
        if let Some((key, message)) = crate::ops::find_non_finite_in_tx(&tx) {
            return Err(DispatchError::Apply(Self::reject_tx(key, message)));
        }
        if let Some((key, message)) = crate::ops::find_invalid_size_in_tx(&tx) {
            return Err(DispatchError::Apply(Self::reject_tx(key, message)));
        }

        let mut scratch = self.graph.clone();
        let committed = self.apply_to_graph(&mut scratch, &tx)?;
        let committed = crate::ops::normalize_transaction(committed);
        if let Some((key, message)) = crate::ops::find_non_finite_in_tx(&committed) {
            return Err(DispatchError::Apply(Self::reject_tx(key, message)));
        }
        if let Some((key, message)) = crate::ops::find_invalid_size_in_tx(&committed) {
            return Err(DispatchError::Apply(Self::reject_tx(key, message)));
        }
        self.graph = scratch;
        self.lookups.apply_transaction(&self.graph, &committed);
        self.history.record(committed.clone());
        let changes = NodeGraphChanges::from_transaction(&committed);

        if let Some(middleware) = self.middleware.as_deref_mut() {
            let snapshot = NodeGraphStoreSnapshot {
                graph: &self.graph,
                view_state: &self.view_state,
                history: &self.history,
            };
            middleware.after_dispatch(snapshot, &committed, &changes);
        }

        self.emit(NodeGraphStoreEvent::GraphCommitted {
            committed: &committed,
            changes: &changes,
        });
        self.notify_selectors();
        Ok(DispatchOutcome { committed, changes })
    }

    /// Dispatches a transaction using an externally-owned profile pipeline.
    ///
    /// This is intended for UI integration where the profile is owned by the presenter layer.
    pub fn dispatch_transaction_with_profile(
        &mut self,
        tx: &GraphTransaction,
        profile: &mut dyn GraphProfile,
    ) -> Result<DispatchOutcome, ApplyPipelineError> {
        let mut tx = crate::ops::normalize_transaction(tx.clone());
        if tx.is_empty() {
            return Ok(DispatchOutcome {
                committed: tx,
                changes: NodeGraphChanges::default(),
            });
        }
        if let Some((key, message)) = crate::ops::find_non_finite_in_tx(&tx) {
            return Err(Self::reject_tx(key, message));
        }
        if let Some((key, message)) = crate::ops::find_invalid_size_in_tx(&tx) {
            return Err(Self::reject_tx(key, message));
        }

        if let Some(middleware) = self.middleware.as_deref_mut() {
            let snapshot = NodeGraphStoreSnapshot {
                graph: &self.graph,
                view_state: &self.view_state,
                history: &self.history,
            };
            middleware.before_dispatch(snapshot, &mut tx)?;
        }
        tx = crate::ops::normalize_transaction(tx);
        if tx.is_empty() {
            return Ok(DispatchOutcome {
                committed: tx,
                changes: NodeGraphChanges::default(),
            });
        }
        if let Some((key, message)) = crate::ops::find_non_finite_in_tx(&tx) {
            return Err(Self::reject_tx(key, message));
        }
        if let Some((key, message)) = crate::ops::find_invalid_size_in_tx(&tx) {
            return Err(Self::reject_tx(key, message));
        }

        let mut scratch = self.graph.clone();
        let committed = apply_transaction_with_profile(&mut scratch, profile, &tx)?;
        let committed = crate::ops::normalize_transaction(committed);
        if let Some((key, message)) = crate::ops::find_non_finite_in_tx(&committed) {
            return Err(Self::reject_tx(key, message));
        }
        if let Some((key, message)) = crate::ops::find_invalid_size_in_tx(&committed) {
            return Err(Self::reject_tx(key, message));
        }
        self.graph = scratch;
        self.lookups.apply_transaction(&self.graph, &committed);
        self.history.record(committed.clone());
        let changes = NodeGraphChanges::from_transaction(&committed);

        if let Some(middleware) = self.middleware.as_deref_mut() {
            let snapshot = NodeGraphStoreSnapshot {
                graph: &self.graph,
                view_state: &self.view_state,
                history: &self.history,
            };
            middleware.after_dispatch(snapshot, &committed, &changes);
        }

        self.emit(NodeGraphStoreEvent::GraphCommitted {
            committed: &committed,
            changes: &changes,
        });
        self.notify_selectors();
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
        self.lookups.apply_transaction(&self.graph, &committed);
        self.emit(NodeGraphStoreEvent::GraphCommitted {
            committed: &committed,
            changes: &changes,
        });
        self.notify_selectors();
        Ok(Some(DispatchOutcome { committed, changes }))
    }

    /// Undoes the last committed transaction using an externally-owned profile pipeline.
    pub fn undo_with_profile(
        &mut self,
        profile: &mut dyn GraphProfile,
    ) -> Result<Option<DispatchOutcome>, ApplyPipelineError> {
        let mut scratch = self.graph.clone();
        let mut committed: Option<GraphTransaction> = None;

        let mut history = std::mem::take(&mut self.history);
        let did = history.undo(|tx| -> Result<GraphTransaction, ApplyPipelineError> {
            let committed_tx = apply_transaction_with_profile(&mut scratch, profile, tx)?;
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
        self.lookups.apply_transaction(&self.graph, &committed);
        self.emit(NodeGraphStoreEvent::GraphCommitted {
            committed: &committed,
            changes: &changes,
        });
        self.notify_selectors();
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
        self.lookups.apply_transaction(&self.graph, &committed);
        self.emit(NodeGraphStoreEvent::GraphCommitted {
            committed: &committed,
            changes: &changes,
        });
        self.notify_selectors();
        Ok(Some(DispatchOutcome { committed, changes }))
    }

    /// Redoes the last undone transaction using an externally-owned profile pipeline.
    pub fn redo_with_profile(
        &mut self,
        profile: &mut dyn GraphProfile,
    ) -> Result<Option<DispatchOutcome>, ApplyPipelineError> {
        let mut scratch = self.graph.clone();
        let mut committed: Option<GraphTransaction> = None;

        let mut history = std::mem::take(&mut self.history);
        let did = history.redo(|tx| -> Result<GraphTransaction, ApplyPipelineError> {
            let committed_tx = apply_transaction_with_profile(&mut scratch, profile, tx)?;
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
        self.lookups.apply_transaction(&self.graph, &committed);
        self.emit(NodeGraphStoreEvent::GraphCommitted {
            committed: &committed,
            changes: &changes,
        });
        self.notify_selectors();
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

    fn reject_tx(key: String, message: String) -> ApplyPipelineError {
        ApplyPipelineError::Rejected {
            message: message.clone(),
            diagnostics: vec![Diagnostic {
                key,
                severity: DiagnosticSeverity::Error,
                target: DiagnosticTarget::Graph,
                message,
                fixes: Vec::new(),
            }],
        }
    }

    fn emit(&mut self, event: NodeGraphStoreEvent<'_>) {
        for (_, sub) in &mut self.event_subscriptions {
            sub(event);
        }
    }

    fn notify_selectors(&mut self) {
        if self.selector_subscriptions.is_empty() {
            return;
        }

        let graph = &self.graph;
        let view_state = &self.view_state;
        let history = &self.history;
        for sub in &mut self.selector_subscriptions {
            let snapshot = NodeGraphStoreSnapshot {
                graph,
                view_state,
                history,
            };
            let next = (sub.compute)(snapshot);
            let changed = !(sub.equals)(&*sub.last, &*next);
            if !changed {
                continue;
            }
            (sub.callback)(&*sub.last, &*next);
            sub.last = next;
        }
    }
}

fn view_state_eq(a: &NodeGraphViewState, b: &NodeGraphViewState) -> bool {
    a.pan == b.pan
        && a.zoom == b.zoom
        && a.selected_nodes == b.selected_nodes
        && a.selected_edges == b.selected_edges
        && a.selected_groups == b.selected_groups
        && a.draw_order == b.draw_order
        && a.group_draw_order == b.group_draw_order
        && a.interaction == b.interaction
        && a.runtime_tuning == b.runtime_tuning
}

fn collect_view_projection_changes(
    before: &NodeGraphViewState,
    after: &NodeGraphViewState,
) -> Vec<ViewChange> {
    let mut changes: Vec<ViewChange> = Vec::new();
    if before.pan != after.pan || (before.zoom - after.zoom).abs() > 1.0e-6 {
        changes.push(ViewChange::Viewport {
            pan: after.pan,
            zoom: after.zoom,
        });
    }
    if before.selected_nodes != after.selected_nodes
        || before.selected_edges != after.selected_edges
        || before.selected_groups != after.selected_groups
    {
        changes.push(ViewChange::Selection {
            nodes: after.selected_nodes.clone(),
            edges: after.selected_edges.clone(),
            groups: after.selected_groups.clone(),
        });
    }
    changes
}
