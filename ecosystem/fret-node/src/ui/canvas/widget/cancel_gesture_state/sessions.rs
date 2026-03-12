use crate::ui::canvas::state::InteractionState;

pub(in super::super) fn clear_remaining_gesture_sessions(
    interaction: &mut InteractionState,
) -> bool {
    let mut canceled = false;

    canceled |= super::super::insert_node_drag::clear_insert_node_drag_state(interaction);
    if interaction.edge_insert_drag.take().is_some() {
        canceled = true;
    }
    if interaction.pending_edge_insert_drag.take().is_some() {
        canceled = true;
    }
    if interaction.edge_drag.take().is_some() {
        canceled = true;
    }
    if interaction.pending_node_drag.take().is_some() {
        canceled = true;
    }
    if interaction.group_drag.take().is_some() {
        canceled = true;
    }
    if interaction.pending_group_drag.take().is_some() {
        canceled = true;
    }
    if interaction.group_resize.take().is_some() {
        canceled = true;
    }
    if interaction.pending_group_resize.take().is_some() {
        canceled = true;
    }
    if interaction.node_resize.take().is_some() {
        canceled = true;
    }
    if interaction.pending_node_resize.take().is_some() {
        canceled = true;
    }
    if interaction.pending_wire_drag.take().is_some() {
        canceled = true;
    }
    if interaction.marquee.take().is_some() {
        canceled = true;
    }
    if interaction.pending_marquee.take().is_some() {
        canceled = true;
    }

    canceled
}

#[cfg(test)]
mod tests;
