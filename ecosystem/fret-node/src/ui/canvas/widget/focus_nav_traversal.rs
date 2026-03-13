use super::*;

pub(super) fn focus_next_edge<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    forward: bool,
) -> bool {
    super::focus_nav_traversal_edge::focus_next_edge(canvas, host, forward)
}

pub(super) fn focus_next_node<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    forward: bool,
) -> bool {
    super::focus_nav_traversal_node::focus_next_node(canvas, host, forward)
}

pub(super) fn focus_next_port<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    forward: bool,
) -> bool {
    super::focus_nav_traversal_port::focus_next_port(canvas, host, forward)
}
