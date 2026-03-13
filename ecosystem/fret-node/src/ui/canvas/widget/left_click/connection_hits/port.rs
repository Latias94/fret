mod click_connect;
mod connectable;
mod kind;

use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use crate::core::PortId;
use crate::ui::canvas::state::{ViewSnapshot, WireDragKind};
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn handle_port_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    zoom: f32,
    port: PortId,
) {
    super::super::super::focus_session::clear_edge_focus(&mut canvas.interaction);
    let connectability = connectable::port_connectability(canvas, cx.app, snapshot, port);

    if let click_connect::PortClickConnectHit::Handled =
        click_connect::handle_click_connect_port_hit(
            canvas,
            cx,
            snapshot,
            position,
            zoom,
            port,
            connectability.end,
        )
    {
        return;
    }

    if !connectability.base {
        canvas.interaction.click_connect = false;
        return;
    }

    super::super::super::press_session::prepare_for_port_hit(&mut canvas.interaction);
    let kind = kind::wire_drag_kind_for_port_hit(canvas, cx.app, snapshot, modifiers, port);

    if matches!(kind, WireDragKind::New { .. }) && !connectability.start {
        return;
    }

    kind::arm_pending_wire_drag(canvas, cx, kind, position);
}
