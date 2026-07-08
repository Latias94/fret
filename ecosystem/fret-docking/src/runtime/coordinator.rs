use fret_core::{AppWindowId, DockOp};
use fret_runtime::{CreateWindowRequest, UiHost};

use super::{apply, before_close, request, window_created};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DockRuntimeCommandRoute {
    HostEffects,
    DockingCommands,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DockRuntimeCoordinator {
    route: DockRuntimeCommandRoute,
}

impl DockRuntimeCoordinator {
    pub(super) fn host_effects() -> Self {
        Self {
            route: DockRuntimeCommandRoute::HostEffects,
        }
    }

    pub(super) fn docking_commands() -> Self {
        Self {
            route: DockRuntimeCommandRoute::DockingCommands,
        }
    }

    pub(super) fn handle_dock_op<H: UiHost>(&self, app: &mut H, op: DockOp) -> bool {
        match op {
            op @ DockOp::RequestFloatPanelToNewWindow { .. }
            | op @ DockOp::RequestFloatTabsToNewWindow { .. } => match self.route {
                DockRuntimeCommandRoute::HostEffects => {
                    request::handle_request_float_to_new_window(app, op)
                }
                DockRuntimeCommandRoute::DockingCommands => {
                    request::queue_request_float_to_new_window(app, op)
                }
            },
            op => match self.route {
                DockRuntimeCommandRoute::HostEffects => apply::handle_applied_dock_op(app, op),
                DockRuntimeCommandRoute::DockingCommands => apply::queue_applied_dock_op(app, op),
            },
        }
    }

    pub(super) fn window_created<H: UiHost>(
        &self,
        app: &mut H,
        request: &CreateWindowRequest,
        new_window: AppWindowId,
    ) -> bool {
        match self.route {
            DockRuntimeCommandRoute::HostEffects => {
                window_created::handle_dock_window_created(app, request, new_window)
            }
            DockRuntimeCommandRoute::DockingCommands => {
                window_created::queue_dock_window_created(app, request, new_window)
            }
        }
    }

    pub(super) fn before_close_window<H: UiHost>(
        &self,
        app: &mut H,
        closing_window: AppWindowId,
        target_window: AppWindowId,
    ) -> bool {
        before_close::handle_dock_before_close_window(app, closing_window, target_window)
    }
}
