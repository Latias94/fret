use crate::ops::GraphOp;

pub(super) fn commit_label(ops: &[GraphOp]) -> &'static str {
    if ops
        .iter()
        .all(|op| matches!(op, GraphOp::SetNodeParent { .. }))
    {
        if ops.len() == 1 {
            "Set Node Parent"
        } else {
            "Set Node Parents"
        }
    } else if ops.len() == 1 {
        "Move Node"
    } else {
        "Move Nodes"
    }
}
