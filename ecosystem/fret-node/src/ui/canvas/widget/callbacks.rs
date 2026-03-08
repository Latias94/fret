use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn emit_graph_callbacks(
        &mut self,
        committed: &GraphTransaction,
        changes: &NodeGraphChanges,
    ) {
        callbacks_graph::emit_graph_callbacks(self, committed, changes)
    }

    pub(super) fn emit_connect_start(&mut self, snapshot: &ViewSnapshot, kind: &WireDragKind) {
        callbacks_connect::emit_connect_start(self, snapshot, kind)
    }

    pub(super) fn emit_connect_end(
        &mut self,
        mode: crate::interaction::NodeGraphConnectionMode,
        kind: &WireDragKind,
        target: Option<PortId>,
        outcome: ConnectEndOutcome,
    ) {
        callbacks_connect::emit_connect_end(self, mode, kind, target, outcome)
    }

    pub(super) fn emit_move_start(&mut self, snapshot: &ViewSnapshot, kind: ViewportMoveKind) {
        callbacks_view::emit_move_start(self, snapshot, kind)
    }

    pub(super) fn emit_move_end(
        &mut self,
        snapshot: &ViewSnapshot,
        kind: ViewportMoveKind,
        outcome: ViewportMoveEndOutcome,
    ) {
        callbacks_view::emit_move_end(self, snapshot, kind, outcome)
    }

    pub(super) fn emit_node_drag_start(&mut self, primary: GraphNodeId, nodes: &[GraphNodeId]) {
        callbacks_view::emit_node_drag_start(self, primary, nodes)
    }

    pub(super) fn emit_node_drag_end(
        &mut self,
        primary: GraphNodeId,
        nodes: &[GraphNodeId],
        outcome: NodeDragEndOutcome,
    ) {
        callbacks_view::emit_node_drag_end(self, primary, nodes, outcome)
    }

    pub(super) fn emit_node_drag(&mut self, primary: GraphNodeId, nodes: &[GraphNodeId]) {
        callbacks_view::emit_node_drag(self, primary, nodes)
    }

    pub(super) fn emit_view_callbacks(&mut self, changes: &[ViewChange]) {
        callbacks_view::emit_view_callbacks(self, changes)
    }
}
