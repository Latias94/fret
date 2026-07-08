use fret_core::AppWindowId;
use fret_runtime::{CreateWindowRequest, Effect, UiHost, WindowRequest};

#[derive(Debug, Clone, PartialEq)]
pub enum DockRuntimeCommand {
    CreateWindow(CreateWindowRequest),
    CloseWindow(AppWindowId),
}

#[derive(Clone, Copy)]
pub(super) enum CloseWindowDispatch {
    Effect,
    CommandQueue,
}

#[derive(Default)]
pub(super) struct DockRuntimeCommandQueue {
    commands: Vec<DockRuntimeCommand>,
}

impl DockRuntimeCommandQueue {
    fn push(&mut self, command: DockRuntimeCommand) {
        self.commands.push(command);
    }

    fn take(&mut self) -> Vec<DockRuntimeCommand> {
        std::mem::take(&mut self.commands)
    }
}

pub(super) fn push_runtime_command<H: UiHost>(app: &mut H, command: DockRuntimeCommand) {
    app.with_global_mut(DockRuntimeCommandQueue::default, |queue, _app| {
        queue.push(command);
    });
}

pub(super) fn queue_close_window<H: UiHost>(app: &mut H, window: AppWindowId) {
    push_runtime_command(app, DockRuntimeCommand::CloseWindow(window));
}

pub(super) fn dispatch_close_window<H: UiHost>(
    app: &mut H,
    dispatch: CloseWindowDispatch,
    window: AppWindowId,
) {
    match dispatch {
        CloseWindowDispatch::Effect => {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
        }
        CloseWindowDispatch::CommandQueue => {
            queue_close_window(app, window);
        }
    }
}

pub(super) fn take_runtime_commands<H: UiHost>(app: &mut H) -> Vec<DockRuntimeCommand> {
    app.with_global_mut(DockRuntimeCommandQueue::default, |queue, _app| queue.take())
}
