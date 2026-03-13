use super::focus::{prepare_for_edge_anchor_hit, prepare_for_edge_hit, prepare_for_node_hit};

#[test]
fn prepare_for_node_hit_clears_competing_sessions_and_edge_focus() {
    let mut interaction = super::super::test_support::sample_interaction();

    prepare_for_node_hit(&mut interaction);

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
    assert!(interaction.hover_edge.is_some());
    assert!(interaction.focused_edge.is_none());
    assert!(interaction.hover_port.is_none());
    assert!(!interaction.click_connect);
}

#[test]
fn prepare_for_edge_anchor_hit_clears_hover_edge_but_preserves_focus_edge() {
    let mut interaction = super::super::test_support::sample_interaction();
    let focused = interaction.focused_edge;

    prepare_for_edge_anchor_hit(&mut interaction);

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
    assert!(interaction.hover_edge.is_none());
    assert_eq!(interaction.focused_edge, focused);
    assert!(interaction.hover_port.is_none());
    assert!(!interaction.click_connect);
}

#[test]
fn prepare_for_edge_hit_preserves_edge_drag_and_marquee_state() {
    let mut interaction = super::super::test_support::sample_interaction();
    let focused = interaction.focused_edge;

    prepare_for_edge_hit(&mut interaction);

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
    assert!(interaction.edge_drag.is_some());
    assert!(interaction.pending_edge_insert_drag.is_none());
    assert!(interaction.edge_insert_drag.is_none());
    assert!(interaction.pending_marquee.is_some());
    assert!(interaction.marquee.is_some());
    assert_eq!(interaction.focused_edge, focused);
    assert!(interaction.hover_port.is_none());
    assert!(!interaction.click_connect);
}
