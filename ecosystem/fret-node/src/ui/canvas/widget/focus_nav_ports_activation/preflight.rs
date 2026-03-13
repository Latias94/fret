use crate::ui::canvas::widget::*;

pub(super) struct ActivationTarget {
    pub port: PortId,
    pub position: Point,
}

pub(super) fn activation_target<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
) -> Option<ActivationTarget> {
    if !snapshot.interaction.elements_selectable {
        return None;
    }

    let port = canvas
        .interaction
        .focused_port
        .or(canvas.interaction.hover_port)?;
    let position = super::super::focus_nav_ports_center::focused_port_activation_point(
        canvas, host, snapshot, port,
    );

    Some(ActivationTarget { port, position })
}
