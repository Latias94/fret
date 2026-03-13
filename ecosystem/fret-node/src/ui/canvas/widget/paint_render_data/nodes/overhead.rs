use super::super::*;

pub(super) fn node_render_label_overhead<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
) -> f32 {
    let node_pad = canvas.style.geometry.node_padding;
    let pin_gap = 8.0;
    let pin_r = canvas.style.geometry.pin_radius;
    2.0 * node_pad + 2.0 * (pin_r + pin_gap)
}
