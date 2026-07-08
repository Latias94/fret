use fret_core::DockOp;
use fret_runtime::{PlatformCapabilities, UiHost};

use crate::DockManager;
use crate::dock::{DockPanelDragPayload, DockTabsDragPayload};

use super::in_window::default_in_window_float_rect;
use super::tear_off::{
    DockTearOffKind, DockTearOffMachine, dock_tear_off_supported, push_dock_floating_window_create,
    queue_dock_floating_window_create,
};

pub(super) fn handle_request_float_to_new_window<H: UiHost>(app: &mut H, op: DockOp) -> bool {
    handle_request_float_to_new_window_with_dispatch(app, op, CreateWindowDispatch::Effect)
}

pub(super) fn queue_request_float_to_new_window<H: UiHost>(app: &mut H, op: DockOp) -> bool {
    handle_request_float_to_new_window_with_dispatch(app, op, CreateWindowDispatch::CommandQueue)
}

#[derive(Clone, Copy)]
enum CreateWindowDispatch {
    Effect,
    CommandQueue,
}

fn handle_request_float_to_new_window_with_dispatch<H: UiHost>(
    app: &mut H,
    op: DockOp,
    dispatch: CreateWindowDispatch,
) -> bool {
    match op {
        DockOp::RequestFloatPanelToNewWindow {
            source_window,
            panel,
            anchor,
        } => {
            if app.global::<DockManager>().is_none() {
                return false;
            }

            if !dock_tear_off_supported(app.global::<PlatformCapabilities>()) {
                let target_window = anchor.map(|a| a.window).unwrap_or(source_window);
                let rect = default_in_window_float_rect(app, target_window, anchor);
                return super::handle_dock_op(
                    app,
                    DockOp::FloatPanelInWindow {
                        source_window,
                        panel,
                        target_window,
                        rect,
                    },
                );
            }

            let now = app.tick_id();
            let pointer_id = app.find_drag_pointer_id(|d| {
                d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL && d.source_window == source_window
            });
            let pointer_id = pointer_id.or_else(|| {
                app.find_drag_pointer_id(|d| {
                    d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                        && d.payload::<DockPanelDragPayload>()
                            .is_some_and(|p| p.panel == panel)
                })
            });
            let should_emit = app.with_global_mut(DockTearOffMachine::default, |machine, _app| {
                machine.register_request(
                    now,
                    source_window,
                    &panel,
                    DockTearOffKind::Panel,
                    pointer_id,
                )
            });
            if !should_emit {
                return true;
            }

            dispatch_dock_floating_window_create(app, dispatch, source_window, panel, anchor);
            true
        }
        DockOp::RequestFloatTabsToNewWindow {
            source_window,
            source_tabs,
            panel,
            anchor,
        } => {
            if app.global::<DockManager>().is_none() {
                return false;
            }

            if !dock_tear_off_supported(app.global::<PlatformCapabilities>()) {
                let target_window = anchor.map(|a| a.window).unwrap_or(source_window);
                let rect = default_in_window_float_rect(app, target_window, anchor);
                return super::handle_dock_op(
                    app,
                    DockOp::FloatTabsInWindow {
                        source_window,
                        source_tabs,
                        target_window,
                        rect,
                    },
                );
            }

            let now = app.tick_id();
            let pointer_id = app.find_drag_pointer_id(|d| {
                d.kind == fret_runtime::DRAG_KIND_DOCK_TABS && d.source_window == source_window
            });
            let pointer_id = pointer_id.or_else(|| {
                app.find_drag_pointer_id(|d| {
                    d.kind == fret_runtime::DRAG_KIND_DOCK_TABS
                        && d.payload::<DockTabsDragPayload>().is_some_and(|p| {
                            p.source_tabs == source_tabs && p.tabs.contains(&panel)
                        })
                })
            });
            let should_emit = app.with_global_mut(DockTearOffMachine::default, |machine, _app| {
                machine.register_request(
                    now,
                    source_window,
                    &panel,
                    DockTearOffKind::Tabs { source_tabs },
                    pointer_id,
                )
            });
            if !should_emit {
                return true;
            }

            dispatch_dock_floating_window_create(app, dispatch, source_window, panel, anchor);
            true
        }
        _ => unreachable!("expected DockFloating request op"),
    }
}

fn dispatch_dock_floating_window_create<H: UiHost>(
    app: &mut H,
    dispatch: CreateWindowDispatch,
    source_window: fret_core::AppWindowId,
    panel: fret_core::PanelKey,
    anchor: Option<fret_core::WindowAnchor>,
) {
    match dispatch {
        CreateWindowDispatch::Effect => {
            push_dock_floating_window_create(app, source_window, panel, anchor);
        }
        CreateWindowDispatch::CommandQueue => {
            queue_dock_floating_window_create(app, source_window, panel, anchor);
        }
    }
}
