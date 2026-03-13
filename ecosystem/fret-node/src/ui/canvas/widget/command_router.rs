mod dispatch;
mod edit;
mod focus;
mod group;
mod insert;
mod view;

use super::*;
use dispatch::direct_command_route;

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
            Some(route) => {
                insert::handle_direct_insert_command(self, cx, snapshot, route)
                    || group::handle_direct_group_command(self, cx, snapshot, route)
                    || view::handle_direct_view_command(self, cx, snapshot, route)
                    || focus::handle_direct_focus_command(self, cx, snapshot, route)
                    || edit::handle_direct_edit_command(self, cx, snapshot, route)
            }
            None => false,
        }
    }
}
