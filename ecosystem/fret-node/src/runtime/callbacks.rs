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

use crate::core::{EdgeId, EdgeKind, PortId};
use crate::ops::{EdgeEndpoints, GraphOp, GraphTransaction};
use crate::runtime::changes::{EdgeChange, NodeChange, NodeGraphChanges};
use crate::runtime::events::{NodeGraphStoreEvent, SubscriptionToken, ViewChange};
use crate::runtime::store::NodeGraphStore;

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

/// Object-safe callback trait for B-layer consumers.
///
/// Ordering guarantees (per store event):
///
/// - For `GraphCommitted`:
///   1) `on_graph_commit`
///   2) `on_nodes_change` (if non-empty)
///   3) `on_edges_change` (if non-empty)
///   4) `on_connection_change` for each derived `ConnectionChange`
/// - For `ViewChanged`:
///   1) `on_view_change`
pub trait NodeGraphCallbacks: 'static {
    fn on_graph_commit(&mut self, _committed: &GraphTransaction, _changes: &NodeGraphChanges) {}

    fn on_nodes_change(&mut self, _changes: &[NodeChange]) {}
    fn on_edges_change(&mut self, _changes: &[EdgeChange]) {}

    fn on_connection_change(&mut self, _change: ConnectionChange) {}

    fn on_view_change(&mut self, _changes: &[ViewChange]) {}
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
            }
        }
        NodeGraphStoreEvent::ViewChanged { changes, .. } => {
            callbacks.on_view_change(changes);
        }
    })
}

pub fn connection_changes_from_transaction(tx: &GraphTransaction) -> Vec<ConnectionChange> {
    let mut out = Vec::new();
    out.reserve(tx.ops.len().min(8));

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
            GraphOp::RemoveEdge { id, edge } => {
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
