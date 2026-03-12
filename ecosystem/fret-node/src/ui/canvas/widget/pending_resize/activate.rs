use crate::ui::canvas::state::{InteractionState, PendingNodeResize};

pub(super) fn activate_pending_node_resize(
    interaction: &mut InteractionState,
    pending: PendingNodeResize,
) {
    super::super::pending_resize_session::activate_pending_node_resize(interaction, pending);
}
