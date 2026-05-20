use fret_ui::UiHost;
use fret_ui::retained_bridge::EventCx;

use super::pending_node_drag_release_cx::PendingNodeDragReleaseCx;

impl<H: UiHost> PendingNodeDragReleaseCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }
}
