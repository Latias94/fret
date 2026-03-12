mod apply;
mod collect;
mod preflight;
mod select;

use super::*;

pub(super) fn focus_next_port<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    forward: bool,
) -> bool {
    let Some(input) = preflight::traversal_input(canvas, host) else {
        return false;
    };

    let ports = collect::candidate_ports(canvas, host, input.focused_node, input.wire_dir);
    let Some(next) = select::next_port(&ports, canvas.interaction.focused_port, forward) else {
        return false;
    };

    apply::apply_port_focus(canvas, host, input.focused_node, next);
    true
}
