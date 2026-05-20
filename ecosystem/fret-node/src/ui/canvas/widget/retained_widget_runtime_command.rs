use super::*;

pub(super) fn handle_retained_command<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    command: &CommandId,
) -> bool {
    super::retained_widget_runtime_shared::sync_runtime_theme(
        canvas,
        cx.theme().snapshot(),
        Some(cx.services),
    );
    let snapshot = canvas.sync_view_state(cx.app);
    if should_defer_command_to_text_input(cx, command) {
        return false;
    }

    canvas.handle_command(cx, &snapshot, command)
}

fn should_defer_command_to_text_input<H: UiHost>(
    cx: &CommandCx<'_, H>,
    command: &CommandId,
) -> bool {
    cx.input_ctx.focus_is_text_input
        && (command.as_str().starts_with("node_graph.")
            || matches!(
                command.as_str(),
                "edit.copy" | "edit.cut" | "edit.paste" | "edit.select_all"
            ))
}
