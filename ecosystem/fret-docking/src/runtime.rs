//! App/runner integration helpers for docking.
//!
//! The docking UI emits high-level `DockOp` transactions via `Effect::Dock(...)` (ADR 0013).
//! These ops must be applied by the app/runner layer:
//! - apply graph mutations (`DockGraph::apply_op`)
//! - translate `RequestFloatPanelToNewWindow` into a `WindowRequest::Create`
//! - translate `RequestFloatTabsToNewWindow` into a `WindowRequest::Create`
//! - complete the float by updating the graph once the OS window exists

use fret_core::{AppWindowId, DockOp};
use fret_runtime::{CreateWindowRequest, UiHost};

use crate::DockManager;

mod auto_close;
mod before_close;
mod in_window;
mod layout_invalidation;
mod request;
mod tear_off;
mod window_created;

pub use in_window::recenter_in_window_floatings;
use tear_off::DockTearOffMachine;
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
        op => {
            if app.global::<DockManager>().is_none() {
                return false;
            }

            let tearoff_log =
                std::env::var_os("FRET_DOCK_TEAROFF_LOG").is_some_and(|v| !v.is_empty());
            let mut windows_to_auto_close: Vec<AppWindowId> = Vec::new();
            let handled = app.with_global_mut(DockManager::default, |dock, app| {
                let now = app.tick_id();
                app.with_global_mut(DockTearOffMachine::default, |machine, _app| {
                    machine.prune_and_cancel_for_op(now, dock, &op);
                });

                let changed = dock.graph.apply_op(&op);
                if !changed {
                    return false;
                }

                if tearoff_log {
                    match &op {
                        DockOp::MovePanel {
                            source_window,
                            target_window,
                            ..
                        }
                        | DockOp::MoveTabs {
                            source_window,
                            target_window,
                            ..
                        }
                        | DockOp::MergeWindowInto {
                            source_window,
                            target_window,
                            ..
                        } => {
                            let src_panels = dock.graph.collect_panels_in_window(*source_window);
                            let tgt_panels = dock.graph.collect_panels_in_window(*target_window);
                            tracing::info!(
                                op = ?op,
                                source_window = ?source_window,
                                target_window = ?target_window,
                                source_panel_count = src_panels.len(),
                                target_panel_count = tgt_panels.len(),
                                "dock tear-off: applied cross-window dock op"
                            );
                        }
                        _ => {}
                    }
                }

                windows_to_auto_close =
                    auto_close::collect_empty_dock_floating_windows(app, dock, tearoff_log);

                layout_invalidation::invalidate_after_dock_op(app, dock, &op);
                true
            });

            auto_close::close_empty_dock_floating_windows(app, &op, windows_to_auto_close);

            handled
        }
    }
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
