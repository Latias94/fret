use super::super::*;
use super::finish_command_paint;

pub(super) fn cmd_create_group<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
) -> bool {
    let at = canvas.interaction.last_canvas_pos.unwrap_or_default();
    canvas.create_group_at(cx.app, cx.window, at);
    finish_command_paint(cx)
}
