use fret_core::AppWindowId;
use fret_ui::{EventCx, UiHost};

use super::pointer_down_double_click_cx::PointerDownDoubleClickCx;

impl<H: UiHost> PointerDownDoubleClickCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn window(&self) -> Option<AppWindowId> {
        self.window
    }
}
