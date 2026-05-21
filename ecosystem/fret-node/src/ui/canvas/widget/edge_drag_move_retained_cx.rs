use fret_ui::UiHost;
use fret_ui::retained_bridge::EventCx;

use super::edge_drag_move_cx::EdgeDragMoveCx;

impl<H: UiHost> EdgeDragMoveCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }
}
