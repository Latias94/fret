mod group;
mod node;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

use crate::ui::canvas::state::{InteractionState, PendingGroupResize, PendingNodeResize};

pub(super) fn activate_pending_group_resize(
    interaction: &mut InteractionState,
    pending: PendingGroupResize,
) {
    group::activate_pending_group_resize(interaction, pending);
}

pub(super) fn activate_pending_node_resize(
    interaction: &mut InteractionState,
    pending: PendingNodeResize,
) {
    node::activate_pending_node_resize(interaction, pending);
}
