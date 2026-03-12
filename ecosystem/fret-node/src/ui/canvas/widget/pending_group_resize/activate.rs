use crate::ui::canvas::state::{InteractionState, PendingGroupResize};

pub(super) fn activate_pending_group_resize(
    interaction: &mut InteractionState,
    pending: PendingGroupResize,
) {
    super::super::pending_resize_session::activate_pending_group_resize(interaction, pending);
}
