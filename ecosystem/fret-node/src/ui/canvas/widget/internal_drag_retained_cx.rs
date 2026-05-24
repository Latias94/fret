use fret_core::{AppWindowId, PointerId};
use fret_runtime::DragSession;
use fret_ui::{EventCx, UiHost};

use super::internal_drag_cx::InternalDragCx;

impl<H: UiHost> InternalDragCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn window(&self) -> Option<AppWindowId> {
        self.window
    }

    fn drag_session(&self, pointer_id: PointerId) -> Option<&DragSession> {
        self.app.drag(pointer_id)
    }
}
