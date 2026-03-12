use crate::core::{CanvasPoint, CanvasRect, EdgeId, GroupId, NodeId as GraphNodeId, PortId};
use crate::rules::{DiagnosticSeverity, EdgeEndpoint};
use crate::ui::canvas::state::{
    EdgeDrag, EdgeInsertDrag, GroupDrag, GroupResize, InteractionState, MarqueeDrag, NodeDrag,
    NodeResize, PendingEdgeInsertDrag, PendingGroupDrag, PendingGroupResize, PendingMarqueeDrag,
    PendingNodeDrag, PendingNodeResize, PendingNodeSelectAction, PendingWireDrag, WireDrag,
    WireDragKind,
};
use fret_core::Point;

pub(super) fn sample_interaction() -> InteractionState {
    InteractionState {
        pending_group_drag: Some(PendingGroupDrag {
            group: GroupId::from_u128(1),
            start_pos: Point::default(),
            start_rect: CanvasRect::default(),
        }),
        group_drag: Some(GroupDrag {
            group: GroupId::from_u128(2),
            start_pos: Point::default(),
            start_rect: CanvasRect::default(),
            nodes: Vec::new(),
            current_rect: CanvasRect::default(),
            current_nodes: Vec::new(),
            preview_rev: 1,
        }),
        pending_group_resize: Some(PendingGroupResize {
            group: GroupId::from_u128(3),
            start_pos: Point::default(),
            start_rect: CanvasRect::default(),
        }),
        group_resize: Some(GroupResize {
            group: GroupId::from_u128(4),
            start_pos: Point::default(),
            start_rect: CanvasRect::default(),
            current_rect: CanvasRect::default(),
            preview_rev: 2,
        }),
        pending_node_drag: Some(PendingNodeDrag {
            primary: GraphNodeId::from_u128(5),
            nodes: vec![GraphNodeId::from_u128(5)],
            grab_offset: Point::default(),
            start_pos: Point::default(),
            select_action: PendingNodeSelectAction::Toggle,
            drag_enabled: true,
        }),
        node_drag: Some(NodeDrag {
            primary: GraphNodeId::from_u128(6),
            node_ids: vec![GraphNodeId::from_u128(6)],
            nodes: vec![(GraphNodeId::from_u128(6), CanvasPoint::default())],
            current_nodes: vec![(GraphNodeId::from_u128(6), CanvasPoint::default())],
            current_groups: vec![(GroupId::from_u128(7), CanvasRect::default())],
            preview_rev: 3,
            grab_offset: Point::default(),
            start_pos: Point::default(),
        }),
        pending_node_resize: Some(PendingNodeResize {
            node: GraphNodeId::from_u128(8),
            handle: crate::ui::canvas::NodeResizeHandle::Right,
            start_pos: Point::default(),
            start_node_pos: CanvasPoint::default(),
            start_size: Default::default(),
            start_size_opt: None,
        }),
        node_resize: Some(NodeResize {
            node: GraphNodeId::from_u128(9),
            handle: crate::ui::canvas::NodeResizeHandle::Right,
            start_pos: Point::default(),
            start_node_pos: CanvasPoint::default(),
            start_size: Default::default(),
            start_size_opt: None,
            current_node_pos: CanvasPoint::default(),
            current_size_opt: None,
            current_groups: vec![(GroupId::from_u128(10), CanvasRect::default())],
            preview_rev: 4,
        }),
        pending_wire_drag: Some(PendingWireDrag {
            kind: WireDragKind::New {
                from: PortId::from_u128(11),
                bundle: vec![PortId::from_u128(11)],
            },
            start_pos: Point::default(),
        }),
        wire_drag: Some(WireDrag {
            kind: WireDragKind::Reconnect {
                edge: EdgeId::from_u128(12),
                endpoint: EdgeEndpoint::To,
                fixed: PortId::from_u128(13),
            },
            pos: Point::default(),
        }),
        pending_edge_insert_drag: Some(PendingEdgeInsertDrag {
            edge: EdgeId::from_u128(14),
            start_pos: Point::default(),
        }),
        edge_insert_drag: Some(EdgeInsertDrag {
            edge: EdgeId::from_u128(15),
            pos: Point::default(),
        }),
        edge_drag: Some(EdgeDrag {
            edge: EdgeId::from_u128(16),
            start_pos: Point::default(),
        }),
        pending_marquee: Some(PendingMarqueeDrag {
            start_pos: Point::default(),
            clear_selection_on_up: true,
        }),
        marquee: Some(MarqueeDrag {
            start_pos: Point::default(),
            pos: Point::default(),
        }),
        hover_edge: Some(EdgeId::from_u128(17)),
        focused_edge: Some(EdgeId::from_u128(18)),
        hover_port: Some(PortId::from_u128(19)),
        hover_port_valid: true,
        hover_port_convertible: true,
        hover_port_diagnostic: Some((DiagnosticSeverity::Error, "diag".into())),
        click_connect: true,
        ..Default::default()
    }
}
