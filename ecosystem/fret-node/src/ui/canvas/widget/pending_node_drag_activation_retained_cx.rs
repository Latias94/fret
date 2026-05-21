use fret_ui::UiHost;
use fret_ui::retained_bridge::EventCx;

use super::pending_node_drag_activation_cx::PendingNodeDragActivationCx;

impl<H: UiHost> PendingNodeDragActivationCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }
}
