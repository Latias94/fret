use super::{GraphOp, GraphTransaction};

pub(crate) fn normalize_transaction(mut tx: GraphTransaction) -> GraphTransaction {
    tx.ops.retain(|op| !op_is_noop(op));
    tx
}

fn op_is_noop(op: &GraphOp) -> bool {
    match op {
        GraphOp::SetNodePos { from, to, .. } => from == to,
        GraphOp::SetNodeParent { from, to, .. } => from == to,
        GraphOp::SetNodeSize { from, to, .. } => from == to,
        GraphOp::SetNodeCollapsed { from, to, .. } => from == to,
        GraphOp::SetNodePorts { from, to, .. } => from == to,
        GraphOp::SetNodeData { from, to, .. } => from == to,

        GraphOp::SetEdgeKind { from, to, .. } => from == to,
        GraphOp::SetEdgeEndpoints { from, to, .. } => from == to,

        GraphOp::SetSymbolMeta { from, to, .. } => from == to,

        GraphOp::SetGroupRect { from, to, .. } => from == to,

        GraphOp::AddNode { .. }
        | GraphOp::RemoveNode { .. }
        | GraphOp::AddPort { .. }
        | GraphOp::RemovePort { .. }
        | GraphOp::AddEdge { .. }
        | GraphOp::RemoveEdge { .. }
        | GraphOp::AddSymbol { .. }
        | GraphOp::RemoveSymbol { .. }
        | GraphOp::AddGroup { .. }
        | GraphOp::RemoveGroup { .. }
        | GraphOp::AddStickyNote { .. }
        | GraphOp::RemoveStickyNote { .. } => false,
    }
}
