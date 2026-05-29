use crate::core::NodeId;
use crate::io::NodeGraphViewState;

use super::{
    DragState, MarqueeDragState, NodeDragState, PendingSelectionState, ReconnectDragState,
    effective_selected_nodes_for_paint,
};

pub(super) struct PaintOnlyInteractionFrameInputs<'a> {
    pub(super) view_state: &'a NodeGraphViewState,
    pub(super) drag: Option<DragState>,
    pub(super) marquee: Option<&'a MarqueeDragState>,
    pub(super) node_drag: Option<&'a NodeDragState>,
    pub(super) reconnect_drag: Option<&'a ReconnectDragState>,
    pub(super) pending_selection: Option<&'a PendingSelectionState>,
    pub(super) hovered_node: Option<NodeId>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PaintOnlyInteractionFramePlan {
    pub(super) panning: bool,
    pub(super) marquee_active: bool,
    pub(super) node_drag_armed: bool,
    pub(super) node_dragging: bool,
    pub(super) reconnect_drag_armed: bool,
    pub(super) reconnect_dragging: bool,
    pub(super) hovered: bool,
    pub(super) hovered_node: Option<NodeId>,
    pub(super) effective_selected_nodes: Vec<NodeId>,
}

impl PaintOnlyInteractionFramePlan {
    pub(super) fn selected_nodes_len(&self) -> usize {
        self.effective_selected_nodes.len()
    }
}

pub(super) fn plan_paint_only_interaction_frame(
    inputs: PaintOnlyInteractionFrameInputs<'_>,
) -> PaintOnlyInteractionFramePlan {
    let panning = inputs.drag.is_some();
    let marquee_active = inputs.marquee.is_some_and(|state| state.active);
    let node_drag_armed = inputs.node_drag.is_some_and(NodeDragState::is_armed);
    let node_dragging = inputs.node_drag.is_some_and(NodeDragState::is_active);
    let reconnect_drag_armed = inputs
        .reconnect_drag
        .is_some_and(ReconnectDragState::is_armed);
    let reconnect_dragging = inputs
        .reconnect_drag
        .is_some_and(ReconnectDragState::is_active);
    let effective_selected_nodes = effective_selected_nodes_for_paint(
        inputs.view_state,
        inputs.marquee,
        inputs.pending_selection,
    );

    PaintOnlyInteractionFramePlan {
        panning,
        marquee_active,
        node_drag_armed,
        node_dragging,
        reconnect_drag_armed,
        reconnect_dragging,
        hovered: inputs.hovered_node.is_some(),
        hovered_node: inputs.hovered_node,
        effective_selected_nodes,
    }
}
