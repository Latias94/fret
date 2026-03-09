use crate::ui::canvas::state::{
    GroupResize, InteractionState, PendingGroupResize, PendingNodeResize,
};

pub(super) fn activate_pending_group_resize(
    interaction: &mut InteractionState,
    pending: PendingGroupResize,
) {
    interaction.pending_group_resize = None;
    interaction.group_resize = Some(GroupResize {
        group: pending.group,
        start_pos: pending.start_pos,
        start_rect: pending.start_rect,
        current_rect: pending.start_rect,
        preview_rev: 0,
    });
}

pub(super) fn activate_pending_node_resize(
    interaction: &mut InteractionState,
    pending: PendingNodeResize,
) {
    interaction.pending_node_resize = None;
    interaction.node_resize = Some(crate::ui::canvas::state::NodeResize {
        node: pending.node,
        handle: pending.handle,
        start_pos: pending.start_pos,
        start_node_pos: pending.start_node_pos,
        start_size: pending.start_size,
        start_size_opt: pending.start_size_opt,
        current_node_pos: pending.start_node_pos,
        current_size_opt: pending.start_size_opt,
        current_groups: Vec::new(),
        preview_rev: 0,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{CanvasPoint, CanvasRect, CanvasSize, GroupId, NodeId};
    use crate::ui::canvas::NodeResizeHandle;

    #[test]
    fn activate_pending_group_resize_moves_pending_into_active() {
        let pending = PendingGroupResize {
            group: GroupId::from_u128(1),
            start_pos: Default::default(),
            start_rect: CanvasRect::default(),
        };
        let mut interaction = InteractionState {
            pending_group_resize: Some(pending.clone()),
            ..Default::default()
        };

        activate_pending_group_resize(&mut interaction, pending.clone());

        assert!(interaction.pending_group_resize.is_none());
        let active = interaction.group_resize.expect("group resize active");
        assert_eq!(active.group, pending.group);
        assert_eq!(active.start_rect, pending.start_rect);
        assert_eq!(active.current_rect, pending.start_rect);
    }

    #[test]
    fn activate_pending_node_resize_moves_pending_into_active() {
        let pending = PendingNodeResize {
            node: NodeId::from_u128(1),
            handle: NodeResizeHandle::Right,
            start_pos: Default::default(),
            start_node_pos: CanvasPoint::default(),
            start_size: CanvasSize::default(),
            start_size_opt: Some(CanvasSize::default()),
        };
        let mut interaction = InteractionState {
            pending_node_resize: Some(pending.clone()),
            ..Default::default()
        };

        activate_pending_node_resize(&mut interaction, pending.clone());

        assert!(interaction.pending_node_resize.is_none());
        let active = interaction.node_resize.expect("node resize active");
        assert_eq!(active.node, pending.node);
        assert_eq!(active.start_size, pending.start_size);
        assert_eq!(active.current_node_pos, pending.start_node_pos);
        assert_eq!(active.current_size_opt, pending.start_size_opt);
    }
}
