use std::collections::HashMap;

use crate::core::{CanvasPoint, NodeId};
use crate::ops::GraphOp;
use crate::ui::canvas::state::GroupDrag;

pub(super) fn build_group_drag_ops(drag: &GroupDrag) -> Vec<GraphOp> {
    let mut ops: Vec<GraphOp> = Vec::new();
    if drag.current_rect != drag.start_rect {
        ops.push(GraphOp::SetGroupRect {
            id: drag.group,
            from: drag.start_rect,
            to: drag.current_rect,
        });
    }

    let current_nodes: HashMap<NodeId, CanvasPoint> = drag.current_nodes.iter().copied().collect();
    for (id, start) in &drag.nodes {
        let end = current_nodes.get(id).copied().unwrap_or(*start);
        if end != *start {
            ops.push(GraphOp::SetNodePos {
                id: *id,
                from: *start,
                to: end,
            });
        }
    }
    ops
}

#[cfg(test)]
mod tests;
