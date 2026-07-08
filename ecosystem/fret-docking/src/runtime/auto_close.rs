use fret_core::{AppWindowId, DockOp};
use fret_runtime::UiHost;

use crate::dock::DockManager;

use super::commands::{self, CloseWindowDispatch};
use super::tear_off::DockFloatingOsWindowRegistry;

pub(super) fn collect_empty_dock_floating_windows<H: UiHost>(
    app: &H,
    dock: &DockManager,
    log: bool,
) -> Vec<AppWindowId> {
    let Some(reg) = app.global::<DockFloatingOsWindowRegistry>() else {
        return Vec::new();
    };

    // Close-on-empty is a stable UX expectation in editor-grade docking (ImGui-style).
    // Scan all known dock-owned floating OS windows rather than trying to keep an exhaustive list
    // of which DockOps might have emptied a particular window.
    if log {
        for window in reg.windows() {
            let panel_count = dock.graph.collect_panels_in_window(window).len();
            tracing::info!(
                window = ?window,
                panel_count,
                "dock tear-off: scan dock-floating window panels"
            );
        }
    }

    let mut windows = Vec::new();
    for window in reg.windows() {
        if dock.graph.collect_panels_in_window(window).is_empty() {
            windows.push(window);
        }
    }
    windows
}

pub(super) fn close_empty_dock_floating_windows<H: UiHost>(
    app: &mut H,
    op: &DockOp,
    windows: Vec<AppWindowId>,
    dispatch: CloseWindowDispatch,
) {
    if windows.is_empty() {
        return;
    }

    let log = std::env::var_os("FRET_DOCK_TEAROFF_LOG").is_some_and(|v| !v.is_empty());
    for window in windows {
        if log {
            tracing::info!(
                window = ?window,
                op = ?op,
                "dock tear-off: auto-close empty DockFloating window"
            );
        }
        app.with_global_mut(DockFloatingOsWindowRegistry::default, |reg, _app| {
            reg.remove(window);
        });
        commands::dispatch_close_window(app, dispatch, window);
    }
}
