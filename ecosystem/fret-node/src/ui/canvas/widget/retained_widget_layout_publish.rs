use super::*;

pub(super) fn publish_diagnostics_derived_outputs<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut LayoutCx<'_, H>,
    snapshot: &ViewSnapshot,
) {
    if canvas.diagnostics_anchor_ports.is_some() {
        let (geometry, _index) = canvas.canvas_derived(&*cx.app, snapshot);
        canvas.publish_derived_outputs(&*cx.app, snapshot, cx.bounds, &geometry);
    }
}
