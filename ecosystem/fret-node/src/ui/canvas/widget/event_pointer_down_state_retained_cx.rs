use fret_ui::{EventCx, UiHost};

use super::event_pointer_down_state_cx::PointerDownStateCx;

impl<H: UiHost> PointerDownStateCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }
}
