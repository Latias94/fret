use fret_core::{AppWindowId, DockOp};
use fret_runtime::UiHost;

use crate::dock::DockManager;
use crate::invalidation::DockInvalidationService;

pub(super) fn invalidate_windows<H: UiHost>(
    app: &mut H,
    windows: impl IntoIterator<Item = AppWindowId>,
) {
    DockInvalidationService::bump_windows(app, windows);
}

pub(super) fn invalidate_after_dock_op<H: UiHost>(
    app: &mut H,
    dock: &mut DockManager,
    op: &DockOp,
) {
    match op {
        DockOp::MovePanel {
            source_window,
            target_window,
            ..
        }
        | DockOp::MovePanelToEmptyDockSpace {
            source_window,
            target_window,
            ..
        }
        | DockOp::MoveTabs {
            source_window,
            target_window,
            ..
        }
        | DockOp::MoveTabsToEmptyDockSpace {
            source_window,
            target_window,
            ..
        }
        | DockOp::MoveWindowToEmptyDockSpace {
            source_window,
            target_window,
        }
        | DockOp::FloatPanelInWindow {
            source_window,
            target_window,
            ..
        }
        | DockOp::FloatTabsInWindow {
            source_window,
            target_window,
            ..
        }
        | DockOp::MergeWindowInto {
            source_window,
            target_window,
            ..
        } => {
            dock.clear_viewport_layout_for_window(*source_window);
            dock.clear_viewport_layout_for_window(*target_window);
            invalidate_windows(app, [*source_window, *target_window]);
        }
        DockOp::FloatPanelToWindow {
            source_window,
            new_window,
            ..
        } => {
            dock.clear_viewport_layout_for_window(*source_window);
            dock.clear_viewport_layout_for_window(*new_window);
            invalidate_windows(app, [*source_window, *new_window]);
        }
        DockOp::SetFloatingRect { window, .. }
        | DockOp::RaiseFloating { window, .. }
        | DockOp::MergeFloatingInto { window, .. }
        | DockOp::ClosePanel { window, .. } => {
            dock.clear_viewport_layout_for_window(*window);
            invalidate_windows(app, [*window]);
        }
        DockOp::EnsurePanelVisible {
            preferred_window,
            panel,
        } => {
            let mut windows = vec![*preferred_window];
            if let Some(location) = dock.workspace.graph.panel_location(panel)
                && !windows.contains(&location.window)
            {
                windows.push(location.window);
            }
            for window in &windows {
                dock.clear_viewport_layout_for_window(*window);
            }
            invalidate_windows(app, windows);
        }
        DockOp::SetActiveTab { .. }
        | DockOp::SetSplitFractions { .. }
        | DockOp::SetSplitFractionsMany { .. } => {
            invalidate_windows(app, dock.workspace.graph.windows());
        }
        _ => {}
    }
}
