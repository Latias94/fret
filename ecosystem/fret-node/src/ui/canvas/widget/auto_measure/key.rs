use super::super::*;

pub(super) fn auto_measured_key<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &LayoutCx<'_, H>,
) -> (u64, u32) {
    let graph_rev = canvas.graph.revision(cx.app).unwrap_or(0);
    let scale_bits = cx.scale_factor.to_bits();
    (graph_rev, scale_bits)
}
