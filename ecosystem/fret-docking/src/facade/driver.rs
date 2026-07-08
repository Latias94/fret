use fret_core::{AppWindowId, DockGraph, DockNodeId, DockOp, PanelKey, WindowAnchor};
use fret_runtime::{CreateWindowRequest, Effect, UiHost, WindowRequest};

use crate::dock::DockManager;
use crate::runtime::DockRuntimeCommand;

use super::DockSurface;

/// Explicit host/runtime driver for docking surface integration.
///
/// Ordinary app code should prefer [`DockSurface`] methods. This tier is intentionally separate
/// because it deals in graph construction callbacks, dock operations, runtime commands, and window
/// lifecycle handshakes.
#[derive(Debug, Clone, Copy)]
pub struct DockSurfaceDriver {
    pub(super) surface: DockSurface,
}

impl DockSurfaceDriver {
    pub fn new(surface: DockSurface) -> Self {
        Self { surface }
    }

    pub fn main_window(&self) -> AppWindowId {
        self.surface.main_window
    }

    pub fn ensure_window_root<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
        make_root: impl FnOnce(&mut DockGraph) -> DockNodeId,
    ) -> bool {
        app.with_global_mut(DockManager::default, |dock, _app| {
            if dock.workspace.graph.window_root(window).is_some() {
                return false;
            }
            let root = make_root(&mut dock.workspace.graph);
            dock.workspace.graph.set_window_root(window, root);
            true
        })
    }

    pub fn request_float_panel_to_new_window<H: UiHost>(
        &self,
        app: &mut H,
        source_window: AppWindowId,
        panel: PanelKey,
        anchor: Option<WindowAnchor>,
    ) -> bool {
        crate::runtime::request_float_panel_to_new_window(app, source_window, panel, anchor)
    }

    pub fn request_float_tabs_to_new_window<H: UiHost>(
        &self,
        app: &mut H,
        source_window: AppWindowId,
        source_tabs: DockNodeId,
        panel: PanelKey,
        anchor: Option<WindowAnchor>,
    ) -> bool {
        crate::runtime::request_float_tabs_to_new_window(
            app,
            source_window,
            source_tabs,
            panel,
            anchor,
        )
    }

    pub fn take_runtime_commands<H: UiHost>(&self, app: &mut H) -> Vec<DockRuntimeCommand> {
        crate::runtime::take_runtime_commands(app)
    }

    pub fn flush_runtime_commands_to_effects<H: UiHost>(&self, app: &mut H) -> usize {
        let commands = self.take_runtime_commands(app);
        self.push_runtime_commands_to_effects(app, commands)
    }

    pub(super) fn runtime_command_cursor<H: UiHost>(&self, app: &mut H) -> u64 {
        crate::runtime::runtime_command_cursor(app)
    }

    pub(super) fn flush_runtime_commands_since_to_effects<H: UiHost>(
        &self,
        app: &mut H,
        cursor: u64,
    ) -> usize {
        let commands = crate::runtime::take_runtime_commands_since(app, cursor);
        self.push_runtime_commands_to_effects(app, commands)
    }

    fn push_runtime_commands_to_effects<H: UiHost>(
        &self,
        app: &mut H,
        commands: Vec<DockRuntimeCommand>,
    ) -> usize {
        let count = commands.len();
        for command in commands {
            match command {
                DockRuntimeCommand::CreateWindow(request) => {
                    app.push_effect(Effect::Window(WindowRequest::Create(request)));
                }
                DockRuntimeCommand::CloseWindow(window) => {
                    app.push_effect(Effect::Window(WindowRequest::Close(window)));
                }
            }
        }
        count
    }

    pub fn on_dock_op<H: UiHost>(&self, app: &mut H, op: DockOp) -> bool {
        crate::runtime::handle_dock_op_with_runtime_commands(app, op)
    }

    pub fn on_window_created<H: UiHost>(
        &self,
        app: &mut H,
        request: &CreateWindowRequest,
        new_window: AppWindowId,
    ) -> bool {
        crate::runtime::complete_queued_window_created(app, request, new_window)
    }

    pub fn before_close_window<H: UiHost>(&self, app: &mut H, closing_window: AppWindowId) -> bool {
        self.before_close_window_into(app, closing_window, self.surface.main_window)
    }

    pub fn before_close_window_into<H: UiHost>(
        &self,
        app: &mut H,
        closing_window: AppWindowId,
        target_window: AppWindowId,
    ) -> bool {
        crate::runtime::handle_dock_before_close_window(app, closing_window, target_window)
    }
}
