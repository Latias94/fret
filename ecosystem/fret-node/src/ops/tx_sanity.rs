use crate::core::{CanvasPoint, CanvasRect, CanvasSize};

use super::{EdgeEndpoints, GraphOp, GraphTransaction};

pub(crate) fn find_non_finite_in_tx(tx: &GraphTransaction) -> Option<(String, String)> {
    for (ix, op) in tx.ops.iter().enumerate() {
        if let Some(field) = op_non_finite_field(op) {
            return Some((
                "tx.non_finite".to_string(),
                format!("transaction contains non-finite geometry at op[{ix}] ({field})"),
            ));
        }
    }
    None
}

fn op_non_finite_field(op: &GraphOp) -> Option<&'static str> {
    fn point_is_finite(p: CanvasPoint) -> bool {
        p.x.is_finite() && p.y.is_finite()
    }

    fn size_is_finite(s: CanvasSize) -> bool {
        s.width.is_finite() && s.height.is_finite()
    }

    fn rect_is_finite(r: CanvasRect) -> bool {
        point_is_finite(r.origin) && size_is_finite(r.size)
    }

    fn endpoints_is_finite(_e: EdgeEndpoints) -> bool {
        true
    }

    match op {
        GraphOp::AddNode { node, .. } => node
            .size
            .and_then(|s| (!size_is_finite(s)).then_some("AddNode.node.size"))
            .or_else(|| (!point_is_finite(node.pos)).then_some("AddNode.node.pos")),
        GraphOp::AddGroup { group, .. } => (!rect_is_finite(group.rect)).then_some("AddGroup.rect"),
        GraphOp::AddStickyNote { note, .. } => {
            (!rect_is_finite(note.rect)).then_some("AddStickyNote.rect")
        }

        GraphOp::SetNodePos { to, .. } => (!point_is_finite(*to)).then_some("SetNodePos.to"),
        GraphOp::SetGroupRect { to, .. } => (!rect_is_finite(*to)).then_some("SetGroupRect.to"),
        GraphOp::SetNodeSize { to, .. } => {
            to.and_then(|s| (!size_is_finite(s)).then_some("SetNodeSize.to"))
        }

        GraphOp::SetEdgeEndpoints { from, to, .. } => (!endpoints_is_finite(*from))
            .then_some("SetEdgeEndpoints.from")
            .or_else(|| (!endpoints_is_finite(*to)).then_some("SetEdgeEndpoints.to")),

        GraphOp::AddImport { .. }
        | GraphOp::RemoveImport { .. }
        | GraphOp::SetImportAlias { .. }
        | GraphOp::SetSymbolName { .. }
        | GraphOp::SetSymbolType { .. }
        | GraphOp::SetSymbolDefaultValue { .. }
        | GraphOp::RemoveNode { .. }
        | GraphOp::SetNodeKind { .. }
        | GraphOp::SetNodeKindVersion { .. }
        | GraphOp::SetNodeParent { .. }
        | GraphOp::SetNodeCollapsed { .. }
        | GraphOp::SetNodePorts { .. }
        | GraphOp::SetNodeData { .. }
        | GraphOp::SetGroupTitle { .. }
        | GraphOp::AddPort { .. }
        | GraphOp::RemovePort { .. }
        | GraphOp::AddEdge { .. }
        | GraphOp::RemoveEdge { .. }
        | GraphOp::SetEdgeKind { .. }
        | GraphOp::AddSymbol { .. }
        | GraphOp::RemoveSymbol { .. }
        | GraphOp::SetSymbolMeta { .. }
        | GraphOp::RemoveGroup { .. }
        | GraphOp::RemoveStickyNote { .. } => None,
    }
}

pub(crate) fn find_invalid_size_in_tx(tx: &GraphTransaction) -> Option<(String, String)> {
    for (ix, op) in tx.ops.iter().enumerate() {
        if let Some(field) = op_invalid_size_field(op) {
            return Some((
                "tx.invalid_size".to_string(),
                format!("transaction contains invalid size at op[{ix}] ({field})"),
            ));
        }
    }
    None
}

fn op_invalid_size_field(op: &GraphOp) -> Option<&'static str> {
    fn size_is_valid(s: CanvasSize) -> bool {
        s.width.is_finite() && s.height.is_finite() && s.width > 0.0 && s.height > 0.0
    }

    match op {
        GraphOp::AddNode { node, .. } => node
            .size
            .and_then(|s| (!size_is_valid(s)).then_some("AddNode.node.size")),
        GraphOp::SetNodeSize { to, .. } => {
            to.and_then(|s| (!size_is_valid(s)).then_some("SetNodeSize.to"))
        }
        _ => None,
    }
}
