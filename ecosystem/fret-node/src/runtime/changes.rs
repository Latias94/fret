//! XyFlow-style change model for editor runtimes.
//!
//! In XyFlow/ReactFlow, internal interactions produce "changes" that user code can apply to its
//! node/edge arrays via helpers like `applyNodeChanges`. In `fret-node`, the authoritative model
//! is a reversible `GraphTransaction` (undo/redo friendly). This module bridges the two worlds:
//! - Map `GraphTransaction` -> `(NodeChange, EdgeChange)` events (for callbacks / middleware).
//! - Map `(NodeChange, EdgeChange)` -> reversible `GraphTransaction` (for store dispatch).

use serde::{Deserialize, Serialize};

use crate::core::{
    CanvasPoint, CanvasSize, Edge, EdgeId, EdgeKind, Graph, Node, NodeId, NodeKindKey, PortId,
};
use crate::ops::{EdgeEndpoints, GraphOp, GraphTransaction};

/// Changes targeting nodes (graph-owned).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeChange {
    Add {
        id: NodeId,
        node: Node,
    },
    Remove {
        id: NodeId,
    },

    Position {
        id: NodeId,
        position: CanvasPoint,
    },
    Kind {
        id: NodeId,
        kind: NodeKindKey,
    },
    KindVersion {
        id: NodeId,
        kind_version: u32,
    },
    Size {
        id: NodeId,
        size: Option<CanvasSize>,
    },
    Collapsed {
        id: NodeId,
        collapsed: bool,
    },
    Data {
        id: NodeId,
        data: serde_json::Value,
    },
}

/// Changes targeting edges (graph-owned).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EdgeChange {
    Add {
        id: EdgeId,
        edge: Edge,
    },
    Remove {
        id: EdgeId,
    },

    Kind {
        id: EdgeId,
        kind: EdgeKind,
    },
    Endpoints {
        id: EdgeId,
        from: PortId,
        to: PortId,
    },
}

/// Split view of change events.
#[derive(Debug, Default, Clone)]
pub struct NodeGraphChanges {
    pub nodes: Vec<NodeChange>,
    pub edges: Vec<EdgeChange>,
}

#[derive(Debug, thiserror::Error)]
pub enum ChangesToTransactionError {
    #[error("node not found: {0:?}")]
    MissingNode(NodeId),
    #[error("edge not found: {0:?}")]
    MissingEdge(EdgeId),
}

impl NodeGraphChanges {
    /// Derives change events from a reversible graph transaction.
    ///
    /// This is intended for store callbacks (e.g. "on_nodes_change") and middleware.
    pub fn from_transaction(tx: &GraphTransaction) -> Self {
        let mut out = Self::default();
        for op in &tx.ops {
            match op {
                GraphOp::AddNode { id, node } => out.nodes.push(NodeChange::Add {
                    id: *id,
                    node: node.clone(),
                }),
                GraphOp::RemoveNode { id, .. } => out.nodes.push(NodeChange::Remove { id: *id }),
                GraphOp::SetNodePos { id, to, .. } => out.nodes.push(NodeChange::Position {
                    id: *id,
                    position: *to,
                }),
                GraphOp::SetNodeKind { id, to, .. } => out.nodes.push(NodeChange::Kind {
                    id: *id,
                    kind: to.clone(),
                }),
                GraphOp::SetNodeKindVersion { id, to, .. } => {
                    out.nodes.push(NodeChange::KindVersion {
                        id: *id,
                        kind_version: *to,
                    })
                }
                GraphOp::SetNodeSize { id, to, .. } => {
                    out.nodes.push(NodeChange::Size { id: *id, size: *to })
                }
                GraphOp::SetNodeCollapsed { id, to, .. } => out.nodes.push(NodeChange::Collapsed {
                    id: *id,
                    collapsed: *to,
                }),
                GraphOp::SetNodeData { id, to, .. } => out.nodes.push(NodeChange::Data {
                    id: *id,
                    data: to.clone(),
                }),
                GraphOp::AddEdge { id, edge } => out.edges.push(EdgeChange::Add {
                    id: *id,
                    edge: edge.clone(),
                }),
                GraphOp::RemoveEdge { id, .. } => out.edges.push(EdgeChange::Remove { id: *id }),
                GraphOp::SetEdgeKind { id, to, .. } => {
                    out.edges.push(EdgeChange::Kind { id: *id, kind: *to })
                }
                GraphOp::SetEdgeEndpoints { id, to, .. } => out.edges.push(EdgeChange::Endpoints {
                    id: *id,
                    from: to.from,
                    to: to.to,
                }),
                _ => {}
            }
        }
        out
    }

