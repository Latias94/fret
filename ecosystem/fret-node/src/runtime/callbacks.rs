//! ReactFlow-style callback surface for B-layer integrations.
//!
//! `NodeGraphStore` already emits a low-level event stream (`NodeGraphStoreEvent`), but users
//! typically want a higher-level contract similar to ReactFlow:
//! - `onNodesChange`
//! - `onEdgesChange`
//! - `onConnect` / `onReconnect` / `onDisconnect`
//! - `onViewportChange` / `onSelectionChange`
//!
//! This module provides an object-safe callback trait and an adapter that can be installed into
//! a store subscription.

use crate::core::{CanvasPoint, EdgeId, EdgeKind, GroupId, NodeId, PortId, StickyNoteId};
use crate::interaction::NodeGraphConnectionMode;
use crate::ops::{EdgeEndpoints, GraphOp, GraphTransaction};
use crate::rules::EdgeEndpoint;
use crate::runtime::changes::{EdgeChange, NodeChange, NodeGraphChanges};
use crate::runtime::events::{NodeGraphStoreEvent, SubscriptionToken, ViewChange};
use crate::runtime::store::NodeGraphStore;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeleteChange {
    pub nodes: Vec<NodeId>,
    pub edges: Vec<EdgeId>,
    pub groups: Vec<GroupId>,
    pub sticky_notes: Vec<StickyNoteId>,
}

