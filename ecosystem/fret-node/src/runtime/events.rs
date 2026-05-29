//! B-layer store event model (subscriptions).
//!
//! This is intentionally small and headless-safe.

use crate::core::{CanvasPoint, EdgeId, PortId};
use crate::interaction::NodeGraphConnectionMode;
use crate::io::{
    NodeGraphEditorConfig, NodeGraphInteractionConfig, NodeGraphRuntimeTuning, NodeGraphViewState,
};
use crate::rules::EdgeEndpoint;
use crate::runtime::changes::{NodeGraphChanges, NodeGraphPatch};

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
    pub graph_revision: u64,
    pub view_state: &'a crate::io::NodeGraphViewState,
    pub interaction: &'a NodeGraphInteractionConfig,
    pub runtime_tuning: &'a NodeGraphRuntimeTuning,
    pub history: &'a crate::ops::GraphHistory,
}

/// Atomic document replacement snapshot.
#[derive(Debug, Clone, Copy)]
pub struct NodeGraphDocumentSnapshot<'a> {
    pub graph: &'a crate::core::Graph,
    pub graph_revision: u64,
    pub view_state: &'a NodeGraphViewState,
    pub editor_config: &'a NodeGraphEditorConfig,
}

/// View-state projection change events.
///
/// These are the B-layer equivalent of XyFlow's selection/viewport updates (which are embedded in
/// their node/edge arrays). In fret-node, view-state is intentionally separate from the serialized
/// graph document.
///
/// Only viewport/selection changes are surfaced here. Other persisted editor configuration is
/// observable through selector subscriptions on [`NodeGraphStoreSnapshot`].
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

/// Connection start kind (UI-driven).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectDragKind {
    New {
        from: PortId,
        bundle: Vec<PortId>,
    },
    Reconnect {
        edge: EdgeId,
        endpoint: EdgeEndpoint,
        fixed: PortId,
    },
    ReconnectMany {
        edges: Vec<(EdgeId, EdgeEndpoint, PortId)>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectStart {
    pub kind: ConnectDragKind,
    pub mode: NodeGraphConnectionMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectEndOutcome {
    /// A graph transaction was committed.
    Committed,
    /// A target was chosen but the connect plan was rejected.
    Rejected,
    /// The workflow opened a conversion picker (domain-specific UX).
    OpenConversionPicker,
    /// The workflow opened an insert-node picker (drop on empty background).
    OpenInsertNodePicker,
    /// The gesture was canceled (escape, focus lost, etc.).
    Canceled,
    /// Gesture ended without committing or opening a picker.
    NoOp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectEnd {
    pub kind: ConnectDragKind,
    pub mode: NodeGraphConnectionMode,
    pub target: Option<PortId>,
    pub outcome: ConnectEndOutcome,
}

/// Transient UI gesture event emitted to gesture subscribers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeGraphGestureEvent {
    ConnectStart(ConnectStart),
    ConnectEnd(ConnectEnd),
}

/// Store event emitted to subscribers.
#[derive(Clone, Copy)]
pub enum NodeGraphStoreEvent<'a> {
    DocumentReplaced {
        before: NodeGraphDocumentSnapshot<'a>,
        after: NodeGraphDocumentSnapshot<'a>,
    },
    GraphCommitted {
        patch: &'a NodeGraphPatch,
        node_edge_changes: &'a NodeGraphChanges,
    },
    ViewChanged {
        before: &'a NodeGraphViewState,
        after: &'a NodeGraphViewState,
        changes: &'a [ViewChange],
    },
}
