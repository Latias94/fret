use super::*;

pub(super) fn cmd_focus_next_node<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
) -> bool {
    let did = canvas.focus_next_node(cx.app, true);
    finish_focus_command(cx, did)
}

pub(super) fn cmd_focus_prev_node<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
) -> bool {
    let did = canvas.focus_next_node(cx.app, false);
    finish_focus_command(cx, did)
}

pub(super) fn cmd_focus_next_edge<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
) -> bool {
    let did = canvas.focus_next_edge(cx.app, true);
    finish_focus_command(cx, did)
}

pub(super) fn cmd_focus_prev_edge<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
) -> bool {
    let did = canvas.focus_next_edge(cx.app, false);
    finish_focus_command(cx, did)
}

pub(super) fn cmd_focus_next_port<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
) -> bool {
    let did = canvas.focus_next_port(cx.app, true);
    finish_focus_command(cx, did)
}

pub(super) fn cmd_focus_prev_port<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
) -> bool {
    let did = canvas.focus_next_port(cx.app, false);
    finish_focus_command(cx, did)
}

fn finish_focus_command<H: UiHost>(cx: &mut CommandCx<'_, H>, did: bool) -> bool {
    super::command_ui::finish_command_paint_if(cx, did)
}
