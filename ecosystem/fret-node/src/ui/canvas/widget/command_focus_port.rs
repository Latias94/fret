use super::*;

pub(super) fn cmd_focus_port_left<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    let did = canvas.focus_port_direction(cx.app, snapshot, PortNavDir::Left);
    finish_focus_command(cx, did)
}

pub(super) fn cmd_focus_port_right<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    let did = canvas.focus_port_direction(cx.app, snapshot, PortNavDir::Right);
    finish_focus_command(cx, did)
}

pub(super) fn cmd_focus_port_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    let did = canvas.focus_port_direction(cx.app, snapshot, PortNavDir::Up);
    finish_focus_command(cx, did)
}

pub(super) fn cmd_focus_port_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    let did = canvas.focus_port_direction(cx.app, snapshot, PortNavDir::Down);
    finish_focus_command(cx, did)
}

pub(super) fn cmd_activate<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    let did = canvas.activate_focused_port(cx, snapshot);
    finish_focus_command(cx, did)
}

fn finish_focus_command<H: UiHost>(cx: &mut CommandCx<'_, H>, did: bool) -> bool {
    super::command_ui::finish_command_paint_if(cx, did)
}
