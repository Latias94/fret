use crate::ui::canvas::state::{GroupResize, InteractionState, PendingGroupResize};

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
