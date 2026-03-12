use fret_ui::UiHost;

use crate::ui::canvas::state::InteractionState;

pub(in super::super) fn clear_node_drag_release_state(interaction: &mut InteractionState) {
    interaction.pending_node_resize = None;
    interaction.snap_guides = None;
}

pub(in super::super) fn finish_pointer_up_with_snap_guide_cleanup<H: UiHost>(
    interaction: &mut InteractionState,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) {
    interaction.snap_guides = None;
    super::super::pointer_up_finish::finish_pointer_up(cx);
}

#[cfg(test)]
mod tests;
