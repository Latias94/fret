//! XyFlow-style change model for editor runtimes.
//!
//! In XyFlow/ReactFlow, internal interactions produce "changes" that user code can apply to its
//! node/edge arrays via helpers like `applyNodeChanges`. In `fret-node`, the authoritative model
//! is a reversible `GraphTransaction` (undo/redo friendly). This module bridges the two worlds:
//! - Map `GraphTransaction` -> `(NodeChange, EdgeChange)` events (for callbacks / middleware).
//! - Map `(NodeChange, EdgeChange)` -> reversible `GraphTransaction` (for store dispatch).

use serde::{Deserialize, Serialize};

use crate::core::GroupId;
use crate::core::{
    CanvasPoint, CanvasSize, Edge, EdgeId, EdgeKind, EdgeReconnectable, Graph, Node, NodeExtent,
    NodeId, NodeKindKey, PortId,
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
    Selectable {
        id: NodeId,
        selectable: Option<bool>,
    },
    Draggable {
        id: NodeId,
        draggable: Option<bool>,
    },
    Connectable {
        id: NodeId,
        connectable: Option<bool>,
    },
    Deletable {
        id: NodeId,
        deletable: Option<bool>,
    },
    Parent {
        id: NodeId,
        parent: Option<GroupId>,
    },
    Extent {
        id: NodeId,
        extent: Option<NodeExtent>,
    },
    ExpandParent {
        id: NodeId,
        expand_parent: Option<bool>,
    },
    Size {
        id: NodeId,
        size: Option<CanvasSize>,
    },
    Hidden {
        id: NodeId,
        hidden: bool,
    },
    Collapsed {
        id: NodeId,
        collapsed: bool,
    },
    Data {
        id: NodeId,
        data: serde_json::Value,
    },
    Ports {
        id: NodeId,
        ports: Vec<PortId>,
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
    Selectable {
        id: EdgeId,
        selectable: Option<bool>,
    },
    Deletable {
        id: EdgeId,
        deletable: Option<bool>,
    },
    Reconnectable {
        id: EdgeId,
        reconnectable: Option<EdgeReconnectable>,
    },
    Endpoints {
        id: EdgeId,
        from: PortId,
        to: PortId,
    },
}

/// Full-fidelity committed graph patch.
///
/// This is the primary commit payload for controlled integrations. It preserves every
/// `GraphOp`, including ports, groups, sticky notes, imports, symbols, and other resources that do
/// not fit into XyFlow's node/edge change arrays.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeGraphPatch {
    /// Reversible transaction committed by the store.
    pub transaction: GraphTransaction,
}

impl NodeGraphPatch {
    pub fn new(transaction: GraphTransaction) -> Self {
        Self { transaction }
    }

    pub fn transaction(&self) -> &GraphTransaction {
        &self.transaction
    }

    pub fn into_transaction(self) -> GraphTransaction {
        self.transaction
    }

    pub fn ops(&self) -> &[GraphOp] {
        &self.transaction.ops
    }

    pub fn is_empty(&self) -> bool {
        self.transaction.is_empty()
    }

    pub fn node_edge_changes(&self) -> NodeGraphChanges {
        NodeGraphChanges::from_transaction(&self.transaction)
    }
}

impl From<GraphTransaction> for NodeGraphPatch {
    fn from(transaction: GraphTransaction) -> Self {
        Self::new(transaction)
    }
}

