use super::*;

pub(super) fn focus_port_direction<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    dir: PortNavDir,
) -> bool {
    if !snapshot.interaction.elements_selectable {
        return false;
    }

    if canvas.interaction.focused_port.is_none() {
        return canvas.focus_next_port(host, true);
    }

    let Some(from_port) = canvas.interaction.focused_port else {
        return false;
    };
    let Some(from_center) = canvas.port_center_canvas(host, snapshot, from_port) else {
        return false;
    };

    let required_dir =
        super::focus_port_direction_candidate::required_port_direction_from_wire_drag(canvas, host);
    let Some(next) = super::focus_port_direction_candidate::directional_port_candidate(
        canvas,
        host,
        snapshot,
        from_port,
        from_center,
        dir,
        required_dir,
    ) else {
        return false;
    };

    super::focus_port_direction_apply::apply_directional_port_focus(canvas, host, next)
}
