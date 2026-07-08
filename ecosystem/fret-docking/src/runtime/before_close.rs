use fret_core::{AppWindowId, DockOp};
use fret_runtime::UiHost;

use crate::dock::{DockManager, clear_declarative_dock_interactions_for_window};

use super::commands;
use super::layout_invalidation::invalidate_windows;
use super::tear_off::{DockFloatingOsWindowRegistry, DockTearOffMachine};

pub(super) fn handle_dock_before_close_window<H: UiHost>(
    app: &mut H,
    closing_window: AppWindowId,
    target_window: AppWindowId,
) -> bool {
    if closing_window == target_window {
        return true;
    }
    if app.global::<DockManager>().is_none() {
        return true;
    }

    app.with_global_mut(DockFloatingOsWindowRegistry::default, |reg, _app| {
        reg.remove(closing_window);
    });

    let changed = app.with_global_mut(DockManager::default, |dock, app| {
        if dock.workspace.graph.window_root(closing_window).is_none() {
            return true;
        }
        let op = if let Some(target_tabs) = dock.workspace.graph.first_tabs_in_window(target_window)
        {
            DockOp::MergeWindowInto {
                source_window: closing_window,
                target_window,
                target_tabs,
            }
        } else {
            DockOp::MoveWindowToEmptyDockSpace {
                source_window: closing_window,
                target_window,
            }
        };
        let now = app.tick_id();
        app.with_global_mut(DockTearOffMachine::default, |machine, _app| {
            let canceled_creates = machine.prune_and_cancel_for_op(now, dock, &op);
            commands::remove_queued_create_windows(_app, &canceled_creates);
        });

        let changed = dock.workspace.graph.apply_op(&op);
        if !changed {
            return false;
        }

        dock.clear_transient_state_for_window_transfer(closing_window, target_window);
        invalidate_windows(app, [target_window]);
        true
    });

    if changed {
        clear_declarative_dock_interactions_for_window(app, closing_window);
        clear_declarative_dock_interactions_for_window(app, target_window);
        app.with_global_mut_untracked(
            fret_runtime::WindowInteractionDiagnosticsStore::default,
            |store, _app| {
                store.clear_window(closing_window);
            },
        );
    }

    changed
}
