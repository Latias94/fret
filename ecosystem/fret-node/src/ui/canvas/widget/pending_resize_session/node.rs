use crate::ui::canvas::state::{InteractionState, PendingNodeResize};

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
