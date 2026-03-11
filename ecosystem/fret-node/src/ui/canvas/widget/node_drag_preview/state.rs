use crate::core::{CanvasPoint, CanvasRect, GroupId, NodeId as GraphNodeId};
use crate::ui::canvas::state::NodeDrag;

pub(super) fn update_drag_preview_state(
    drag: &mut NodeDrag,
    next_nodes: Vec<(GraphNodeId, CanvasPoint)>,
    next_groups: Vec<(GroupId, CanvasRect)>,
) {
    if drag.current_nodes != next_nodes || drag.current_groups != next_groups {
        drag.current_nodes = next_nodes;
        drag.current_groups = next_groups;
        drag.preview_rev = drag.preview_rev.wrapping_add(1);
    }
}
