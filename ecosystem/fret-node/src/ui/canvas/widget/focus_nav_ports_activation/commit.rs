use crate::ui::canvas::widget::*;

pub(super) fn commit_click_connect_wire_drag<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
    port: PortId,
    position: Point,
) {
    sync_wire_drag_position(canvas, position);
    let _ = wire_drag::handle_wire_left_up_with_forced_target(
        canvas,
        cx,
        snapshot,
        snapshot.zoom,
        Some(port),
    );
    super::super::focus_nav_ports_hints::refresh_focused_port_hints(canvas, cx.app);
}

fn sync_wire_drag_position<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    position: Point,
) {
    if let Some(mut wire_drag) = canvas.interaction.wire_drag.take() {
        wire_drag.pos = position;
        canvas.interaction.wire_drag = Some(wire_drag);
    }
}
