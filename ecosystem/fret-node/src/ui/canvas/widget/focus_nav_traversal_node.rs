mod apply;
mod collect;
mod preflight;
mod select;

use super::*;

pub(super) fn focus_next_node<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    forward: bool,
) -> bool {
    let Some(snapshot) = preflight::traversal_snapshot(canvas, host) else {
        return false;
    };

    let ordered = collect::ordered_selectable_nodes::<M>(canvas, host, &snapshot);
    let Some(next) = select::next_node(
        &ordered,
        preflight::current_node(canvas, &snapshot),
        forward,
    ) else {
        return false;
    };

    apply::apply_node_focus(canvas, host, next);
    true
}
