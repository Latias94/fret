use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_command<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
        command: &CommandId,
    ) -> bool {
        match command.as_str() {
            CMD_NODE_GRAPH_OPEN_INSERT_NODE => self.cmd_open_insert_node(cx, snapshot),
            CMD_NODE_GRAPH_CREATE_GROUP => self.cmd_create_group(cx),
            CMD_NODE_GRAPH_GROUP_BRING_TO_FRONT => self.cmd_group_bring_to_front(cx, snapshot),
            CMD_NODE_GRAPH_GROUP_SEND_TO_BACK => self.cmd_group_send_to_back(cx, snapshot),
            CMD_NODE_GRAPH_GROUP_RENAME => self.cmd_group_rename(cx, snapshot),
            CMD_NODE_GRAPH_OPEN_SPLIT_EDGE_INSERT_NODE => {
                self.cmd_open_split_edge_insert_node(cx, snapshot)
            }
            CMD_NODE_GRAPH_INSERT_REROUTE => self.cmd_insert_reroute(cx, snapshot),
            CMD_NODE_GRAPH_OPEN_CONVERSION_PICKER => self.cmd_open_conversion_picker(cx, snapshot),

            CMD_NODE_GRAPH_FRAME_SELECTION => self.cmd_frame_selection(cx, snapshot),
            CMD_NODE_GRAPH_FRAME_ALL => self.cmd_frame_all(cx, snapshot),
            CMD_NODE_GRAPH_RESET_VIEW => self.cmd_reset_view(cx),
            CMD_NODE_GRAPH_ZOOM_IN => self.cmd_zoom_in(cx, snapshot),
            CMD_NODE_GRAPH_ZOOM_OUT => self.cmd_zoom_out(cx, snapshot),

            CMD_NODE_GRAPH_TOGGLE_CONNECTION_MODE => self.cmd_toggle_connection_mode(cx, snapshot),
            CMD_NODE_GRAPH_UNDO => self.cmd_undo(cx, snapshot),
            CMD_NODE_GRAPH_REDO => self.cmd_redo(cx, snapshot),

            CMD_NODE_GRAPH_FOCUS_NEXT => self.cmd_focus_next_node(cx, snapshot),
            CMD_NODE_GRAPH_FOCUS_PREV => self.cmd_focus_prev_node(cx, snapshot),
            CMD_NODE_GRAPH_FOCUS_NEXT_EDGE => self.cmd_focus_next_edge(cx, snapshot),
            CMD_NODE_GRAPH_FOCUS_PREV_EDGE => self.cmd_focus_prev_edge(cx, snapshot),
            CMD_NODE_GRAPH_FOCUS_NEXT_PORT => self.cmd_focus_next_port(cx, snapshot),
            CMD_NODE_GRAPH_FOCUS_PREV_PORT => self.cmd_focus_prev_port(cx, snapshot),
            CMD_NODE_GRAPH_FOCUS_PORT_LEFT => self.cmd_focus_port_left(cx, snapshot),
            CMD_NODE_GRAPH_FOCUS_PORT_RIGHT => self.cmd_focus_port_right(cx, snapshot),
            CMD_NODE_GRAPH_FOCUS_PORT_UP => self.cmd_focus_port_up(cx, snapshot),
            CMD_NODE_GRAPH_FOCUS_PORT_DOWN => self.cmd_focus_port_down(cx, snapshot),
            CMD_NODE_GRAPH_ACTIVATE => self.cmd_activate(cx, snapshot),

            CMD_NODE_GRAPH_SELECT_ALL => self.cmd_select_all(cx, snapshot),
            CMD_NODE_GRAPH_COPY => self.cmd_copy(cx, snapshot),
            CMD_NODE_GRAPH_CUT => self.cmd_cut(cx, snapshot),
            CMD_NODE_GRAPH_PASTE => self.cmd_paste(cx, snapshot),
            CMD_NODE_GRAPH_DUPLICATE => self.cmd_duplicate(cx, snapshot),
            CMD_NODE_GRAPH_DELETE_SELECTION => self.cmd_delete_selection(cx, snapshot),

            "edit.select_all" => self.cmd_select_all(cx, snapshot),
            "edit.copy" => self.cmd_copy(cx, snapshot),
            "edit.cut" => self.cmd_cut(cx, snapshot),
            "edit.paste" => self.cmd_paste(cx, snapshot),

            CMD_NODE_GRAPH_NUDGE_LEFT => {
                self.cmd_nudge_selection(cx, snapshot, CanvasPoint { x: -1.0, y: 0.0 }, false)
            }
            CMD_NODE_GRAPH_NUDGE_RIGHT => {
                self.cmd_nudge_selection(cx, snapshot, CanvasPoint { x: 1.0, y: 0.0 }, false)
            }
            CMD_NODE_GRAPH_NUDGE_UP => {
                self.cmd_nudge_selection(cx, snapshot, CanvasPoint { x: 0.0, y: -1.0 }, false)
            }
            CMD_NODE_GRAPH_NUDGE_DOWN => {
                self.cmd_nudge_selection(cx, snapshot, CanvasPoint { x: 0.0, y: 1.0 }, false)
            }
            CMD_NODE_GRAPH_NUDGE_LEFT_FAST => {
                self.cmd_nudge_selection(cx, snapshot, CanvasPoint { x: -1.0, y: 0.0 }, true)
            }
            CMD_NODE_GRAPH_NUDGE_RIGHT_FAST => {
                self.cmd_nudge_selection(cx, snapshot, CanvasPoint { x: 1.0, y: 0.0 }, true)
            }
            CMD_NODE_GRAPH_NUDGE_UP_FAST => {
                self.cmd_nudge_selection(cx, snapshot, CanvasPoint { x: 0.0, y: -1.0 }, true)
            }
            CMD_NODE_GRAPH_NUDGE_DOWN_FAST => {
                self.cmd_nudge_selection(cx, snapshot, CanvasPoint { x: 0.0, y: 1.0 }, true)
            }

            CMD_NODE_GRAPH_ALIGN_LEFT => {
                self.cmd_align_or_distribute_selection(cx, snapshot, AlignDistributeMode::AlignLeft)
            }
            CMD_NODE_GRAPH_ALIGN_RIGHT => self.cmd_align_or_distribute_selection(
                cx,
                snapshot,
                AlignDistributeMode::AlignRight,
            ),
            CMD_NODE_GRAPH_ALIGN_TOP => {
                self.cmd_align_or_distribute_selection(cx, snapshot, AlignDistributeMode::AlignTop)
            }
            CMD_NODE_GRAPH_ALIGN_BOTTOM => self.cmd_align_or_distribute_selection(
                cx,
                snapshot,
                AlignDistributeMode::AlignBottom,
            ),
            CMD_NODE_GRAPH_ALIGN_CENTER_X => self.cmd_align_or_distribute_selection(
                cx,
                snapshot,
                AlignDistributeMode::AlignCenterX,
            ),
            CMD_NODE_GRAPH_ALIGN_CENTER_Y => self.cmd_align_or_distribute_selection(
                cx,
                snapshot,
                AlignDistributeMode::AlignCenterY,
            ),
            CMD_NODE_GRAPH_DISTRIBUTE_X => self.cmd_align_or_distribute_selection(
                cx,
                snapshot,
                AlignDistributeMode::DistributeX,
            ),
            CMD_NODE_GRAPH_DISTRIBUTE_Y => self.cmd_align_or_distribute_selection(
                cx,
                snapshot,
                AlignDistributeMode::DistributeY,
            ),

            _ => false,
        }
    }
}
