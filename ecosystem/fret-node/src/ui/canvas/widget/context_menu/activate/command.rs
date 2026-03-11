use crate::ui::canvas::widget::*;

pub(super) fn activate_command_context_action<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    target: &ContextMenuTarget,
    command: fret_runtime::CommandId,
) {
    super::super::clear_context_menu(&mut canvas.interaction);
    if let ContextMenuTarget::Group(group_id) = target {
        canvas.select_group_context_target(cx.app, *group_id);
    }
    cx.dispatch_command(command);
}
