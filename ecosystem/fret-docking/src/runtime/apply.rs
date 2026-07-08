use fret_core::{AppWindowId, DockOp};
use fret_runtime::UiHost;

use crate::dock::DockManager;

use super::commands::CloseWindowDispatch;
use super::tear_off::DockTearOffMachine;
use super::{auto_close, layout_invalidation};

pub(super) fn handle_applied_dock_op<H: UiHost>(app: &mut H, op: DockOp) -> bool {
    handle_applied_dock_op_with_close_dispatch(app, op, CloseWindowDispatch::Effect)
}

pub(super) fn queue_applied_dock_op<H: UiHost>(app: &mut H, op: DockOp) -> bool {
    handle_applied_dock_op_with_close_dispatch(app, op, CloseWindowDispatch::CommandQueue)
}

fn handle_applied_dock_op_with_close_dispatch<H: UiHost>(
    app: &mut H,
    op: DockOp,
    close_dispatch: CloseWindowDispatch,
) -> bool {
    if app.global::<DockManager>().is_none() {
        return false;
    }

    let tearoff_log = std::env::var_os("FRET_DOCK_TEAROFF_LOG").is_some_and(|v| !v.is_empty());
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
            log_cross_window_op(dock, &op);
        }

        windows_to_auto_close =
            auto_close::collect_empty_dock_floating_windows(app, dock, tearoff_log);

        layout_invalidation::invalidate_after_dock_op(app, dock, &op);
        true
    });

    auto_close::close_empty_dock_floating_windows(app, &op, windows_to_auto_close, close_dispatch);

    handled
}

fn log_cross_window_op(dock: &DockManager, op: &DockOp) {
    match op {
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
        | DockOp::MoveWindowToEmptyDockSpace {
            source_window,
            target_window,
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
