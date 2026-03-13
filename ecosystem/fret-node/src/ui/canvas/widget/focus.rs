use super::*;
use crate::ui::canvas::state::DrawOrderFingerprint;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn repair_focused_edge_after_graph_change<H: UiHost>(
        &mut self,
        host: &mut H,
        preferred: Option<EdgeId>,
    ) {
        focus_edge_repair::repair_focused_edge_after_graph_change(self, host, preferred)
    }

    pub(super) fn draw_order_fingerprint(ids: &[GraphNodeId]) -> DrawOrderFingerprint {
        focus_draw_order::draw_order_fingerprint(ids)
    }

    pub(super) fn focus_port_direction<H: UiHost>(
        &mut self,
        host: &mut H,
        snapshot: &ViewSnapshot,
        dir: PortNavDir,
    ) -> bool {
        focus_port_direction::focus_port_direction(self, host, snapshot, dir)
    }
}
