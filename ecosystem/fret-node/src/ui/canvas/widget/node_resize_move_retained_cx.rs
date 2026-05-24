use fret_ui::EventCx;
use fret_ui::UiHost;

use super::node_resize_move_cx::NodeResizeMoveCx;

impl<H: UiHost> NodeResizeMoveCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }
}
