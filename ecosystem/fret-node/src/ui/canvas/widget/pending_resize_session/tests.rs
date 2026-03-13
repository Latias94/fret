use super::*;
use crate::ui::canvas::state::InteractionState;

#[test]
fn activate_pending_group_resize_moves_pending_into_active() {
    let pending = super::test_support::pending_group_resize();
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
    let pending = super::test_support::pending_node_resize();
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
