use fret_ui::{UiHost, retained_bridge::EventCx};

use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use super::SearcherInputCx;

impl<H: UiHost, M: NodeGraphCanvasMiddleware> SearcherInputCx<H, M> for EventCx<'_, H> {
    fn try_activate_searcher_row(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        row_ix: usize,
    ) -> bool {
        canvas.try_activate_searcher_row(self, row_ix)
    }
}
