use fret_ui::UiHost;
use fret_ui::retained_bridge::EventCx;

use super::node_resize_move_cx::NodeResizeMoveCx;

impl<H: UiHost> NodeResizeMoveCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }
}
