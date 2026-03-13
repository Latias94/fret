use crate::ui::canvas::widget::*;

pub(super) fn apply_port_focus<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    focused_node: GraphNodeId,
    next: PortId,
) {
    super::super::focus_session::focus_port(&mut canvas.interaction, focused_node, next);
    canvas.refresh_focused_port_hints(host);
    canvas.update_view_state(host, |s| {
        super::super::focus_session::select_only_node(s, focused_node, false);
    });
}
