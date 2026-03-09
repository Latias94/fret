use fret_ui::UiHost;

use crate::ui::canvas::state::InteractionState;

pub(super) fn finish_pending_release<H: UiHost, T>(
    slot: &mut Option<T>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    if slot.take().is_none() {
        return false;
    }

    super::pointer_up_finish::finish_pointer_up(cx);
    true
}

pub(super) fn clear_node_drag_release_state(interaction: &mut InteractionState) {
    interaction.pending_node_resize = None;
    interaction.snap_guides = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::canvas::snaplines::SnapGuides;
    use crate::ui::canvas::state::InteractionState;

    #[test]
    fn clear_node_drag_release_state_clears_pending_resize_and_snap_guides() {
        let mut interaction = InteractionState {
            pending_node_resize: Some(crate::ui::canvas::state::PendingNodeResize {
                node: crate::core::NodeId::from_u128(1),
                handle: crate::ui::canvas::NodeResizeHandle::Right,
                start_pos: Default::default(),
                start_node_pos: Default::default(),
                start_size: Default::default(),
                start_size_opt: None,
            }),
            snap_guides: Some(SnapGuides::default()),
            ..Default::default()
        };

        clear_node_drag_release_state(&mut interaction);

        assert!(interaction.pending_node_resize.is_none());
        assert!(interaction.snap_guides.is_none());
    }
}
