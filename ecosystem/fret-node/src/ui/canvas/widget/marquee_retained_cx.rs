use fret_ui::UiHost;
use fret_ui::retained_bridge::EventCx;

use super::marquee_cx::MarqueeCx;

impl<H: UiHost> MarqueeCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn capture_self_pointer(&mut self) {
        self.capture_pointer(self.node);
    }
}
