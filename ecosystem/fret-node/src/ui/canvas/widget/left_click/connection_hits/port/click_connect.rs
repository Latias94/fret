use fret_core::Point;
use fret_ui::UiHost;

use crate::core::PortId;
use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) enum PortClickConnectHit {
    NotHandled,
    Handled,
}

pub(super) fn handle_click_connect_port_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
    port: PortId,
    port_connectable_end: bool,
) -> PortClickConnectHit {
    if !should_handle_click_connect_port_hit(
        snapshot.interaction.connect_on_click,
        canvas.interaction.click_connect,
        canvas.interaction.wire_drag.is_some(),
    ) {
        return PortClickConnectHit::NotHandled;
    }
    if !port_connectable_end {
        return PortClickConnectHit::Handled;
    }

    let Some(mut wire_drag) = canvas.interaction.wire_drag.take() else {
        return PortClickConnectHit::NotHandled;
    };

    wire_drag.pos = position;
    canvas.interaction.wire_drag = Some(wire_drag);
    canvas.interaction.click_connect = false;
    canvas.interaction.pending_wire_drag = None;
    let _ = crate::ui::canvas::widget::wire_drag::handle_wire_left_up_with_forced_target(
        canvas,
        cx,
        snapshot,
        zoom,
        Some(port),
    );
    PortClickConnectHit::Handled
}

fn should_handle_click_connect_port_hit(
    connect_on_click: bool,
    click_connect: bool,
    has_wire_drag: bool,
) -> bool {
    connect_on_click && click_connect && has_wire_drag
}

#[cfg(test)]
mod tests;
