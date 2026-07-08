//! App/runner integration helpers for docking.
//!
//! Internal app/runner integration helpers for docking.
//!
//! Ordinary app code uses [`crate::DockSurface`]. This module remains crate-private so runtime
//! ordering, pending-window correlation, fallback, and close handling stay inside the docking
//! surface instead of becoming public application protocol.

use fret_core::{AppWindowId, DockNodeId, DockOp, PanelKey, WindowAnchor};
use fret_runtime::{CreateWindowRequest, UiHost};

mod apply;
mod auto_close;
mod before_close;
mod commands;
mod coordinator;
mod in_window;
mod layout_invalidation;
mod request;
mod tear_off;
mod window_created;

pub use commands::DockRuntimeCommand;
use coordinator::DockRuntimeCoordinator;
pub use in_window::recenter_in_window_floatings;
pub(crate) use tear_off::is_dock_floating_os_window;

/// Request docking layout invalidation for the provided windows.
///
/// This is a small app-layer integration hook: it bumps the internal invalidation models that the
/// dock host observes, forcing a layout pass on the next frame.
pub fn request_dock_invalidation<H: UiHost>(
    app: &mut H,
    windows: impl IntoIterator<Item = AppWindowId>,
) {
    layout_invalidation::invalidate_windows(app, windows);
}

pub(crate) fn handle_dock_op<H: UiHost>(app: &mut H, op: DockOp) -> bool {
    DockRuntimeCoordinator::host_effects().handle_dock_op(app, op)
}

/// Handle a docking transaction on the docking-owned runtime command route.
///
/// This is the route used by [`crate::DockSurface`]. Graph mutations are still applied through the
/// core `DockOp` vocabulary, while docking-owned OS-window creates/closes are queued as
/// [`DockRuntimeCommand`] instead of being pushed through the host effect queue.
pub(crate) fn handle_dock_op_with_runtime_commands<H: UiHost>(app: &mut H, op: DockOp) -> bool {
    DockRuntimeCoordinator::docking_commands().handle_dock_op(app, op)
}

pub(crate) fn request_float_panel_to_new_window<H: UiHost>(
    app: &mut H,
    source_window: AppWindowId,
    panel: PanelKey,
    anchor: Option<WindowAnchor>,
) -> bool {
    DockRuntimeCoordinator::docking_commands().handle_dock_op(
        app,
        DockOp::RequestFloatPanelToNewWindow {
            source_window,
            panel,
            anchor,
        },
    )
}

/// Queue a docking-owned OS-window tear-off command for a tab stack.
pub(crate) fn request_float_tabs_to_new_window<H: UiHost>(
    app: &mut H,
    source_window: AppWindowId,
    source_tabs: DockNodeId,
    panel: PanelKey,
    anchor: Option<WindowAnchor>,
) -> bool {
    DockRuntimeCoordinator::docking_commands().handle_dock_op(
        app,
        DockOp::RequestFloatTabsToNewWindow {
            source_window,
            source_tabs,
            panel,
            anchor,
        },
    )
}

/// Drain docking-owned runtime commands queued by [`crate::DockSurface`] or host adapters.
pub(crate) fn take_runtime_commands<H: UiHost>(app: &mut H) -> Vec<DockRuntimeCommand> {
    commands::take_runtime_commands(app)
}

/// Complete a dock floating window creation on the docking-owned command route.
///
/// Cancellation and missing-manager cleanup are queued as [`DockRuntimeCommand::CloseWindow`]
/// instead of being pushed through the host effect queue.
pub(crate) fn complete_queued_window_created<H: UiHost>(
    app: &mut H,
    request: &CreateWindowRequest,
    new_window: AppWindowId,
) -> bool {
    DockRuntimeCoordinator::docking_commands().window_created(app, request, new_window)
}

#[cfg(test)]
pub(crate) fn handle_dock_window_created<H: UiHost>(
    app: &mut H,
    request: &CreateWindowRequest,
    new_window: AppWindowId,
) -> bool {
    DockRuntimeCoordinator::host_effects().window_created(app, request, new_window)
}

pub(crate) fn handle_dock_before_close_window<H: UiHost>(
    app: &mut H,
    closing_window: AppWindowId,
    target_window: AppWindowId,
) -> bool {
    DockRuntimeCoordinator::host_effects().before_close_window(app, closing_window, target_window)
}

#[cfg(test)]
mod tests;
