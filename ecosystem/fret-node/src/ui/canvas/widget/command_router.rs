mod dispatch;

use super::*;
use dispatch::{DirectCommandRoute, direct_command_route};

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_command<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
        command: &CommandId,
    ) -> bool {
        if let Some(request) = super::command_router_nudge::nudge_command_request(command.as_str())
        {
            return self.cmd_nudge_selection(cx, snapshot, request.dir, request.fast);
        }

        if let Some(mode) = super::command_router_align::align_or_distribute_mode(command.as_str())
        {
            return self.cmd_align_or_distribute_selection(cx, snapshot, mode);
        }

        match direct_command_route(command.as_str()) {
            Some(DirectCommandRoute::OpenInsertNode) => self.cmd_open_insert_node(cx, snapshot),
            Some(DirectCommandRoute::CreateGroup) => self.cmd_create_group(cx),
            Some(DirectCommandRoute::GroupBringToFront) => {
                self.cmd_group_bring_to_front(cx, snapshot)
            }
            Some(DirectCommandRoute::GroupSendToBack) => self.cmd_group_send_to_back(cx, snapshot),
            Some(DirectCommandRoute::GroupRename) => self.cmd_group_rename(cx, snapshot),
            Some(DirectCommandRoute::OpenSplitEdgeInsertNode) => {
                self.cmd_open_split_edge_insert_node(cx, snapshot)
            }
            Some(DirectCommandRoute::InsertReroute) => self.cmd_insert_reroute(cx, snapshot),
            Some(DirectCommandRoute::OpenConversionPicker) => {
                self.cmd_open_conversion_picker(cx, snapshot)
            }
            Some(DirectCommandRoute::FrameSelection) => self.cmd_frame_selection(cx, snapshot),
            Some(DirectCommandRoute::FrameAll) => self.cmd_frame_all(cx, snapshot),
            Some(DirectCommandRoute::ResetView) => self.cmd_reset_view(cx),
            Some(DirectCommandRoute::ZoomIn) => self.cmd_zoom_in(cx, snapshot),
            Some(DirectCommandRoute::ZoomOut) => self.cmd_zoom_out(cx, snapshot),
            Some(DirectCommandRoute::ToggleConnectionMode) => {
                self.cmd_toggle_connection_mode(cx, snapshot)
            }
            Some(DirectCommandRoute::Undo) => self.cmd_undo(cx, snapshot),
            Some(DirectCommandRoute::Redo) => self.cmd_redo(cx, snapshot),
            Some(DirectCommandRoute::FocusNextNode) => self.cmd_focus_next_node(cx, snapshot),
            Some(DirectCommandRoute::FocusPrevNode) => self.cmd_focus_prev_node(cx, snapshot),
            Some(DirectCommandRoute::FocusNextEdge) => self.cmd_focus_next_edge(cx, snapshot),
            Some(DirectCommandRoute::FocusPrevEdge) => self.cmd_focus_prev_edge(cx, snapshot),
            Some(DirectCommandRoute::FocusNextPort) => self.cmd_focus_next_port(cx, snapshot),
            Some(DirectCommandRoute::FocusPrevPort) => self.cmd_focus_prev_port(cx, snapshot),
            Some(DirectCommandRoute::FocusPortLeft) => self.cmd_focus_port_left(cx, snapshot),
            Some(DirectCommandRoute::FocusPortRight) => self.cmd_focus_port_right(cx, snapshot),
            Some(DirectCommandRoute::FocusPortUp) => self.cmd_focus_port_up(cx, snapshot),
            Some(DirectCommandRoute::FocusPortDown) => self.cmd_focus_port_down(cx, snapshot),
            Some(DirectCommandRoute::Activate) => self.cmd_activate(cx, snapshot),
            Some(DirectCommandRoute::SelectAll) => self.cmd_select_all(cx, snapshot),
            Some(DirectCommandRoute::Copy) => self.cmd_copy(cx, snapshot),
            Some(DirectCommandRoute::Cut) => self.cmd_cut(cx, snapshot),
            Some(DirectCommandRoute::Paste) => self.cmd_paste(cx, snapshot),
            Some(DirectCommandRoute::Duplicate) => self.cmd_duplicate(cx, snapshot),
            Some(DirectCommandRoute::DeleteSelection) => self.cmd_delete_selection(cx, snapshot),
            None => false,
        }
    }
}