    /// Converts change events into a reversible transaction by looking up "from" values in the
    /// current graph.
    ///
    /// This enables an XyFlow-like runtime to accept `(NodeChange, EdgeChange)` and still keep
    /// `GraphHistory` undo/redo semantics.
    pub fn to_transaction(
        &self,
        graph: &Graph,
    ) -> Result<GraphTransaction, ChangesToTransactionError> {
        let mut tx = GraphTransaction::new();

        for change in &self.nodes {
            match change {
                NodeChange::Add { id, node } => tx.push(GraphOp::AddNode {
                    id: *id,
                    node: node.clone(),
                }),
                NodeChange::Remove { id } => {
                    let node = graph
                        .nodes
                        .get(id)
                        .cloned()
                        .ok_or(ChangesToTransactionError::MissingNode(*id))?;
                    let ports: Vec<_> = graph
                        .ports
                        .iter()
                        .filter_map(|(pid, p)| (p.node == *id).then_some((*pid, p.clone())))
                        .collect();
                    let port_ids: std::collections::HashSet<_> =
                        ports.iter().map(|(pid, _)| *pid).collect();
                    let edges: Vec<_> = graph
                        .edges
                        .iter()
                        .filter_map(|(eid, e)| {
                            (port_ids.contains(&e.from) || port_ids.contains(&e.to))
                                .then_some((*eid, e.clone()))
                        })
                        .collect();

                    tx.push(GraphOp::RemoveNode {
                        id: *id,
                        node,
                        ports,
                        edges,
                    });
                }
                NodeChange::Position { id, position } => {
                    let from = graph
                        .nodes
                        .get(id)
                        .map(|n| n.pos)
                        .ok_or(ChangesToTransactionError::MissingNode(*id))?;
                    tx.push(GraphOp::SetNodePos {
                        id: *id,
                        from,
                        to: *position,
                    });
                }
                NodeChange::Kind { id, kind } => {
                    let from = graph
                        .nodes
                        .get(id)
                        .map(|n| n.kind.clone())
                        .ok_or(ChangesToTransactionError::MissingNode(*id))?;
                    tx.push(GraphOp::SetNodeKind {
                        id: *id,
                        from,
                        to: kind.clone(),
                    });
                }
                NodeChange::KindVersion { id, kind_version } => {
                    let from = graph
                        .nodes
                        .get(id)
                        .map(|n| n.kind_version)
                        .ok_or(ChangesToTransactionError::MissingNode(*id))?;
                    tx.push(GraphOp::SetNodeKindVersion {
                        id: *id,
                        from,
                        to: *kind_version,
                    });
                }
                NodeChange::Size { id, size } => {
                    let from = graph
                        .nodes
                        .get(id)
                        .map(|n| n.size)
                        .ok_or(ChangesToTransactionError::MissingNode(*id))?;
                    tx.push(GraphOp::SetNodeSize {
                        id: *id,
                        from,
                        to: *size,
                    });
                }
                NodeChange::Collapsed { id, collapsed } => {
                    let from = graph
                        .nodes
                        .get(id)
                        .map(|n| n.collapsed)
                        .ok_or(ChangesToTransactionError::MissingNode(*id))?;
                    tx.push(GraphOp::SetNodeCollapsed {
                        id: *id,
                        from,
                        to: *collapsed,
                    });
                }
                NodeChange::Data { id, data } => {
                    let from = graph
                        .nodes
                        .get(id)
                        .map(|n| n.data.clone())
                        .ok_or(ChangesToTransactionError::MissingNode(*id))?;
                    tx.push(GraphOp::SetNodeData {
                        id: *id,
                        from,
                        to: data.clone(),
                    });
                }
            }
        }

        for change in &self.edges {
            match change {
                EdgeChange::Add { id, edge } => tx.push(GraphOp::AddEdge {
                    id: *id,
                    edge: edge.clone(),
                }),
                EdgeChange::Remove { id } => {
                    let edge = graph
                        .edges
                        .get(id)
                        .cloned()
                        .ok_or(ChangesToTransactionError::MissingEdge(*id))?;
                    tx.push(GraphOp::RemoveEdge { id: *id, edge });
                }
                EdgeChange::Kind { id, kind } => {
                    let from = graph
                        .edges
                        .get(id)
                        .map(|e| e.kind)
                        .ok_or(ChangesToTransactionError::MissingEdge(*id))?;
                    tx.push(GraphOp::SetEdgeKind {
                        id: *id,
                        from,
                        to: *kind,
                    });
                }
                EdgeChange::Endpoints { id, from, to } => {
                    let edge = graph
                        .edges
                        .get(id)
                        .cloned()
                        .ok_or(ChangesToTransactionError::MissingEdge(*id))?;
                    tx.push(GraphOp::SetEdgeEndpoints {
                        id: *id,
                        from: EdgeEndpoints {
                            from: edge.from,
                            to: edge.to,
                        },
                        to: EdgeEndpoints {
                            from: *from,
                            to: *to,
                        },
                    });
                }
            }
        }

        Ok(tx)
    }
}
