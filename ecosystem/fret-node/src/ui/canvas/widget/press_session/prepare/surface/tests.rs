use super::*;

#[test]
fn prepare_for_pan_begin_preserves_edge_insert_sessions() {
    let mut interaction = super::super::test_support::sample_interaction();

    prepare_for_pan_begin(&mut interaction);

    assert!(interaction.pending_group_drag.is_none());
    assert!(interaction.group_drag.is_none());
    assert!(interaction.pending_group_resize.is_none());
    assert!(interaction.group_resize.is_none());
    assert!(interaction.pending_node_drag.is_none());
    assert!(interaction.node_drag.is_none());
    assert!(interaction.pending_node_resize.is_none());
    assert!(interaction.node_resize.is_none());
    assert!(interaction.pending_wire_drag.is_none());
    assert!(interaction.wire_drag.is_none());
    assert!(interaction.edge_drag.is_none());
    assert!(interaction.pending_marquee.is_none());
    assert!(interaction.marquee.is_none());
    assert!(interaction.pending_edge_insert_drag.is_some());
    assert!(interaction.edge_insert_drag.is_some());
    assert!(interaction.hover_edge.is_none());
    assert!(interaction.focused_edge.is_none());
    assert!(interaction.hover_port.is_none());
}

#[test]
fn prepare_for_background_interaction_clears_all_surface_pointer_sessions() {
    let mut interaction = super::super::test_support::sample_interaction();

    prepare_for_background_interaction(&mut interaction);

    assert!(interaction.pending_group_drag.is_none());
    assert!(interaction.group_drag.is_none());
    assert!(interaction.pending_group_resize.is_none());
    assert!(interaction.group_resize.is_none());
    assert!(interaction.pending_node_drag.is_none());
    assert!(interaction.node_drag.is_none());
    assert!(interaction.pending_node_resize.is_none());
    assert!(interaction.node_resize.is_none());
    assert!(interaction.pending_wire_drag.is_none());
    assert!(interaction.wire_drag.is_none());
    assert!(interaction.edge_drag.is_none());
    assert!(interaction.pending_edge_insert_drag.is_none());
    assert!(interaction.edge_insert_drag.is_none());
    assert!(interaction.pending_marquee.is_none());
    assert!(interaction.marquee.is_none());
    assert!(!interaction.click_connect);
}
