use fret_core::AppWindowId;
use fret_runtime::{CreateWindowKind, CreateWindowRequest, UiHost};

use crate::DockManager;

use super::commands::{self, CloseWindowDispatch};
use super::layout_invalidation::invalidate_windows;
use super::tear_off::{
    DockFloatingOsWindowRegistry, DockTearOffCompletion, DockTearOffKind, DockTearOffMachine,
};

pub(super) fn handle_dock_window_created<H: UiHost>(
    app: &mut H,
    request: &CreateWindowRequest,
    new_window: AppWindowId,
) -> bool {
    handle_dock_window_created_with_close_dispatch(
        app,
        request,
        new_window,
        CloseWindowDispatch::Effect,
    )
}

pub(super) fn queue_dock_window_created<H: UiHost>(
    app: &mut H,
    request: &CreateWindowRequest,
    new_window: AppWindowId,
) -> bool {
    handle_dock_window_created_with_close_dispatch(
        app,
        request,
        new_window,
        CloseWindowDispatch::CommandQueue,
    )
}

fn handle_dock_window_created_with_close_dispatch<H: UiHost>(
    app: &mut H,
    request: &CreateWindowRequest,
    new_window: AppWindowId,
    close_dispatch: CloseWindowDispatch,
) -> bool {
    let now = app.tick_id();
    let (completion, pending) = app
        .with_global_mut(DockTearOffMachine::default, |machine, _app| {
            machine.complete_for_create_request(request, now)
        });
    if matches!(completion, DockTearOffCompletion::CancelAndCloseWindow) {
        if std::env::var_os("FRET_DOCK_TEAROFF_LOG").is_some_and(|v| !v.is_empty()) {
            tracing::info!(
                new_window = ?new_window,
                request_kind = ?request.kind,
                "dock tear-off: cancel and close newly created window"
            );
        }
        commands::dispatch_close_window(app, close_dispatch, new_window);
        return true;
    }

    let CreateWindowKind::DockFloating {
        source_window,
        panel,
    } = &request.kind
    else {
        return false;
    };

    if app.global::<DockManager>().is_none() {
        if std::env::var_os("FRET_DOCK_TEAROFF_LOG").is_some_and(|v| !v.is_empty()) {
            tracing::info!(
                new_window = ?new_window,
                request_kind = ?request.kind,
                "dock tear-off: missing DockManager; closing newly created window"
            );
        }
        commands::dispatch_close_window(app, close_dispatch, new_window);
        return true;
    }

    let kind = pending
        .as_ref()
        .map(|p| p.kind)
        .unwrap_or(DockTearOffKind::Panel);
    let handled = app.with_global_mut(DockManager::default, |dock, app| {
        let changed = match kind {
            DockTearOffKind::Panel => {
                dock.graph
                    .float_panel_to_window(*source_window, panel.clone(), new_window)
            }
            DockTearOffKind::Tabs { source_tabs } => {
                dock.graph
                    .float_tabs_to_window(*source_window, source_tabs, new_window)
            }
        };
        if !changed {
            return false;
        }

        let drag_kind = match kind {
            DockTearOffKind::Panel => fret_runtime::DRAG_KIND_DOCK_PANEL,
            DockTearOffKind::Tabs { .. } => fret_runtime::DRAG_KIND_DOCK_TABS,
        };
        let pointer_id_hint = pending.as_ref().and_then(|p| p.pointer_id);
        let pointer_id = pointer_id_hint.or_else(|| {
            app.find_drag_pointer_id(|d| d.kind == drag_kind && d.source_window == *source_window)
        });
        if let Some(pointer_id) = pointer_id
            && let Some(drag) = app.drag_mut(pointer_id)
            && drag.kind == drag_kind
        {
            drag.source_window = new_window;
            drag.current_window = new_window;
        }

        dock.clear_viewport_layout_for_window(*source_window);
        dock.clear_viewport_layout_for_window(new_window);
        invalidate_windows(app, [*source_window, new_window]);
        true
    });

    if handled {
        app.with_global_mut(DockFloatingOsWindowRegistry::default, |reg, _app| {
            reg.register(new_window);
        });
    }

    handled
}
