//! Command dispatch adapter contract for retained compatibility host operations.
//!
//! This module is intentionally retained-context agnostic. Concrete bindings to `fret_ui`
//! retained contexts live in `retained_command_adapter.rs`.

use fret_runtime::CommandId;

pub(super) trait CanvasCommandDispatchCx {
    fn dispatch_canvas_command(&mut self, command: CommandId);
}

pub(super) fn dispatch_canvas_command(cx: &mut impl CanvasCommandDispatchCx, command: CommandId) {
    cx.dispatch_canvas_command(command);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct StubCx {
        commands: Vec<CommandId>,
    }

    impl CanvasCommandDispatchCx for StubCx {
        fn dispatch_canvas_command(&mut self, command: CommandId) {
            self.commands.push(command);
        }
    }

    #[test]
    fn dispatch_canvas_command_forwards_command_to_adapter() {
        let mut cx = StubCx::default();
        let command = CommandId::from("node_graph.close");

        dispatch_canvas_command(&mut cx, command.clone());

        assert_eq!(cx.commands, vec![command]);
    }
}
