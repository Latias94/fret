use fret_ui::{UiHost, retained_bridge::EventCx};

use super::cancel_cx::CancelGestureCx;

impl<H: UiHost> CancelGestureCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }
}
