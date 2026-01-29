use fret_core::{AppWindowId, DockOp};
use fret_runtime::{CreateWindowRequest, UiHost};

/// Driver-facing docking integration facade.
///
/// This type intentionally stays platform-agnostic and only wraps the crate-level runtime helpers.
#[derive(Debug, Clone, Copy)]
pub struct DockingRuntime {
    main_window: AppWindowId,
}

impl DockingRuntime {
    pub fn new(main_window: AppWindowId) -> Self {
        Self { main_window }
    }

    pub fn main_window(&self) -> AppWindowId {
        self.main_window
    }

    pub fn on_dock_op<H: UiHost>(&self, app: &mut H, op: DockOp) -> bool {
        crate::runtime::handle_dock_op(app, op)
    }

    pub fn on_window_created<H: UiHost>(
        &self,
        app: &mut H,
        request: &CreateWindowRequest,
        new_window: AppWindowId,
    ) -> bool {
        crate::runtime::handle_dock_window_created(app, request, new_window)
    }

    pub fn before_close_window<H: UiHost>(&self, app: &mut H, closing_window: AppWindowId) -> bool {
        crate::runtime::handle_dock_before_close_window(app, closing_window, self.main_window)
    }
}
