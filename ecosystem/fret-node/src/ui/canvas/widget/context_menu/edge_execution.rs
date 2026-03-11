mod custom_action;
mod delete;
mod open_insert;
mod reroute;

use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn activate_edge_context_action<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        edge_id: EdgeId,
        invoked_at: Point,
        action: NodeGraphContextMenuAction,
    ) -> bool {
        match action {
            NodeGraphContextMenuAction::OpenInsertNodePicker => {
                open_insert::open_edge_insert_context_menu(self, cx, edge_id, invoked_at);
                true
            }
            NodeGraphContextMenuAction::InsertReroute => {
                reroute::insert_edge_reroute(self, cx, edge_id, invoked_at);
                true
            }
            NodeGraphContextMenuAction::DeleteEdge => {
                delete::delete_edge(self, cx, edge_id);
                true
            }
            NodeGraphContextMenuAction::Custom(action_id) => {
                custom_action::apply_custom_edge_context_action(self, cx, edge_id, action_id);
                true
            }
            _ => false,
        }
    }
}
