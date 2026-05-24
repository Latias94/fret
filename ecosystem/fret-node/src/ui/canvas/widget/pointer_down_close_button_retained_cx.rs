use fret_runtime::CommandId;
use fret_ui::{EventCx, UiHost};

use super::pointer_down_close_button_cx::PointerDownCloseButtonCx;

impl<H: UiHost> PointerDownCloseButtonCx<H> for EventCx<'_, H> {
    fn dispatch_close_command(&mut self, command: CommandId) {
        self.dispatch_command(command);
    }
}
