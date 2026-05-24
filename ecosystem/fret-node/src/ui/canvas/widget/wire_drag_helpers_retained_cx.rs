use fret_ui::{EventCx, UiHost};

use super::wire_drag_helpers_cx::WireDragStartCx;

impl<H: UiHost> WireDragStartCx<H> for EventCx<'_, H> {
    fn capture_self_pointer(&mut self) {
        self.capture_pointer(self.node);
    }
}
