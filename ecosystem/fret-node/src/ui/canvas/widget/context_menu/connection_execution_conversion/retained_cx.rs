use fret_core::{AppWindowId, Point};
use fret_ui::{UiHost, retained_bridge::EventCx};

use crate::ui::canvas::widget::*;

use super::ConnectionConversionMenuCx;

impl<H: UiHost> ConnectionConversionMenuCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn window(&self) -> Option<AppWindowId> {
        self.window
    }

    fn restore_connection_conversion_wire_drag<M: NodeGraphCanvasMiddleware>(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        fallback_from: PortId,
        invoked_at: Point,
    ) {
        canvas.restore_connection_menu_wire_drag(self, fallback_from, invoked_at);
    }
}
