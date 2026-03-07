use crate::ui::canvas::widget::*;
impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in crate::ui::canvas::widget) fn activate_context_menu_item<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        target: &ContextMenuTarget,
        invoked_at: Point,
        item: NodeGraphContextMenuItem,
        menu_candidates: &[InsertNodeCandidate],
    ) {
        match (target, item.action) {
            (_, NodeGraphContextMenuAction::Command(command)) => {
                self.interaction.context_menu = None;
                if let ContextMenuTarget::Group(group_id) = target {
                    self.select_group_context_target(cx.app, *group_id);
                }
                cx.dispatch_command(command);
            }
            (ContextMenuTarget::BackgroundInsertNodePicker { at }, action) => {
                if self.activate_background_context_action(cx, *at, action, menu_candidates) {
                    return;
                }
            }
            (ContextMenuTarget::ConnectionInsertNodePicker { from, at }, action) => {
                if self.activate_connection_insert_picker_action(
                    cx,
                    *from,
                    *at,
                    invoked_at,
                    action,
                    menu_candidates,
                ) {
                    return;
                }
            }
            (ContextMenuTarget::Edge(edge_id), action) => {
                if self.activate_edge_context_action(cx, *edge_id, invoked_at, action) {
                    return;
                }
            }
            (
                ContextMenuTarget::EdgeInsertNodePicker(edge_id),
                NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix),
            ) => {
                let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                    return;
                };
                edge_insert::insert_node_on_edge(self, cx, *edge_id, invoked_at, candidate);
            }
            (ContextMenuTarget::ConnectionConvertPicker { from, to, at }, action) => {
                if self.activate_connection_conversion_picker_action(
                    cx,
                    *from,
                    *to,
                    *at,
                    invoked_at,
                    action,
                    menu_candidates,
                ) {
                    return;
                }
            }
            _ => {}
        }
    }
}
