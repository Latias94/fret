use super::*;
use crate::ui::canvas::geometry::node_size_default_px;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn node_default_size_for_ports(&self, inputs: usize, outputs: usize) -> (f32, f32) {
        node_size_default_px(inputs, outputs, &self.style)
    }

    pub(super) fn reroute_pos_for_invoked_at(&self, invoked_at: Point) -> CanvasPoint {
        let (w, h) = self.node_default_size_for_ports(Self::REROUTE_INPUTS, Self::REROUTE_OUTPUTS);
        CanvasPoint {
            x: invoked_at.x.0 - 0.5 * w,
            y: invoked_at.y.0 - 0.5 * h,
        }
    }
}
