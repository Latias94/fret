use crate::core::{CanvasPoint, CanvasRect, CanvasSize, GroupId, NodeId};
use crate::ui::canvas::NodeResizeHandle;
use crate::ui::canvas::state::{PendingGroupResize, PendingNodeResize};

pub(super) fn pending_group_resize() -> PendingGroupResize {
    PendingGroupResize {
        group: GroupId::from_u128(1),
        start_pos: Default::default(),
        start_rect: CanvasRect::default(),
    }
}

pub(super) fn pending_node_resize() -> PendingNodeResize {
    PendingNodeResize {
        node: NodeId::from_u128(1),
        handle: NodeResizeHandle::Right,
        start_pos: Default::default(),
        start_node_pos: CanvasPoint::default(),
        start_size: CanvasSize::default(),
        start_size_opt: Some(CanvasSize::default()),
    }
}
