use fret_runtime::CommandId;
use fret_ui::{EventCx, UiHost};

use super::keyboard_shortcuts::KeyboardShortcutCommandSink;

impl<H: UiHost> KeyboardShortcutCommandSink for EventCx<'_, H> {
    fn dispatch_keyboard_command(&mut self, command: &'static str) {
        self.dispatch_command(CommandId::from(command));
        self.stop_propagation();
    }
}
