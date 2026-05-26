//! Retained-context bindings for the command dispatch adapter contract.

use fret_runtime::CommandId;
use fret_ui::{EventCx, UiHost};

use super::command_adapter;

impl<H: UiHost> command_adapter::CanvasCommandDispatchCx for EventCx<'_, H> {
    fn dispatch_canvas_command(&mut self, command: CommandId) {
        EventCx::dispatch_command(self, command);
    }
}
