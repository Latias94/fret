mod command;
mod target;

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
        let action = item.action;
        if let NodeGraphContextMenuAction::Command(command) = action {
            command::activate_command_context_action(self, cx, target, command);
            return;
        }

        target::activate_target_context_action(
            self,
            cx,
            target,
            invoked_at,
            action,
            menu_candidates,
        );
    }
}