/// Viewport move gesture kind (UI-driven).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportMoveKind {
    /// Pointer-drag panning (mouse/touch drag).
    PanDrag,
    /// Inertial/momentum panning after releasing a pan drag.
    PanInertia,
    /// Panning via scroll wheel / trackpad scroll when `pan_on_scroll` is enabled.
    PanScroll,
    /// Zooming via scroll wheel (e.g. Ctrl+wheel).
    ZoomWheel,
    /// Zooming via pinch gesture (trackpad pinch).
    ZoomPinch,
    /// Zooming via double-click gesture.
    ZoomDoubleClick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportMoveEndOutcome {
    Ended,
    Canceled,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportMoveStart {
    pub kind: ViewportMoveKind,
    pub pan: CanvasPoint,
    pub zoom: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportMoveEnd {
    pub kind: ViewportMoveKind,
    pub pan: CanvasPoint,
    pub zoom: f32,
    pub outcome: ViewportMoveEndOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeDragEndOutcome {
    Committed,
    Rejected,
    Canceled,
    NoOp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeDragStart {
    pub primary: NodeId,
    pub nodes: Vec<NodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeDragEnd {
    pub primary: NodeId,
    pub nodes: Vec<NodeId>,
    pub outcome: NodeDragEndOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeConnection {
    pub edge: EdgeId,
    pub from: PortId,
    pub to: PortId,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionChange {
    Connected(EdgeConnection),
    Disconnected(EdgeConnection),
    Reconnected {
        edge: EdgeId,
        from: EdgeEndpoints,
        to: EdgeEndpoints,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionChange {
    pub nodes: Vec<NodeId>,
    pub edges: Vec<EdgeId>,
    pub groups: Vec<GroupId>,
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

/// Headless/store commit callbacks for B-layer consumers.
///
/// Use this layer for controlled graph synchronization, analytics, and transaction-driven
/// integrations that only care about committed graph diffs.
///
/// Ordering guarantees (per `GraphCommitted` store event):
///
/// 1) `on_graph_commit`
/// 2) `on_nodes_change` (if non-empty)
/// 3) `on_edges_change` (if non-empty)
/// 4) `on_connection_change` for each derived `ConnectionChange`
/// 5) `on_connect`/`on_disconnect`/`on_reconnect` for each derived `ConnectionChange`
pub trait NodeGraphCommitCallbacks: 'static {
    fn on_graph_commit(&mut self, _committed: &GraphTransaction, _changes: &NodeGraphChanges) {}

    fn on_nodes_change(&mut self, _changes: &[NodeChange]) {}
    fn on_edges_change(&mut self, _changes: &[EdgeChange]) {}

    fn on_connection_change(&mut self, _change: ConnectionChange) {}

    fn on_connect(&mut self, _conn: EdgeConnection) {}
    fn on_disconnect(&mut self, _conn: EdgeConnection) {}
    fn on_reconnect(&mut self, _edge: EdgeId, _from: EdgeEndpoints, _to: EdgeEndpoints) {}

    /// ReactFlow-style alias for reconnect (`onEdgeUpdate`).
    ///
    /// This hook is derived from committed ops (headless-safe), just like `on_reconnect`.
    fn on_edge_update(&mut self, _edge: EdgeId, _from: EdgeEndpoints, _to: EdgeEndpoints) {}

    /// ReactFlow-style delete hook (`onNodesDelete`).
    fn on_nodes_delete(&mut self, _nodes: &[NodeId]) {}
    /// ReactFlow-style delete hook (`onEdgesDelete`).
    fn on_edges_delete(&mut self, _edges: &[EdgeId]) {}
    /// Delete hook for group containers.
    fn on_groups_delete(&mut self, _groups: &[GroupId]) {}
    /// Delete hook for sticky notes.
    fn on_sticky_notes_delete(&mut self, _notes: &[StickyNoteId]) {}
    /// Combined delete hook (ReactFlow `onDelete`-like).
    fn on_delete(&mut self, _change: DeleteChange) {}
}

/// Headless/store view callbacks for B-layer consumers.
///
/// Use this layer for app-owned viewport/selection synchronization. These hooks are derived from
/// `ViewChange` and remain headless-safe.
///
/// Ordering guarantees (per `ViewChanged` store event):
///
/// 1) `on_view_change`
/// 2) `on_viewport_change` / `on_selection_change` for each derived `ViewChange`
pub trait NodeGraphViewCallbacks: 'static {
    fn on_view_change(&mut self, _changes: &[ViewChange]) {}

    fn on_viewport_change(&mut self, _pan: CanvasPoint, _zoom: f32) {}
    fn on_selection_change(&mut self, _sel: SelectionChange) {}

    /// ReactFlow-style alias for viewport updates (`onMove`).
    ///
    /// This hook is derived from view-state changes (headless-safe), just like `on_viewport_change`.
    fn on_move(&mut self, _pan: CanvasPoint, _zoom: f32) {}
}

/// UI gesture lifecycle callbacks for retained/editor shells.
///
/// Use this layer for canvas-owned transient gesture observation. App-facing controlled
/// integrations usually only need commit/view callbacks unless they intentionally react to
/// pointer-driven lifecycle events.
pub trait NodeGraphGestureCallbacks: 'static {
    /// UI-driven hook: viewport pan/zoom gesture start (ReactFlow `onMoveStart`).
    fn on_move_start(&mut self, _ev: ViewportMoveStart) {}
    /// UI-driven hook: viewport pan/zoom gesture end (ReactFlow `onMoveEnd`).
    fn on_move_end(&mut self, _ev: ViewportMoveEnd) {}

    /// UI-driven hook: node drag gesture start (ReactFlow `onNodeDragStart`).
    fn on_node_drag_start(&mut self, _ev: NodeDragStart) {}
    /// UI-driven hook: node drag gesture end (ReactFlow `onNodeDragStop`).
    fn on_node_drag_end(&mut self, _ev: NodeDragEnd) {}
    /// UI-driven hook: node drag gesture move (ReactFlow `onNodeDrag`).
    fn on_node_drag(&mut self, _primary: NodeId, _nodes: &[NodeId]) {}

    /// UI-driven hook: called when a connection gesture starts (after drag threshold / click-to-connect).
    fn on_connect_start(&mut self, _ev: ConnectStart) {}
    /// UI-driven hook: called when a connection gesture ends (commit/reject/cancel/picker).
    fn on_connect_end(&mut self, _ev: ConnectEnd) {}

    /// UI-driven hook: called when a reconnect gesture starts.
    ///
    /// This is a reconnect-only alias that mirrors ReactFlow's `onReconnectStart`.
    /// Note that `on_connect_start` is still emitted (with `ConnectDragKind::Reconnect*`).
    fn on_reconnect_start(&mut self, _ev: ConnectStart) {}

    /// UI-driven hook: called when a reconnect gesture ends.
    ///
    /// This is a reconnect-only alias that mirrors ReactFlow's `onReconnectEnd`.
    /// Note that `on_connect_end` is still emitted (with `ConnectDragKind::Reconnect*`).
    fn on_reconnect_end(&mut self, _ev: ConnectEnd) {}

    /// UI-driven hook: called when an edge update (reconnect) gesture starts.
    ///
    /// This is a reconnect-only alias that mirrors ReactFlow's `onEdgeUpdateStart`.
    /// Note that `on_connect_start` is still emitted (with `ConnectDragKind::Reconnect*`).
    fn on_edge_update_start(&mut self, _ev: ConnectStart) {}

    /// UI-driven hook: called when an edge update (reconnect) gesture ends.
    ///
    /// This is a reconnect-only alias that mirrors ReactFlow's `onEdgeUpdateEnd`.
    /// Note that `on_connect_end` is still emitted (with `ConnectDragKind::Reconnect*`).
    fn on_edge_update_end(&mut self, _ev: ConnectEnd) {}
}

/// Composite callback surface consumed by store/canvas adapters.
///
/// Prefer implementing the smallest concern traits:
///
/// - `NodeGraphCommitCallbacks` for committed graph diffs,
/// - `NodeGraphViewCallbacks` for viewport/selection synchronization,
/// - `NodeGraphGestureCallbacks` for transient UI gesture lifecycle.
///
/// `NodeGraphCallbacks` itself is only the composition boundary used by `install_callbacks` and
/// retained canvas wiring.
pub trait NodeGraphCallbacks:
    NodeGraphCommitCallbacks + NodeGraphViewCallbacks + NodeGraphGestureCallbacks
{
}

impl<T> NodeGraphCallbacks for T where
    T: NodeGraphCommitCallbacks + NodeGraphViewCallbacks + NodeGraphGestureCallbacks
{
}

/// Installs callbacks into a store via a subscription.
pub fn install_callbacks(
    store: &mut NodeGraphStore,
    callbacks: impl NodeGraphCallbacks,
) -> SubscriptionToken {
    let mut callbacks: Box<dyn NodeGraphCallbacks> = Box::new(callbacks);
    store.subscribe(move |ev| match ev {
        NodeGraphStoreEvent::GraphCommitted { committed, changes } => {
            callbacks.on_graph_commit(committed, changes);
            if !changes.nodes.is_empty() {
                callbacks.on_nodes_change(&changes.nodes);
            }
            if !changes.edges.is_empty() {
                callbacks.on_edges_change(&changes.edges);
            }

            for change in connection_changes_from_transaction(committed) {
                callbacks.on_connection_change(change);
                match change {
                    ConnectionChange::Connected(conn) => callbacks.on_connect(conn),
                    ConnectionChange::Disconnected(conn) => callbacks.on_disconnect(conn),
                    ConnectionChange::Reconnected { edge, from, to } => {
                        callbacks.on_reconnect(edge, from, to);
                        callbacks.on_edge_update(edge, from, to);
                    }
                }
            }

            let deleted = delete_changes_from_transaction(committed);
            if !deleted.nodes.is_empty() {
                callbacks.on_nodes_delete(&deleted.nodes);
            }
            if !deleted.edges.is_empty() {
                callbacks.on_edges_delete(&deleted.edges);
            }
            if !deleted.groups.is_empty() {
                callbacks.on_groups_delete(&deleted.groups);
            }
            if !deleted.sticky_notes.is_empty() {
                callbacks.on_sticky_notes_delete(&deleted.sticky_notes);
            }
            if !deleted.nodes.is_empty()
                || !deleted.edges.is_empty()
                || !deleted.groups.is_empty()
                || !deleted.sticky_notes.is_empty()
            {
                callbacks.on_delete(deleted);
            }
        }
        NodeGraphStoreEvent::ViewChanged { changes, .. } => {
            callbacks.on_view_change(changes);
            for change in changes.iter() {
                match change {
                    ViewChange::Viewport { pan, zoom } => {
                        callbacks.on_viewport_change(*pan, *zoom);
                        callbacks.on_move(*pan, *zoom);
                    }
                    ViewChange::Selection {
                        nodes,
                        edges,
                        groups,
                    } => callbacks.on_selection_change(SelectionChange {
                        nodes: nodes.clone(),
                        edges: edges.clone(),
                        groups: groups.clone(),
                    }),
                }
            }
        }
    })
}

pub fn connection_changes_from_transaction(tx: &GraphTransaction) -> Vec<ConnectionChange> {
    let mut out = Vec::new();
    out.reserve(tx.ops.len().min(8));

    let mut removed_edges: std::collections::BTreeSet<EdgeId> = std::collections::BTreeSet::new();
    for op in &tx.ops {
        match op {
            GraphOp::AddEdge { id, edge } => {
                out.push(ConnectionChange::Connected(EdgeConnection {
                    edge: *id,
                    from: edge.from,
                    to: edge.to,
                    kind: edge.kind,
                }))
            }
            GraphOp::RemoveNode { edges, .. } => {
                for (id, edge) in edges {
                    if !removed_edges.insert(*id) {
                        continue;
                    }
                    out.push(ConnectionChange::Disconnected(EdgeConnection {
                        edge: *id,
                        from: edge.from,
                        to: edge.to,
                        kind: edge.kind,
                    }))
                }
            }
            GraphOp::RemovePort { edges, .. } => {
                for (id, edge) in edges {
                    if !removed_edges.insert(*id) {
                        continue;
                    }
                    out.push(ConnectionChange::Disconnected(EdgeConnection {
                        edge: *id,
                        from: edge.from,
                        to: edge.to,
                        kind: edge.kind,
                    }))
                }
            }
            GraphOp::RemoveEdge { id, edge } => {
                let _ = removed_edges.insert(*id);
                out.push(ConnectionChange::Disconnected(EdgeConnection {
                    edge: *id,
                    from: edge.from,
                    to: edge.to,
                    kind: edge.kind,
                }))
            }
            GraphOp::SetEdgeEndpoints { id, from, to } => out.push(ConnectionChange::Reconnected {
                edge: *id,
                from: *from,
                to: *to,
            }),
            _ => {}
        }
    }

    out
}

pub fn delete_changes_from_transaction(tx: &GraphTransaction) -> DeleteChange {
    let mut out = DeleteChange::default();

    for op in &tx.ops {
        match op {
            GraphOp::RemoveNode { id, edges, .. } => {
                out.nodes.push(*id);
                for (edge_id, _edge) in edges {
                    out.edges.push(*edge_id);
                }
            }
            GraphOp::RemoveEdge { id, .. } => out.edges.push(*id),
            GraphOp::RemoveGroup { id, .. } => out.groups.push(*id),
            GraphOp::RemoveStickyNote { id, .. } => out.sticky_notes.push(*id),
            GraphOp::RemovePort { edges, .. } => {
                for (edge_id, _edge) in edges {
                    out.edges.push(*edge_id);
                }
            }
            _ => {}
        }
    }

    out.nodes.sort_unstable();
    out.nodes.dedup();
    out.edges.sort_unstable();
    out.edges.dedup();
    out.groups.sort_unstable();
    out.groups.dedup();
    out.sticky_notes.sort_unstable();
    out.sticky_notes.dedup();

    out
}
