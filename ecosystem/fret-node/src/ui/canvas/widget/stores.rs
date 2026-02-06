use super::*;

mod internals;
mod measured;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn publish_derived_outputs<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
        bounds: Rect,
        geom: &CanvasGeometry,
    ) {
        self.update_measured_output_store(snapshot.zoom, geom);
        self.update_internals_store(host, snapshot, bounds, geom);
    }
}
