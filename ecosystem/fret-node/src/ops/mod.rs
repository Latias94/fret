//! Undoable graph edit operations.

mod apply;
mod build;
mod fragment;
mod history;

use serde::{Deserialize, Serialize};

use crate::core::{
    CanvasPoint, Edge, EdgeId, EdgeKind, Group, GroupId, Node, NodeId, Port, PortId, StickyNote,
    StickyNoteId, Symbol, SymbolId,
};

pub use apply::{ApplyError, apply_op, apply_transaction};
pub use build::GraphOpBuilderExt;
pub use fragment::{GraphFragment, IdRemapSeed, IdRemapper, PasteTuning};
pub use history::{DEFAULT_HISTORY_LIMIT, GraphHistory, invert_transaction};

/// Edge endpoint pair (from/to ports).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeEndpoints {
    pub from: PortId,
    pub to: PortId,
}

/// A reversible edit operation.
///
/// Destructive variants carry the removed data so the operation can be inverted for undo/redo.
/// Higher-level tools should batch multiple ops into a single transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum GraphOp {
    /// Adds a node.
    AddNode { id: NodeId, node: Node },
    /// Removes a node.
    ///
    /// This operation is expected to remove associated ports and edges as well.
    RemoveNode {
        id: NodeId,
        node: Node,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        ports: Vec<(PortId, Port)>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        edges: Vec<(EdgeId, Edge)>,
    },
    /// Sets a node position.
    SetNodePos {
        id: NodeId,
        from: CanvasPoint,
        to: CanvasPoint,
    },
    /// Sets a node collapsed state.
    SetNodeCollapsed { id: NodeId, from: bool, to: bool },
    /// Sets a node's port ordering.
    SetNodePorts {
        id: NodeId,
        from: Vec<PortId>,
        to: Vec<PortId>,
    },

    /// Adds a port.
    AddPort { id: PortId, port: Port },
    /// Removes a port.
    ///
    /// This operation is expected to remove associated edges as well.
    RemovePort {
        id: PortId,
        port: Port,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        edges: Vec<(EdgeId, Edge)>,
    },

    /// Adds an edge.
    AddEdge { id: EdgeId, edge: Edge },
    /// Removes an edge.
    RemoveEdge { id: EdgeId, edge: Edge },
    /// Sets an edge kind.
    SetEdgeKind {
        id: EdgeId,
        from: EdgeKind,
        to: EdgeKind,
    },
    /// Sets an edge's endpoints (preserving edge identity for reconnection workflows).
    SetEdgeEndpoints {
        id: EdgeId,
        from: EdgeEndpoints,
        to: EdgeEndpoints,
    },

    /// Adds a symbol.
    AddSymbol { id: SymbolId, symbol: Symbol },
    /// Removes a symbol.
    RemoveSymbol { id: SymbolId, symbol: Symbol },
    /// Updates a symbol metadata payload (domain-owned).
    SetSymbolMeta {
        id: SymbolId,
        from: serde_json::Value,
        to: serde_json::Value,
    },

    /// Adds a group.
    AddGroup { id: GroupId, group: Group },
    /// Removes a group.
    RemoveGroup { id: GroupId, group: Group },

    /// Adds a sticky note.
    AddStickyNote { id: StickyNoteId, note: StickyNote },
    /// Removes a sticky note.
    RemoveStickyNote { id: StickyNoteId, note: StickyNote },
}

/// A batch of edit operations that should be applied and undone as one unit.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphTransaction {
    /// Optional human-readable label for history UI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Operations in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ops: Vec<GraphOp>,
}

impl GraphTransaction {
    /// Creates an empty transaction.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Pushes an op.
    pub fn push(&mut self, op: GraphOp) {
        self.ops.push(op);
    }
}

#[cfg(test)]
mod tests;
