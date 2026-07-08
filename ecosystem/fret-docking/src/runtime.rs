//! App/runner integration helpers for docking.
//!
//! The docking UI emits high-level `DockOp` transactions via `Effect::Dock(...)` (ADR 0013).
//! These ops must be applied by the app/runner layer:
//! - apply graph mutations (`DockGraph::apply_op`)
//! - translate `RequestFloatPanelToNewWindow` into a `WindowRequest::Create`
//! - translate `RequestFloatTabsToNewWindow` into a `WindowRequest::Create`
//! - complete the float by updating the graph once the OS window exists

use fret_core::{AppWindowId, DockNodeId, DockOp, PanelKey, WindowAnchor};
use fret_runtime::{CreateWindowRequest, UiHost};

mod apply;
mod auto_close;
mod before_close;
mod commands;
mod in_window;
mod layout_invalidation;
mod request;
mod tear_off;
mod window_created;

pub use commands::DockRuntimeCommand;
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

/// Handle a docking transaction emitted by the UI layer.
///
/// Call this from your runner/driver when consuming `Effect::Dock(op)`.
pub fn handle_dock_op<H: UiHost>(app: &mut H, op: DockOp) -> bool {
    match op {
        op @ DockOp::RequestFloatPanelToNewWindow { .. } => {
            request::handle_request_float_to_new_window(app, op)
        }
        op @ DockOp::RequestFloatTabsToNewWindow { .. } => {
            request::handle_request_float_to_new_window(app, op)
        }
        op => apply::handle_applied_dock_op(app, op),
    }
}

/// Handle a docking transaction on the docking-owned runtime command route.
///
/// This is the route used by [`crate::DockSurface`]. Graph mutations are still applied through the
/// core `DockOp` vocabulary, while docking-owned OS-window creates/closes are queued as
/// [`DockRuntimeCommand`] instead of being pushed through the host effect queue.
pub(crate) fn handle_dock_op_with_runtime_commands<H: UiHost>(app: &mut H, op: DockOp) -> bool {
    match op {
        op @ DockOp::RequestFloatPanelToNewWindow { .. } => {
            request::queue_request_float_to_new_window(app, op)
        }
        op @ DockOp::RequestFloatTabsToNewWindow { .. } => {
            request::queue_request_float_to_new_window(app, op)
        }
        op => apply::queue_applied_dock_op(app, op),
    }
}

pub(crate) fn request_float_panel_to_new_window<H: UiHost>(
    app: &mut H,
    source_window: AppWindowId,
    panel: PanelKey,
    anchor: Option<WindowAnchor>,
) -> bool {
    request::queue_request_float_to_new_window(
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
    request::queue_request_float_to_new_window(
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
    window_created::queue_dock_window_created(app, request, new_window)
}

/// Complete a dock floating window creation by updating the dock graph.
///
/// Call this from your runner/driver `window_created(...)` callback.
pub fn handle_dock_window_created<H: UiHost>(
    app: &mut H,
    request: &CreateWindowRequest,
    new_window: AppWindowId,
) -> bool {
    window_created::handle_dock_window_created(app, request, new_window)
}

/// Merge a closing floating dock window back into a target window.
///
/// This matches the common editor UX expectation that closing a floating dock window keeps its
/// panels alive by merging them into a stable target (usually the main window).
///
/// Call this from your runner/driver `before_close_window(...)` hook.
pub fn handle_dock_before_close_window<H: UiHost>(
    app: &mut H,
    closing_window: AppWindowId,
    target_window: AppWindowId,
) -> bool {
    before_close::handle_dock_before_close_window(app, closing_window, target_window)
}

#[cfg(test)]
mod tests;
