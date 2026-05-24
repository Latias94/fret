use fret_ui::{EventCx, UiHost};

use super::event::ContextMenuFocusCx;

impl<H: UiHost> ContextMenuFocusCx<H> for EventCx<'_, H> {
    fn request_context_menu_focus(&mut self) {
        EventCx::request_focus(self, self.node);
    }
}
