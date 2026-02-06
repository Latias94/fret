use super::{GraphOp, GraphTransaction};

pub(crate) fn normalize_transaction(mut tx: GraphTransaction) -> GraphTransaction {
    tx.ops = coalesce_setter_chains(tx.ops);
    tx.ops.retain(|op| !op_is_noop(op));
    tx
}

fn coalesce_setter_chains(ops: Vec<GraphOp>) -> Vec<GraphOp> {
    let mut out = Vec::with_capacity(ops.len());
    for op in ops {
        if let Some(last) = out.last_mut()
            && try_coalesce_setter(last, &op)
        {
            continue;
        }
        out.push(op);
    }
    out
}

fn try_coalesce_setter(last: &mut GraphOp, next: &GraphOp) -> bool {
    match (last, next) {
        (
            GraphOp::SetNodePos {
                id: a, to: last_to, ..
            },
            GraphOp::SetNodePos { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetNodeKind {
                id: a, to: last_to, ..
            },
            GraphOp::SetNodeKind { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = to.clone();
            true
        }
        (
            GraphOp::SetNodeKindVersion {
                id: a, to: last_to, ..
            },
            GraphOp::SetNodeKindVersion { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetNodeSelectable {
                id: a, to: last_to, ..
            },
            GraphOp::SetNodeSelectable { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetNodeDraggable {
                id: a, to: last_to, ..
            },
            GraphOp::SetNodeDraggable { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetNodeConnectable {
                id: a, to: last_to, ..
            },
            GraphOp::SetNodeConnectable { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetNodeDeletable {
                id: a, to: last_to, ..
            },
            GraphOp::SetNodeDeletable { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetNodeParent {
                id: a, to: last_to, ..
            },
            GraphOp::SetNodeParent { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetNodeExtent {
                id: a, to: last_to, ..
            },
            GraphOp::SetNodeExtent { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetNodeExpandParent {
                id: a, to: last_to, ..
            },
            GraphOp::SetNodeExpandParent { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetNodeSize {
                id: a, to: last_to, ..
            },
            GraphOp::SetNodeSize { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetNodeHidden {
                id: a, to: last_to, ..
            },
            GraphOp::SetNodeHidden { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetNodeCollapsed {
                id: a, to: last_to, ..
            },
            GraphOp::SetNodeCollapsed { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetNodePorts {
                id: a, to: last_to, ..
            },
            GraphOp::SetNodePorts { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = to.clone();
            true
        }
        (
            GraphOp::SetNodeData {
                id: a, to: last_to, ..
            },
            GraphOp::SetNodeData { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = to.clone();
            true
        }
        (
            GraphOp::SetPortConnectable {
                id: a, to: last_to, ..
            },
            GraphOp::SetPortConnectable { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetPortConnectableStart {
                id: a, to: last_to, ..
            },
            GraphOp::SetPortConnectableStart { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetPortConnectableEnd {
                id: a, to: last_to, ..
            },
            GraphOp::SetPortConnectableEnd { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetPortType {
                id: a, to: last_to, ..
            },
            GraphOp::SetPortType { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = to.clone();
            true
        }
        (
            GraphOp::SetPortData {
                id: a, to: last_to, ..
            },
            GraphOp::SetPortData { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = to.clone();
            true
        }
        (
            GraphOp::SetEdgeKind {
                id: a, to: last_to, ..
            },
            GraphOp::SetEdgeKind { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetEdgeSelectable {
                id: a, to: last_to, ..
            },
            GraphOp::SetEdgeSelectable { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetEdgeDeletable {
                id: a, to: last_to, ..
            },
            GraphOp::SetEdgeDeletable { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetEdgeReconnectable {
                id: a, to: last_to, ..
            },
            GraphOp::SetEdgeReconnectable { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = to.clone();
            true
        }
        (
            GraphOp::SetEdgeEndpoints {
                id: a, to: last_to, ..
            },
            GraphOp::SetEdgeEndpoints { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetImportAlias {
                id: a, to: last_to, ..
            },
            GraphOp::SetImportAlias { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = to.clone();
            true
        }
        (
            GraphOp::SetSymbolMeta {
                id: a, to: last_to, ..
            },
            GraphOp::SetSymbolMeta { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = to.clone();
            true
        }
        (
            GraphOp::SetSymbolName {
                id: a, to: last_to, ..
            },
            GraphOp::SetSymbolName { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = to.clone();
            true
        }
        (
            GraphOp::SetSymbolType {
                id: a, to: last_to, ..
            },
            GraphOp::SetSymbolType { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = to.clone();
            true
        }
        (
            GraphOp::SetSymbolDefaultValue {
                id: a, to: last_to, ..
            },
            GraphOp::SetSymbolDefaultValue { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = to.clone();
            true
        }
        (
            GraphOp::SetGroupRect {
                id: a, to: last_to, ..
            },
            GraphOp::SetGroupRect { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetGroupColor {
                id: a, to: last_to, ..
            },
            GraphOp::SetGroupColor { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = to.clone();
            true
        }
        (
            GraphOp::SetStickyNoteText {
                id: a, to: last_to, ..
            },
            GraphOp::SetStickyNoteText { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = to.clone();
            true
        }
        (
            GraphOp::SetStickyNoteRect {
                id: a, to: last_to, ..
            },
            GraphOp::SetStickyNoteRect { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
            true
        }
        (
            GraphOp::SetStickyNoteColor {
                id: a, to: last_to, ..
            },
            GraphOp::SetStickyNoteColor { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = to.clone();
            true
        }
        _ => false,
    }
}

fn op_is_noop(op: &GraphOp) -> bool {
    match op {
        GraphOp::SetNodePos { from, to, .. } => from == to,
        GraphOp::SetNodeKind { from, to, .. } => from == to,
        GraphOp::SetNodeKindVersion { from, to, .. } => from == to,
        GraphOp::SetNodeSelectable { from, to, .. } => from == to,
        GraphOp::SetNodeDraggable { from, to, .. } => from == to,
        GraphOp::SetNodeConnectable { from, to, .. } => from == to,
        GraphOp::SetNodeDeletable { from, to, .. } => from == to,
        GraphOp::SetNodeParent { from, to, .. } => from == to,
        GraphOp::SetNodeExtent { from, to, .. } => from == to,
        GraphOp::SetNodeExpandParent { from, to, .. } => from == to,
        GraphOp::SetNodeSize { from, to, .. } => from == to,
        GraphOp::SetNodeHidden { from, to, .. } => from == to,
        GraphOp::SetNodeCollapsed { from, to, .. } => from == to,
        GraphOp::SetNodePorts { from, to, .. } => from == to,
        GraphOp::SetNodeData { from, to, .. } => from == to,

        GraphOp::SetPortConnectable { from, to, .. } => from == to,
        GraphOp::SetPortConnectableStart { from, to, .. } => from == to,
        GraphOp::SetPortConnectableEnd { from, to, .. } => from == to,
        GraphOp::SetPortType { from, to, .. } => from == to,
        GraphOp::SetPortData { from, to, .. } => from == to,

        GraphOp::SetEdgeKind { from, to, .. } => from == to,
        GraphOp::SetEdgeSelectable { from, to, .. } => from == to,
        GraphOp::SetEdgeDeletable { from, to, .. } => from == to,
        GraphOp::SetEdgeReconnectable { from, to, .. } => from == to,
        GraphOp::SetEdgeEndpoints { from, to, .. } => from == to,

        GraphOp::SetImportAlias { from, to, .. } => from == to,

        GraphOp::SetSymbolName { from, to, .. } => from == to,
        GraphOp::SetSymbolType { from, to, .. } => from == to,
        GraphOp::SetSymbolDefaultValue { from, to, .. } => from == to,
        GraphOp::SetSymbolMeta { from, to, .. } => from == to,

        GraphOp::SetGroupRect { from, to, .. } => from == to,
        GraphOp::SetGroupTitle { from, to, .. } => from == to,
        GraphOp::SetGroupColor { from, to, .. } => from == to,
        GraphOp::SetStickyNoteText { from, to, .. } => from == to,
        GraphOp::SetStickyNoteRect { from, to, .. } => from == to,
        GraphOp::SetStickyNoteColor { from, to, .. } => from == to,

        GraphOp::AddNode { .. }
        | GraphOp::RemoveNode { .. }
        | GraphOp::AddPort { .. }
        | GraphOp::RemovePort { .. }
        | GraphOp::AddEdge { .. }
        | GraphOp::RemoveEdge { .. }
        | GraphOp::AddImport { .. }
        | GraphOp::RemoveImport { .. }
        | GraphOp::AddSymbol { .. }
        | GraphOp::RemoveSymbol { .. }
        | GraphOp::AddGroup { .. }
        | GraphOp::RemoveGroup { .. }
        | GraphOp::AddStickyNote { .. }
        | GraphOp::RemoveStickyNote { .. } => false,
    }
}
