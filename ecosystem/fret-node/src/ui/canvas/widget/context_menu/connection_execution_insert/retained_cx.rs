use fret_core::{AppWindowId, Point};
use fret_ui::{UiHost, retained_bridge::EventCx};

use crate::ui::canvas::widget::*;

use super::ConnectionInsertMenuCx;

impl<H: UiHost> ConnectionInsertMenuCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn window(&self) -> Option<AppWindowId> {
        self.window
    }

    fn resume_connection_insert_wire_drag<M: NodeGraphCanvasMiddleware>(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        fallback_from: PortId,
        invoked_at: Point,
        continue_from: Option<PortId>,
    ) {
        let resume_pos = canvas.interaction.last_pos.unwrap_or(invoked_at);
        if let Some(port) = continue_from {
            canvas.interaction.suspended_wire_drag = None;
            canvas.start_sticky_wire_drag_from_port(self, port, resume_pos);
        } else {
            canvas.restore_suspended_wire_drag(self, Some(fallback_from), resume_pos);
        }
    }

    fn restore_connection_menu_wire_drag<M: NodeGraphCanvasMiddleware>(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        fallback_from: PortId,
        invoked_at: Point,
    ) {
        let resume_pos = canvas.interaction.last_pos.unwrap_or(invoked_at);
        canvas.restore_suspended_wire_drag(self, Some(fallback_from), resume_pos);
    }
}
