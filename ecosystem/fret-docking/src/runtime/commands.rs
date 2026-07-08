use fret_core::{AppWindowId, PanelKey};
use fret_runtime::{CreateWindowKind, CreateWindowRequest, Effect, UiHost, WindowRequest};

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

    fn len(&self) -> usize {
        self.commands.len()
    }

    fn take(&mut self) -> Vec<DockRuntimeCommand> {
        std::mem::take(&mut self.commands)
    }

    fn take_since(&mut self, baseline: usize) -> Vec<DockRuntimeCommand> {
        if baseline >= self.commands.len() {
            return Vec::new();
        }
        self.commands.drain(baseline..).collect()
    }

    fn remove_create_windows(&mut self, canceled: &[(AppWindowId, PanelKey)]) -> usize {
        if canceled.is_empty() {
            return 0;
        }
        let before = self.commands.len();
        self.commands.retain(|command| {
            let DockRuntimeCommand::CreateWindow(request) = command else {
                return true;
            };
            let CreateWindowKind::DockFloating {
                source_window,
                panel,
            } = &request.kind
            else {
                return true;
            };
            !canceled.iter().any(|(canceled_window, canceled_panel)| {
                canceled_window == source_window && canceled_panel == panel
            })
        });
        before.saturating_sub(self.commands.len())
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

pub(super) fn runtime_command_count<H: UiHost>(app: &mut H) -> usize {
    app.with_global_mut(DockRuntimeCommandQueue::default, |queue, _app| queue.len())
}

pub(super) fn take_runtime_commands_since<H: UiHost>(
    app: &mut H,
    baseline: usize,
) -> Vec<DockRuntimeCommand> {
    app.with_global_mut(DockRuntimeCommandQueue::default, |queue, _app| {
        queue.take_since(baseline)
    })
}

pub(super) fn remove_queued_create_windows<H: UiHost>(
    app: &mut H,
    canceled: &[(AppWindowId, PanelKey)],
) -> usize {
    app.with_global_mut(DockRuntimeCommandQueue::default, |queue, _app| {
        queue.remove_create_windows(canceled)
    })
}
