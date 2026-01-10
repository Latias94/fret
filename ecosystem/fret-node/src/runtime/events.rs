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

/// View-state change events.
///
/// These are the B-layer equivalent of XyFlow's selection/viewport updates (which are embedded in
/// their node/edge arrays). In fret-node, view-state is intentionally separate from the serialized
/// graph document.
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
