mod close_button;
mod layers;

use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_overlays<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        zoom: f32,
        viewport_origin_x: f32,
        viewport_origin_y: f32,
        viewport_w: f32,
        viewport_h: f32,
    ) {
        close_button::paint_close_button(self, cx, snapshot, zoom);
        layers::paint_overlay_layers(
            self,
            cx,
            snapshot,
            zoom,
            viewport_origin_x,
            viewport_origin_y,
            viewport_w,
            viewport_h,
        );
    }
}
