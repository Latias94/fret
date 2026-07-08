use fret_core::{AppWindowId, DockOp};
use fret_runtime::{CreateWindowRequest, UiHost};

use super::{DockSurface, DockSurfaceDriver};

/// App-facing host lifecycle adapter for docking callbacks.
///
/// This wraps the advanced driver so ordinary host code does not manually manage runtime command
/// cursors or flush ordering.
#[derive(Debug, Clone, Copy)]
pub struct DockSurfaceHostSession {
    pub(super) surface: DockSurface,
}

impl DockSurfaceHostSession {
    pub fn on_dock_op<H: UiHost>(&self, app: &mut H, op: DockOp) -> bool {
        let driver = DockSurfaceDriver::new(self.surface);
        let command_cursor = driver.runtime_command_cursor(app);
        let changed = driver.on_dock_op(app, op);
        driver.flush_runtime_commands_since_to_effects(app, command_cursor);
        changed
    }

    pub fn on_window_created<H: UiHost>(
        &self,
        app: &mut H,
        request: &CreateWindowRequest,
        new_window: AppWindowId,
    ) -> bool {
        let driver = DockSurfaceDriver::new(self.surface);
        let command_cursor = driver.runtime_command_cursor(app);
        let changed = driver.on_window_created(app, request, new_window);
        driver.flush_runtime_commands_since_to_effects(app, command_cursor);
        changed
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
        let driver = DockSurfaceDriver::new(self.surface);
        let command_cursor = driver.runtime_command_cursor(app);
        let changed = driver.before_close_window_into(app, closing_window, target_window);
        driver.flush_runtime_commands_since_to_effects(app, command_cursor);
        changed
    }
}
