use fret_core::{AppWindowId, PointerId, Rect};
use fret_runtime::TickId;
use fret_ui::EventCx;
use fret_ui::UiHost;

use super::insert_node_drag_move_cx::InsertNodeDragMoveCx;

impl<H: UiHost> InsertNodeDragMoveCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn pointer_id(&self) -> Option<PointerId> {
        self.pointer_id
    }

    fn window(&self) -> Option<AppWindowId> {
        self.window
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn tick_id(&self) -> TickId {
        self.app.tick_id()
    }
}
