use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn node_default_size_for_ports(&self, inputs: usize, outputs: usize) -> (f32, f32) {
        let rows = inputs.max(outputs) as f32;
        let base = self.style.node_header_height + 2.0 * self.style.node_padding;
        let pin_area = rows * self.style.pin_row_height;
        (self.style.node_width, base + pin_area)
    }

    pub(super) fn reroute_pos_for_invoked_at(&self, invoked_at: Point) -> CanvasPoint {
        let (w, h) = self.node_default_size_for_ports(Self::REROUTE_INPUTS, Self::REROUTE_OUTPUTS);
        CanvasPoint {
            x: invoked_at.x.0 - 0.5 * w,
            y: invoked_at.y.0 - 0.5 * h,
        }
    }
}
