use super::*;

impl<H: UiHost> prepaint_cull_window_adapter::PrepaintCullWindowCx<H> for PrepaintCx<'_, H> {
    fn prepaint_cull_window_host(&mut self) -> &mut H {
        self.app
    }

    fn prepaint_cull_window_bounds(&self) -> Rect {
        self.bounds
    }

    fn record_node_graph_cull_window_shift(&mut self, cull_window_key: u64) {
        self.debug_record_node_graph_cull_window_shift(cull_window_key);
    }
}

pub(super) fn prepaint_cull_window<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PrepaintCx<'_, H>,
) {
    prepaint_cull_window_adapter::sync_prepaint_cull_window(canvas, cx);
}
