use fret_ui::EventCx;
use fret_ui::UiHost;

use super::node_drag_move_tail_cx::NodeDragMoveTailCx;

impl<H: UiHost> NodeDragMoveTailCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }
}
