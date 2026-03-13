mod apply;
mod measure;

use super::*;

pub(super) fn measure_node_sizes<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut LayoutCx<'_, H>,
    nodes: &[super::auto_measure_collect::NodeMeasureInput],
) -> Vec<(GraphNodeId, (f32, f32))> {
    measure::measure_node_sizes(canvas, cx, nodes)
}

pub(super) fn apply_measured_sizes<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    measured: Vec<(GraphNodeId, (f32, f32))>,
) {
    apply::apply_measured_sizes(canvas, measured)
}
