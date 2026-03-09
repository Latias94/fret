use crate::ui::canvas::state::InteractionState;

pub(super) fn prepare_for_port_hit(interaction: &mut InteractionState) {
    clear_group_drag(interaction);
    clear_group_resize(interaction);
    clear_node_drag(interaction);
    clear_wire_drag(interaction);
    clear_edge_drag(interaction);
    clear_edge_insert_drag(interaction);
    clear_marquee(interaction);
    super::focus_session::clear_edge_focus_and_hover_port_hints(interaction);
}

pub(super) fn prepare_for_edge_anchor_hit(interaction: &mut InteractionState) {
    clear_group_drag(interaction);
    clear_group_resize(interaction);
    clear_node_drag(interaction);
    clear_node_resize(interaction);
    clear_wire_drag(interaction);
    clear_edge_drag(interaction);
    clear_edge_insert_drag(interaction);
    clear_marquee(interaction);
    super::focus_session::clear_hover_port_hints(interaction);
    interaction.hover_edge = None;
}

pub(super) fn prepare_for_resize_hit(interaction: &mut InteractionState) {
    clear_group_drag(interaction);
    clear_group_resize(interaction);
    clear_node_drag(interaction);
    clear_node_resize(interaction);
    clear_wire_drag(interaction);
    clear_edge_drag(interaction);
    clear_edge_insert_drag(interaction);
    clear_marquee(interaction);
    super::focus_session::clear_hover_port_hints(interaction);
}

pub(super) fn prepare_for_node_hit(interaction: &mut InteractionState) {
    clear_group_drag(interaction);
    clear_group_resize(interaction);
    clear_node_drag(interaction);
    clear_node_resize(interaction);
    clear_wire_drag(interaction);
    clear_edge_drag(interaction);
    clear_edge_insert_drag(interaction);
    clear_marquee(interaction);
    super::focus_session::clear_edge_focus_and_hover_port_hints(interaction);
}

pub(super) fn prepare_for_edge_hit(interaction: &mut InteractionState) {
    clear_group_drag(interaction);
    clear_group_resize(interaction);
    clear_node_drag(interaction);
    clear_node_resize(interaction);
    clear_wire_drag(interaction);
    clear_edge_insert_drag(interaction);
    super::focus_session::clear_hover_port_hints(interaction);
}

pub(super) fn prepare_for_group_resize(interaction: &mut InteractionState) {
    clear_group_drag(interaction);
    clear_node_drag(interaction);
    clear_node_resize(interaction);
    clear_wire_drag(interaction);
    clear_edge_drag(interaction);
    clear_edge_insert_drag(interaction);
    clear_marquee(interaction);
    super::focus_session::clear_edge_focus_and_hover_port_hints(interaction);
}

pub(super) fn prepare_for_group_drag(interaction: &mut InteractionState) {
    clear_node_drag(interaction);
    clear_node_resize(interaction);
    clear_wire_drag(interaction);
    clear_edge_drag(interaction);
    clear_edge_insert_drag(interaction);
    clear_marquee(interaction);
    super::focus_session::clear_edge_focus_and_hover_port_hints(interaction);
}

pub(super) fn prepare_for_background_interaction(interaction: &mut InteractionState) {
    clear_edge_drag(interaction);
    clear_edge_insert_drag(interaction);
    clear_group_drag(interaction);
    clear_group_resize(interaction);
    clear_node_drag(interaction);
    clear_node_resize(interaction);
    clear_wire_drag(interaction);
    clear_marquee(interaction);
    super::focus_session::clear_edge_focus_and_hover_port_hints(interaction);
}

pub(super) fn prepare_for_selection_marquee(interaction: &mut InteractionState) {
    clear_edge_drag(interaction);
    clear_edge_insert_drag(interaction);
    clear_group_drag(interaction);
    clear_group_resize(interaction);
    clear_node_drag(interaction);
    clear_node_resize(interaction);
    clear_wire_drag(interaction);
    clear_marquee(interaction);
    super::focus_session::clear_edge_focus_and_hover_port_hints(interaction);
}

pub(super) fn prepare_for_pan_begin(interaction: &mut InteractionState) {
    clear_group_drag(interaction);
    clear_group_resize(interaction);
    clear_node_drag(interaction);
    clear_node_resize(interaction);
    clear_wire_drag(interaction);
    clear_edge_drag(interaction);
    clear_marquee(interaction);
    super::focus_session::clear_hover_edge_focus_and_hover_port_hints(interaction);
}

fn clear_group_drag(interaction: &mut InteractionState) {
    interaction.pending_group_drag = None;
    interaction.group_drag = None;
}

fn clear_group_resize(interaction: &mut InteractionState) {
    interaction.pending_group_resize = None;
    interaction.group_resize = None;
}

fn clear_node_drag(interaction: &mut InteractionState) {
    interaction.pending_node_drag = None;
    interaction.node_drag = None;
}

fn clear_node_resize(interaction: &mut InteractionState) {
    interaction.pending_node_resize = None;
    interaction.node_resize = None;
}

fn clear_wire_drag(interaction: &mut InteractionState) {
    interaction.pending_wire_drag = None;
    interaction.wire_drag = None;
    interaction.click_connect = false;
}

fn clear_edge_drag(interaction: &mut InteractionState) {
    interaction.edge_drag = None;
}

fn clear_edge_insert_drag(interaction: &mut InteractionState) {
    interaction.pending_edge_insert_drag = None;
    interaction.edge_insert_drag = None;
}

fn clear_marquee(interaction: &mut InteractionState) {
    interaction.pending_marquee = None;
    interaction.marquee = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::Point;

    use crate::core::{CanvasPoint, CanvasRect, EdgeId, GroupId, NodeId as GraphNodeId, PortId};
    use crate::rules::{DiagnosticSeverity, EdgeEndpoint};
    use crate::ui::canvas::state::{
        EdgeDrag, EdgeInsertDrag, GroupDrag, GroupResize, MarqueeDrag, NodeDrag, NodeResize,
        PendingEdgeInsertDrag, PendingGroupDrag, PendingGroupResize, PendingMarqueeDrag,
        PendingNodeDrag, PendingNodeResize, PendingNodeSelectAction, PendingWireDrag, WireDrag,
        WireDragKind,
    };

    fn sample_interaction() -> InteractionState {
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

    #[test]
    fn prepare_for_port_hit_preserves_node_resize_but_clears_competing_pointer_sessions() {
        let mut interaction = sample_interaction();

        prepare_for_port_hit(&mut interaction);

        assert!(interaction.pending_group_drag.is_none());
        assert!(interaction.group_drag.is_none());
        assert!(interaction.pending_group_resize.is_none());
        assert!(interaction.group_resize.is_none());
        assert!(interaction.pending_node_drag.is_none());
        assert!(interaction.node_drag.is_none());
        assert!(interaction.pending_wire_drag.is_none());
        assert!(interaction.wire_drag.is_none());
        assert!(interaction.edge_drag.is_none());
        assert!(interaction.pending_edge_insert_drag.is_none());
        assert!(interaction.edge_insert_drag.is_none());
        assert!(interaction.pending_marquee.is_none());
        assert!(interaction.marquee.is_none());
        assert!(!interaction.click_connect);
        assert!(interaction.pending_node_resize.is_some());
        assert!(interaction.node_resize.is_some());
    }

    #[test]
    fn prepare_for_pan_begin_preserves_edge_insert_sessions() {
        let mut interaction = sample_interaction();

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
}
