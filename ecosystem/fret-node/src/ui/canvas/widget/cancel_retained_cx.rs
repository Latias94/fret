use fret_ui::{EventCx, UiHost};

use super::cancel_cx::CancelGestureCx;

impl<H: UiHost> CancelGestureCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }
}
