use fret_core::AppWindowId;
use fret_ui::{EventCx, UiHost};

use crate::ui::canvas::widget::*;

use super::EdgeContextActionCx;

impl<H: UiHost> EdgeContextActionCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn window(&self) -> Option<AppWindowId> {
        self.window
    }

    fn open_edge_insert_context_menu<M: NodeGraphCanvasMiddleware>(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        edge_id: EdgeId,
        invoked_at: Point,
    ) {
        edge_insert::open_edge_insert_context_menu(canvas, self, edge_id, invoked_at);
    }
}
