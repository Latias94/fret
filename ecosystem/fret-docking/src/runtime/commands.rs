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
    commands: Vec<QueuedDockRuntimeCommand>,
    next_sequence: u64,
}

struct QueuedDockRuntimeCommand {
    sequence: u64,
    command: DockRuntimeCommand,
}

impl DockRuntimeCommandQueue {
    fn push(&mut self, command: DockRuntimeCommand) {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.commands
            .push(QueuedDockRuntimeCommand { sequence, command });
    }

    fn cursor(&self) -> u64 {
        self.next_sequence
    }

    fn take(&mut self) -> Vec<DockRuntimeCommand> {
        std::mem::take(&mut self.commands)
            .into_iter()
            .map(|queued| queued.command)
            .collect()
    }

    fn take_since(&mut self, cursor: u64) -> Vec<DockRuntimeCommand> {
        let mut taken = Vec::new();
        let mut retained = Vec::with_capacity(self.commands.len());
        for queued in self.commands.drain(..) {
            if queued.sequence >= cursor {
                taken.push(queued.command);
            } else {
                retained.push(queued);
            }
        }
        self.commands = retained;
        taken
    }

    fn remove_create_windows(&mut self, canceled: &[(AppWindowId, PanelKey)]) -> usize {
        if canceled.is_empty() {
            return 0;
        }
        let before = self.commands.len();
        self.commands.retain(|queued| {
            let DockRuntimeCommand::CreateWindow(request) = &queued.command else {
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

pub(super) fn runtime_command_cursor<H: UiHost>(app: &mut H) -> u64 {
    app.with_global_mut(DockRuntimeCommandQueue::default, |queue, _app| {
        queue.cursor()
    })
}

pub(super) fn take_runtime_commands_since<H: UiHost>(
    app: &mut H,
    cursor: u64,
) -> Vec<DockRuntimeCommand> {
    app.with_global_mut(DockRuntimeCommandQueue::default, |queue, _app| {
        queue.take_since(cursor)
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
