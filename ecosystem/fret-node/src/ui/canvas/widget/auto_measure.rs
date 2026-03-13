mod key;

use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn update_auto_measured_node_sizes<H: UiHost>(&mut self, cx: &mut LayoutCx<'_, H>) {
        let key = key::auto_measured_key(self, cx);
        if self.auto_measured_key == Some(key) {
            return;
        }
        self.auto_measured_key = Some(key);

        let Some(nodes) = super::auto_measure_collect::collect_node_measure_inputs(self, cx.app)
        else {
            return;
        };
        let measured = super::auto_measure_apply::measure_node_sizes(self, cx, &nodes);
        super::auto_measure_apply::apply_measured_sizes(self, measured);
    }
}
