//! Undoable graph edit operations.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{
    CanvasPoint, Edge, EdgeId, EdgeKind, Group, GroupId, Node, NodeId, Port, PortId, StickyNote,
    StickyNoteId, Symbol, SymbolId,
};

/// A minimal, reversible edit operation.
///
/// Higher-level tools should batch multiple ops into a single transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum GraphOp {
    /// Adds a node.
    AddNode { id: NodeId, node: Node },
    /// Removes a node.
    RemoveNode { id: NodeId },
    /// Sets a node position.
    SetNodePos { id: NodeId, pos: CanvasPoint },
    /// Sets a node collapsed state.
    SetNodeCollapsed { id: NodeId, collapsed: bool },

    /// Adds a port.
    AddPort { id: PortId, port: Port },
    /// Removes a port.
    RemovePort { id: PortId },

    /// Adds an edge.
    AddEdge { id: EdgeId, edge: Edge },
    /// Removes an edge.
    RemoveEdge { id: EdgeId },
    /// Sets an edge kind.
    SetEdgeKind { id: EdgeId, kind: EdgeKind },

    /// Adds a symbol.
    AddSymbol { id: SymbolId, symbol: Symbol },
    /// Removes a symbol.
    RemoveSymbol { id: SymbolId },
    /// Updates a symbol payload (domain-owned metadata).
    SetSymbolMeta { id: SymbolId, meta: Value },

    /// Adds a group.
    AddGroup { id: GroupId, group: Group },
    /// Removes a group.
    RemoveGroup { id: GroupId },

    /// Adds a sticky note.
    AddStickyNote { id: StickyNoteId, note: StickyNote },
    /// Removes a sticky note.
    RemoveStickyNote { id: StickyNoteId },
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
