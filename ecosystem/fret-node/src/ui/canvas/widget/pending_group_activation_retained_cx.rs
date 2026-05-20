use fret_ui::UiHost;
use fret_ui::retained_bridge::EventCx;

use super::pending_group_activation_cx::PendingGroupActivationCx;

impl<H: UiHost> PendingGroupActivationCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }
}
