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
            GraphOp::SetNodeParent {
                id: a, to: last_to, ..
            },
            GraphOp::SetNodeParent { id: b, from, to },
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
            GraphOp::SetEdgeKind {
                id: a, to: last_to, ..
            },
            GraphOp::SetEdgeKind { id: b, from, to },
        ) if a == b && last_to == from => {
            *last_to = *to;
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
        _ => false,
    }
}

fn op_is_noop(op: &GraphOp) -> bool {
    match op {
        GraphOp::SetNodePos { from, to, .. } => from == to,
        GraphOp::SetNodeKind { from, to, .. } => from == to,
        GraphOp::SetNodeKindVersion { from, to, .. } => from == to,
        GraphOp::SetNodeParent { from, to, .. } => from == to,
        GraphOp::SetNodeSize { from, to, .. } => from == to,
        GraphOp::SetNodeCollapsed { from, to, .. } => from == to,
        GraphOp::SetNodePorts { from, to, .. } => from == to,
        GraphOp::SetNodeData { from, to, .. } => from == to,

        GraphOp::SetEdgeKind { from, to, .. } => from == to,
        GraphOp::SetEdgeEndpoints { from, to, .. } => from == to,

        GraphOp::SetImportAlias { from, to, .. } => from == to,

        GraphOp::SetSymbolName { from, to, .. } => from == to,
        GraphOp::SetSymbolType { from, to, .. } => from == to,
        GraphOp::SetSymbolDefaultValue { from, to, .. } => from == to,
        GraphOp::SetSymbolMeta { from, to, .. } => from == to,

        GraphOp::SetGroupRect { from, to, .. } => from == to,
        GraphOp::SetGroupTitle { from, to, .. } => from == to,

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
