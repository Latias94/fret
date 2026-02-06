//! Edit history (undo/redo) for graph transactions.
//!
//! The history stores committed (post-concretization) transactions and derives inverse transactions
//! by inverting operations in reverse order. It is intentionally headless and can be used by UI and
//! non-UI drivers.

use crate::ops::{GraphOp, GraphTransaction};

/// Maximum number of transactions retained by default.
pub const DEFAULT_HISTORY_LIMIT: usize = 256;

/// A simple undo/redo history for committed graph transactions.
#[derive(Debug, Clone)]
pub struct GraphHistory {
    limit: usize,
    undo: Vec<GraphTransaction>,
    redo: Vec<GraphTransaction>,
}

impl Default for GraphHistory {
    fn default() -> Self {
        Self::new(DEFAULT_HISTORY_LIMIT)
    }
}

impl GraphHistory {
    pub fn new(limit: usize) -> Self {
        Self {
            limit: limit.max(1),
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }

    pub fn redo_len(&self) -> usize {
        self.redo.len()
    }

    /// Records a committed transaction (original + derived concretization ops).
    pub fn record(&mut self, tx: GraphTransaction) {
        let tx = super::normalize_transaction(tx);
        if tx.ops.is_empty() {
            return;
        }
        self.undo.push(tx);
        self.redo.clear();
        if self.undo.len() > self.limit {
            let overflow = self.undo.len() - self.limit;
            self.undo.drain(0..overflow);
        }
    }

    /// Undoes the last recorded transaction by applying its inverse transaction.
    ///
    /// The `apply` closure is responsible for applying the transaction to the graph and returning
    /// the committed transaction (including any derived ops produced by the profile pipeline).
    pub fn undo<E>(
        &mut self,
        mut apply: impl FnMut(&GraphTransaction) -> Result<GraphTransaction, E>,
    ) -> Result<bool, E> {
        let Some(tx) = self.undo.pop() else {
            return Ok(false);
        };

        let inverse = invert_transaction(&tx);
        match apply(&inverse) {
            Ok(committed) => {
                let redo_tx = invert_transaction(&committed);
                self.redo.push(redo_tx);
                Ok(true)
            }
            Err(err) => {
                self.undo.push(tx);
                Err(err)
            }
        }
    }

    /// Redoes the last undone transaction.
    pub fn redo<E>(
        &mut self,
        mut apply: impl FnMut(&GraphTransaction) -> Result<GraphTransaction, E>,
    ) -> Result<bool, E> {
        let Some(tx) = self.redo.pop() else {
            return Ok(false);
        };

        match apply(&tx) {
            Ok(committed) => {
                self.undo.push(committed);
                Ok(true)
            }
            Err(err) => {
                self.redo.push(tx);
                Err(err)
            }
        }
    }
}

/// Builds an inverse transaction that restores the graph state before `tx`.
pub fn invert_transaction(tx: &GraphTransaction) -> GraphTransaction {
    let mut out = GraphTransaction::new();
    for op in tx.ops.iter().rev() {
        out.ops.extend(invert_op(op));
    }
    out
}

fn invert_op(op: &GraphOp) -> Vec<GraphOp> {
    match op {
        GraphOp::AddNode { id, node } => vec![GraphOp::RemoveNode {
            id: *id,
            node: node.clone(),
            ports: Vec::new(),
            edges: Vec::new(),
        }],
        GraphOp::RemoveNode {
            id,
            node,
            ports,
            edges,
        } => {
            let mut out: Vec<GraphOp> = Vec::new();
            out.push(GraphOp::AddNode {
                id: *id,
                node: node.clone(),
            });
            for (port_id, port) in ports {
                out.push(GraphOp::AddPort {
                    id: *port_id,
                    port: port.clone(),
                });
            }
            for (edge_id, edge) in edges {
                out.push(GraphOp::AddEdge {
                    id: *edge_id,
                    edge: edge.clone(),
                });
            }
            out
        }
        GraphOp::SetNodePos { id, from, to } => vec![GraphOp::SetNodePos {
            id: *id,
            from: *to,
            to: *from,
        }],
        GraphOp::SetNodeKind { id, from, to } => vec![GraphOp::SetNodeKind {
            id: *id,
            from: to.clone(),
            to: from.clone(),
        }],
        GraphOp::SetNodeKindVersion { id, from, to } => vec![GraphOp::SetNodeKindVersion {
            id: *id,
            from: *to,
            to: *from,
        }],
        GraphOp::SetNodeSelectable { id, from, to } => vec![GraphOp::SetNodeSelectable {
            id: *id,
            from: *to,
            to: *from,
        }],
        GraphOp::SetNodeDraggable { id, from, to } => vec![GraphOp::SetNodeDraggable {
            id: *id,
            from: *to,
            to: *from,
        }],
        GraphOp::SetNodeConnectable { id, from, to } => vec![GraphOp::SetNodeConnectable {
            id: *id,
            from: *to,
            to: *from,
        }],
        GraphOp::SetNodeDeletable { id, from, to } => vec![GraphOp::SetNodeDeletable {
            id: *id,
            from: *to,
            to: *from,
        }],
        GraphOp::SetNodeParent { id, from, to } => vec![GraphOp::SetNodeParent {
            id: *id,
            from: *to,
            to: *from,
        }],
        GraphOp::SetNodeExtent { id, from, to } => vec![GraphOp::SetNodeExtent {
            id: *id,
            from: *to,
            to: *from,
        }],
        GraphOp::SetNodeExpandParent { id, from, to } => vec![GraphOp::SetNodeExpandParent {
            id: *id,
            from: *to,
            to: *from,
        }],
        GraphOp::SetNodeSize { id, from, to } => vec![GraphOp::SetNodeSize {
            id: *id,
            from: *to,
            to: *from,
        }],
        GraphOp::SetNodeHidden { id, from, to } => vec![GraphOp::SetNodeHidden {
            id: *id,
            from: *to,
            to: *from,
        }],
        GraphOp::SetNodeCollapsed { id, from, to } => vec![GraphOp::SetNodeCollapsed {
            id: *id,
            from: *to,
            to: *from,
        }],
        GraphOp::SetNodePorts { id, from, to } => vec![GraphOp::SetNodePorts {
            id: *id,
            from: to.clone(),
            to: from.clone(),
        }],
        GraphOp::SetNodeData { id, from, to } => vec![GraphOp::SetNodeData {
            id: *id,
            from: to.clone(),
            to: from.clone(),
        }],

        GraphOp::AddPort { id, port } => vec![GraphOp::RemovePort {
            id: *id,
            port: port.clone(),
            edges: Vec::new(),
        }],
        GraphOp::RemovePort { id, port, edges } => {
            let mut out: Vec<GraphOp> = Vec::new();
            out.push(GraphOp::AddPort {
                id: *id,
                port: port.clone(),
            });
            for (edge_id, edge) in edges {
                out.push(GraphOp::AddEdge {
                    id: *edge_id,
                    edge: edge.clone(),
                });
            }
            out
        }
        GraphOp::SetPortConnectable { id, from, to } => vec![GraphOp::SetPortConnectable {
            id: *id,
            from: *to,
            to: *from,
        }],
        GraphOp::SetPortConnectableStart { id, from, to } => {
            vec![GraphOp::SetPortConnectableStart {
                id: *id,
                from: *to,
                to: *from,
            }]
        }
        GraphOp::SetPortConnectableEnd { id, from, to } => vec![GraphOp::SetPortConnectableEnd {
            id: *id,
            from: *to,
            to: *from,
        }],
        GraphOp::SetPortType { id, from, to } => vec![GraphOp::SetPortType {
            id: *id,
            from: to.clone(),
            to: from.clone(),
        }],
        GraphOp::SetPortData { id, from, to } => vec![GraphOp::SetPortData {
            id: *id,
            from: to.clone(),
            to: from.clone(),
        }],

        GraphOp::AddEdge { id, edge } => vec![GraphOp::RemoveEdge {
            id: *id,
            edge: edge.clone(),
        }],
        GraphOp::RemoveEdge { id, edge } => vec![GraphOp::AddEdge {
            id: *id,
            edge: edge.clone(),
        }],
        GraphOp::SetEdgeKind { id, from, to } => vec![GraphOp::SetEdgeKind {
            id: *id,
            from: *to,
            to: *from,
        }],
        GraphOp::SetEdgeSelectable { id, from, to } => vec![GraphOp::SetEdgeSelectable {
            id: *id,
            from: *to,
            to: *from,
        }],
        GraphOp::SetEdgeDeletable { id, from, to } => vec![GraphOp::SetEdgeDeletable {
            id: *id,
            from: *to,
            to: *from,
        }],
        GraphOp::SetEdgeReconnectable { id, from, to } => vec![GraphOp::SetEdgeReconnectable {
            id: *id,
            from: to.clone(),
            to: from.clone(),
        }],
        GraphOp::SetEdgeEndpoints { id, from, to } => vec![GraphOp::SetEdgeEndpoints {
            id: *id,
            from: *to,
            to: *from,
        }],

        GraphOp::AddImport { id, import } => vec![GraphOp::RemoveImport {
            id: *id,
            import: import.clone(),
        }],
        GraphOp::RemoveImport { id, import } => vec![GraphOp::AddImport {
            id: *id,
            import: import.clone(),
        }],
        GraphOp::SetImportAlias { id, from, to } => vec![GraphOp::SetImportAlias {
            id: *id,
            from: to.clone(),
            to: from.clone(),
        }],

        GraphOp::AddSymbol { id, symbol } => vec![GraphOp::RemoveSymbol {
            id: *id,
            symbol: symbol.clone(),
        }],
        GraphOp::RemoveSymbol { id, symbol } => vec![GraphOp::AddSymbol {
            id: *id,
            symbol: symbol.clone(),
        }],
        GraphOp::SetSymbolName { id, from, to } => vec![GraphOp::SetSymbolName {
            id: *id,
            from: to.clone(),
            to: from.clone(),
        }],
        GraphOp::SetSymbolType { id, from, to } => vec![GraphOp::SetSymbolType {
            id: *id,
            from: to.clone(),
            to: from.clone(),
        }],
        GraphOp::SetSymbolDefaultValue { id, from, to } => vec![GraphOp::SetSymbolDefaultValue {
            id: *id,
            from: to.clone(),
            to: from.clone(),
        }],
        GraphOp::SetSymbolMeta { id, from, to } => vec![GraphOp::SetSymbolMeta {
            id: *id,
            from: to.clone(),
            to: from.clone(),
        }],

        GraphOp::AddGroup { id, group } => vec![GraphOp::RemoveGroup {
            id: *id,
            group: group.clone(),
            detached: Vec::new(),
        }],
        GraphOp::RemoveGroup {
            id,
            group,
            detached,
        } => {
            let mut out: Vec<GraphOp> = Vec::new();
            out.push(GraphOp::AddGroup {
                id: *id,
                group: group.clone(),
            });
            for (node_id, parent) in detached {
                out.push(GraphOp::SetNodeParent {
                    id: *node_id,
                    from: None,
                    to: *parent,
                });
            }
            out
        }
        GraphOp::SetGroupRect { id, from, to } => vec![GraphOp::SetGroupRect {
            id: *id,
            from: *to,
            to: *from,
        }],
        GraphOp::SetGroupTitle { id, from, to } => vec![GraphOp::SetGroupTitle {
            id: *id,
            from: to.clone(),
            to: from.clone(),
        }],
        GraphOp::SetGroupColor { id, from, to } => vec![GraphOp::SetGroupColor {
            id: *id,
            from: to.clone(),
            to: from.clone(),
        }],

        GraphOp::AddStickyNote { id, note } => vec![GraphOp::RemoveStickyNote {
            id: *id,
            note: note.clone(),
        }],
        GraphOp::RemoveStickyNote { id, note } => vec![GraphOp::AddStickyNote {
            id: *id,
            note: note.clone(),
        }],
        GraphOp::SetStickyNoteText { id, from, to } => vec![GraphOp::SetStickyNoteText {
            id: *id,
            from: to.clone(),
            to: from.clone(),
        }],
        GraphOp::SetStickyNoteRect { id, from, to } => vec![GraphOp::SetStickyNoteRect {
            id: *id,
            from: *to,
            to: *from,
        }],
        GraphOp::SetStickyNoteColor { id, from, to } => vec![GraphOp::SetStickyNoteColor {
            id: *id,
            from: to.clone(),
            to: from.clone(),
        }],
    }
}
