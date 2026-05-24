use fret_core::Rect;
use fret_ui::EventCx;
use fret_ui::UiHost;

use super::node_drag_move_cx::NodeDragMoveCx;

impl<H: UiHost> NodeDragMoveCx<H> for EventCx<'_, H> {
    fn bounds(&self) -> Rect {
        self.bounds
    }
}