/// XyFlow-style node/edge projection of a graph patch.
///
/// This adapter is intentionally lossy: it only contains node and edge changes. Use
/// [`NodeGraphPatch`] when a consumer must observe full graph resources such as ports, groups,
/// sticky notes, imports, or symbols.
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
    pub(crate) fn from_transaction(tx: &GraphTransaction) -> Self {
        let mut out = Self::default();
        for op in &tx.ops {
            match op {
                GraphOp::AddNode { id, node } => out.nodes.push(NodeChange::Add {
                    id: *id,
                    node: node.clone(),
                }),
                GraphOp::RemoveNode { id, edges, .. } => {
                    out.nodes.push(NodeChange::Remove { id: *id });
                    for (edge_id, _edge) in edges {
                        out.edges.push(EdgeChange::Remove { id: *edge_id });
                    }
                }
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
                GraphOp::SetNodeSelectable { id, to, .. } => {
                    out.nodes.push(NodeChange::Selectable {
                        id: *id,
                        selectable: *to,
                    })
                }
                GraphOp::SetNodeDraggable { id, to, .. } => out.nodes.push(NodeChange::Draggable {
                    id: *id,
                    draggable: *to,
                }),
                GraphOp::SetNodeConnectable { id, to, .. } => {
                    out.nodes.push(NodeChange::Connectable {
                        id: *id,
                        connectable: *to,
                    })
                }
                GraphOp::SetNodeDeletable { id, to, .. } => out.nodes.push(NodeChange::Deletable {
                    id: *id,
                    deletable: *to,
                }),
                GraphOp::SetNodeParent { id, to, .. } => out.nodes.push(NodeChange::Parent {
                    id: *id,
                    parent: *to,
                }),
                GraphOp::SetNodeExtent { id, to, .. } => out.nodes.push(NodeChange::Extent {
                    id: *id,
                    extent: *to,
                }),
                GraphOp::SetNodeExpandParent { id, to, .. } => {
                    out.nodes.push(NodeChange::ExpandParent {
                        id: *id,
                        expand_parent: *to,
                    })
                }
                GraphOp::SetNodeSize { id, to, .. } => {
                    out.nodes.push(NodeChange::Size { id: *id, size: *to })
                }
                GraphOp::SetNodeHidden { id, to, .. } => out.nodes.push(NodeChange::Hidden {
                    id: *id,
                    hidden: *to,
                }),
                GraphOp::SetNodeCollapsed { id, to, .. } => out.nodes.push(NodeChange::Collapsed {
                    id: *id,
                    collapsed: *to,
                }),
                GraphOp::SetNodeData { id, to, .. } => out.nodes.push(NodeChange::Data {
                    id: *id,
                    data: to.clone(),
                }),
                GraphOp::SetNodePorts { id, to, .. } => out.nodes.push(NodeChange::Ports {
                    id: *id,
                    ports: to.clone(),
                }),
                GraphOp::RemovePort { edges, .. } => {
                    for (edge_id, _edge) in edges {
                        out.edges.push(EdgeChange::Remove { id: *edge_id });
                    }
                }
                GraphOp::AddEdge { id, edge } => out.edges.push(EdgeChange::Add {
                    id: *id,
                    edge: edge.clone(),
                }),
                GraphOp::RemoveEdge { id, .. } => out.edges.push(EdgeChange::Remove { id: *id }),
                GraphOp::SetEdgeKind { id, to, .. } => {
                    out.edges.push(EdgeChange::Kind { id: *id, kind: *to })
                }
                GraphOp::SetEdgeSelectable { id, to, .. } => {
                    out.edges.push(EdgeChange::Selectable {
                        id: *id,
                        selectable: *to,
                    })
                }
                GraphOp::SetEdgeDeletable { id, to, .. } => out.edges.push(EdgeChange::Deletable {
                    id: *id,
                    deletable: *to,
                }),
                GraphOp::SetEdgeReconnectable { id, to, .. } => {
                    out.edges.push(EdgeChange::Reconnectable {
                        id: *id,
                        reconnectable: *to,
                    })
                }
                GraphOp::SetEdgeEndpoints { id, to, .. } => out.edges.push(EdgeChange::Endpoints {
                    id: *id,
                    from: to.from,
                    to: to.to,
                }),
                GraphOp::RemoveGroup { detached, .. } => {
                    for (node_id, _previous_parent) in detached {
                        out.nodes.push(NodeChange::Parent {
                            id: *node_id,
                            parent: None,
                        });
                    }
                }

                // These variants mutate graph resources that are outside the XyFlow-style
                // node/edge change-array contract. Full-fidelity controlled integrations should
                // apply the committed GraphTransaction from on_graph_commit.
                GraphOp::AddPort { .. }
                | GraphOp::SetPortConnectable { .. }
                | GraphOp::SetPortConnectableStart { .. }
                | GraphOp::SetPortConnectableEnd { .. }
                | GraphOp::SetPortType { .. }
                | GraphOp::SetPortData { .. }
                | GraphOp::AddImport { .. }
                | GraphOp::RemoveImport { .. }
                | GraphOp::SetImportAlias { .. }
                | GraphOp::AddSymbol { .. }
                | GraphOp::RemoveSymbol { .. }
                | GraphOp::SetSymbolName { .. }
                | GraphOp::SetSymbolType { .. }
                | GraphOp::SetSymbolDefaultValue { .. }
                | GraphOp::SetSymbolMeta { .. }
                | GraphOp::AddGroup { .. }
                | GraphOp::SetGroupRect { .. }
                | GraphOp::SetGroupTitle { .. }
                | GraphOp::SetGroupColor { .. }
                | GraphOp::AddStickyNote { .. }
                | GraphOp::RemoveStickyNote { .. }
                | GraphOp::SetStickyNoteText { .. }
                | GraphOp::SetStickyNoteRect { .. }
                | GraphOp::SetStickyNoteColor { .. } => {}
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
                NodeChange::Selectable { id, selectable } => {
                    let from = graph
                        .nodes
                        .get(id)
                        .map(|n| n.selectable)
                        .ok_or(ChangesToTransactionError::MissingNode(*id))?;
                    tx.push(GraphOp::SetNodeSelectable {
                        id: *id,
                        from,
                        to: *selectable,
                    });
                }
                NodeChange::Draggable { id, draggable } => {
                    let from = graph
                        .nodes
                        .get(id)
                        .map(|n| n.draggable)
                        .ok_or(ChangesToTransactionError::MissingNode(*id))?;
                    tx.push(GraphOp::SetNodeDraggable {
                        id: *id,
                        from,
                        to: *draggable,
                    });
                }
                NodeChange::Connectable { id, connectable } => {
                    let from = graph
                        .nodes
                        .get(id)
                        .map(|n| n.connectable)
                        .ok_or(ChangesToTransactionError::MissingNode(*id))?;
                    tx.push(GraphOp::SetNodeConnectable {
                        id: *id,
                        from,
                        to: *connectable,
                    });
                }
                NodeChange::Deletable { id, deletable } => {
                    let from = graph
                        .nodes
                        .get(id)
                        .map(|n| n.deletable)
                        .ok_or(ChangesToTransactionError::MissingNode(*id))?;
                    tx.push(GraphOp::SetNodeDeletable {
                        id: *id,
                        from,
                        to: *deletable,
                    });
                }
                NodeChange::Parent { id, parent } => {
                    let from = graph
                        .nodes
                        .get(id)
                        .map(|n| n.parent)
                        .ok_or(ChangesToTransactionError::MissingNode(*id))?;
                    tx.push(GraphOp::SetNodeParent {
                        id: *id,
                        from,
                        to: *parent,
                    });
                }
                NodeChange::Extent { id, extent } => {
                    let from = graph
                        .nodes
                        .get(id)
                        .map(|n| n.extent)
                        .ok_or(ChangesToTransactionError::MissingNode(*id))?;
                    tx.push(GraphOp::SetNodeExtent {
                        id: *id,
                        from,
                        to: *extent,
                    });
                }
                NodeChange::ExpandParent { id, expand_parent } => {
                    let from = graph
                        .nodes
                        .get(id)
                        .map(|n| n.expand_parent)
                        .ok_or(ChangesToTransactionError::MissingNode(*id))?;
                    tx.push(GraphOp::SetNodeExpandParent {
                        id: *id,
                        from,
                        to: *expand_parent,
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
                NodeChange::Hidden { id, hidden } => {
                    let from = graph
                        .nodes
                        .get(id)
                        .map(|n| n.hidden)
                        .ok_or(ChangesToTransactionError::MissingNode(*id))?;
                    tx.push(GraphOp::SetNodeHidden {
                        id: *id,
                        from,
                        to: *hidden,
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
                NodeChange::Ports { id, ports } => {
                    let from = graph
                        .nodes
                        .get(id)
                        .map(|n| n.ports.clone())
                        .ok_or(ChangesToTransactionError::MissingNode(*id))?;
                    tx.push(GraphOp::SetNodePorts {
                        id: *id,
                        from,
                        to: ports.clone(),
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
                EdgeChange::Selectable { id, selectable } => {
                    let from = graph
                        .edges
                        .get(id)
                        .map(|e| e.selectable)
                        .ok_or(ChangesToTransactionError::MissingEdge(*id))?;
                    tx.push(GraphOp::SetEdgeSelectable {
                        id: *id,
                        from,
                        to: *selectable,
                    });
                }
                EdgeChange::Deletable { id, deletable } => {
                    let from = graph
                        .edges
                        .get(id)
                        .map(|e| e.deletable)
                        .ok_or(ChangesToTransactionError::MissingEdge(*id))?;
                    tx.push(GraphOp::SetEdgeDeletable {
                        id: *id,
                        from,
                        to: *deletable,
                    });
                }
                EdgeChange::Reconnectable { id, reconnectable } => {
                    let from = graph
                        .edges
                        .get(id)
                        .map(|e| e.reconnectable)
                        .ok_or(ChangesToTransactionError::MissingEdge(*id))?;
                    tx.push(GraphOp::SetEdgeReconnectable {
                        id: *id,
                        from,
                        to: *reconnectable,
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
