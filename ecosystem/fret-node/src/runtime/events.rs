//! B-layer store event model (subscriptions).
//!
//! This is intentionally small and headless-safe.

use crate::core::CanvasPoint;
use crate::io::NodeGraphViewState;
use crate::runtime::changes::NodeGraphChanges;

/// Subscription token returned by [`crate::runtime::store::NodeGraphStore::subscribe`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionToken(u64);

impl SubscriptionToken {
    pub(crate) fn new(raw: u64) -> Self {
        Self(raw)
    }
}

/// Immutable snapshot of store state for selector subscriptions.
#[derive(Debug, Clone, Copy)]
pub struct NodeGraphStoreSnapshot<'a> {
    pub graph: &'a crate::core::Graph,
    pub view_state: &'a crate::io::NodeGraphViewState,
    pub history: &'a crate::ops::GraphHistory,
}

/// View-state projection change events.
///
/// These are the B-layer equivalent of XyFlow's selection/viewport updates (which are embedded in
/// their node/edge arrays). In fret-node, view-state is intentionally separate from the serialized
/// graph document.
///
/// Only viewport/selection changes are surfaced here. Other persisted view-state fields such as
/// draw order, interaction config, and runtime tuning are observable through selector
/// subscriptions on [`NodeGraphStoreSnapshot`].
#[derive(Debug, Clone)]
pub enum ViewChange {
    Viewport {
        pan: CanvasPoint,
        zoom: f32,
    },
    Selection {
        nodes: Vec<crate::core::NodeId>,
        edges: Vec<crate::core::EdgeId>,
        groups: Vec<crate::core::GroupId>,
    },
}

/// Store event emitted to subscribers.
#[derive(Clone, Copy)]
pub enum NodeGraphStoreEvent<'a> {
    GraphCommitted {
        committed: &'a crate::ops::GraphTransaction,
        changes: &'a NodeGraphChanges,
    },
    ViewChanged {
        before: &'a NodeGraphViewState,
        after: &'a NodeGraphViewState,
        changes: &'a [ViewChange],
    },
}
