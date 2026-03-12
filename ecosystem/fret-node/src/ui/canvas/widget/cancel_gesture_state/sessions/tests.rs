use super::*;
use crate::core::{EdgeId, GroupId, NodeId, PortId};
use crate::ui::canvas::state::{
    EdgeDrag, EdgeInsertDrag, GroupDrag, GroupResize, MarqueeDrag, NodeDrag, NodeResize,
    PendingEdgeInsertDrag, PendingGroupDrag, PendingGroupResize, PendingMarqueeDrag,
    PendingNodeDrag, PendingNodeResize, PendingWireDrag, WireDragKind,
};
use fret_core::{Point, Px};

fn point(x: f32, y: f32) -> Point {
    Point::new(Px(x), Px(y))
}

#[test]
fn clear_remaining_gesture_sessions_clears_all_pending_and_active_sessions() {
    let node = NodeId::from_u128(1);
    let group = GroupId::from_u128(2);
    let port = PortId::from_u128(3);
    let mut interaction = InteractionState {
        edge_insert_drag: Some(EdgeInsertDrag {
            edge: EdgeId::from_u128(4),
            pos: point(1.0, 2.0),
        }),
        pending_edge_insert_drag: Some(PendingEdgeInsertDrag {
            edge: EdgeId::from_u128(5),
            start_pos: point(3.0, 4.0),
        }),
        edge_drag: Some(EdgeDrag {
            edge: EdgeId::from_u128(6),
            start_pos: point(5.0, 6.0),
        }),
        pending_node_drag: Some(PendingNodeDrag {
            primary: node,
            nodes: vec![node],
            grab_offset: point(0.0, 0.0),
            start_pos: point(7.0, 8.0),
            select_action: Default::default(),
            drag_enabled: true,
        }),
        group_drag: Some(GroupDrag {
            group,
            start_pos: point(0.0, 0.0),
            start_rect: Default::default(),
            nodes: Vec::new(),
            current_rect: Default::default(),
            current_nodes: Vec::new(),
            preview_rev: 0,
        }),
        pending_group_drag: Some(PendingGroupDrag {
            group,
            start_pos: point(0.0, 0.0),
            start_rect: Default::default(),
        }),
        group_resize: Some(GroupResize {
            group,
            start_pos: point(0.0, 0.0),
            start_rect: Default::default(),
            current_rect: Default::default(),
            preview_rev: 0,
        }),
        pending_group_resize: Some(PendingGroupResize {
            group,
            start_pos: point(0.0, 0.0),
            start_rect: Default::default(),
        }),
        node_resize: Some(NodeResize {
            node,
            handle: crate::ui::canvas::NodeResizeHandle::Right,
            start_pos: point(0.0, 0.0),
            start_node_pos: Default::default(),
            start_size: Default::default(),
            start_size_opt: None,
            current_node_pos: Default::default(),
            current_size_opt: None,
            current_groups: Vec::new(),
            preview_rev: 0,
        }),
        pending_node_resize: Some(PendingNodeResize {
            node,
            handle: crate::ui::canvas::NodeResizeHandle::Right,
            start_pos: point(0.0, 0.0),
            start_node_pos: Default::default(),
            start_size: Default::default(),
            start_size_opt: None,
        }),
        pending_wire_drag: Some(PendingWireDrag {
            kind: WireDragKind::New {
                from: port,
                bundle: vec![port],
            },
            start_pos: point(0.0, 0.0),
        }),
        marquee: Some(MarqueeDrag {
            start_pos: point(0.0, 0.0),
            pos: point(1.0, 1.0),
        }),
        pending_marquee: Some(PendingMarqueeDrag {
            start_pos: point(0.0, 0.0),
            clear_selection_on_up: true,
        }),
        node_drag: Some(NodeDrag {
            primary: node,
            node_ids: vec![node],
            nodes: Vec::new(),
            current_nodes: Vec::new(),
            current_groups: Vec::new(),
            preview_rev: 0,
            grab_offset: point(0.0, 0.0),
            start_pos: point(0.0, 0.0),
        }),
        ..Default::default()
    };

    assert!(clear_remaining_gesture_sessions(&mut interaction));
    assert!(interaction.edge_insert_drag.is_none());
    assert!(interaction.pending_edge_insert_drag.is_none());
    assert!(interaction.edge_drag.is_none());
    assert!(interaction.pending_node_drag.is_none());
    assert!(interaction.group_drag.is_none());
    assert!(interaction.pending_group_drag.is_none());
    assert!(interaction.group_resize.is_none());
    assert!(interaction.pending_group_resize.is_none());
    assert!(interaction.node_resize.is_none());
    assert!(interaction.pending_node_resize.is_none());
    assert!(interaction.pending_wire_drag.is_none());
    assert!(interaction.marquee.is_none());
    assert!(interaction.pending_marquee.is_none());
}
