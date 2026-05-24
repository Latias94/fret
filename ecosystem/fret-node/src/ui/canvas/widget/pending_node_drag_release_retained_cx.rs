use fret_ui::EventCx;
use fret_ui::UiHost;

use super::pending_node_drag_release_cx::PendingNodeDragReleaseCx;

impl<H: UiHost> PendingNodeDragReleaseCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }
}
