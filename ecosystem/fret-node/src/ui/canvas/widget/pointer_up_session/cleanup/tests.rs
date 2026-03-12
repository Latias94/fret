use super::*;
use crate::ui::canvas::snaplines::SnapGuides;

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
