use fret_core::PointerId;
use fret_runtime::TickId;
use fret_ui::{UiHost, retained_bridge::EventCx};

use super::arm::SearcherArmCx;

impl<H: UiHost> SearcherArmCx for EventCx<'_, H> {
    fn pointer_id(&self) -> Option<PointerId> {
        self.pointer_id
    }

    fn tick_id(&self) -> TickId {
        self.app.tick_id()
    }

    fn capture_pointer(&mut self) {
        EventCx::capture_pointer(self, self.node);
    }
}
