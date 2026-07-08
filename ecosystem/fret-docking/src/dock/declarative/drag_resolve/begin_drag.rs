use fret_core::{AppWindowId, Modifiers, Point, PointerId};
use fret_ui::UiHost;

use super::super::super::host_frame::begin_cross_window_dock_drag;
use super::super::super::manager::DockManager;
use super::super::super::types::{DockPanelDragPayload, DockTabsDragPayload};
use super::super::interaction::{DeclarativePendingDockDrag, DeclarativePendingDockTabsDrag};

// This file owns declarative docking cross-window drag-session payload startup.

pub(in crate::dock::declarative) fn begin_declarative_panel_drag<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    pointer_id: PointerId,
    pending: DeclarativePendingDockDrag,
    position: Point,
    modifiers: Modifiers,
) {
    let settings = app
        .global::<fret_runtime::DockingInteractionSettings>()
        .copied()
        .unwrap_or_default();
    let wants_dock_previews = settings.drag_inversion.wants_dock_previews(modifiers);
    let grab_offset = pending.grab_offset;
    begin_cross_window_dock_drag(
        app,
        pointer_id,
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        pending.start,
        position,
        DockPanelDragPayload {
            panel: pending.panel,
            grab_offset,
            tear_off_requested: false,
            tear_off_requested_at_tick: None,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: wants_dock_previews,
        },
        None,
        grab_offset,
    );
}

pub(in crate::dock::declarative) fn begin_declarative_tabs_group_drag<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    pointer_id: PointerId,
    pending: DeclarativePendingDockTabsDrag,
    position: Point,
    modifiers: Modifiers,
) {
    let settings = app
        .global::<fret_runtime::DockingInteractionSettings>()
        .copied()
        .unwrap_or_default();
    let wants_dock_previews = settings.drag_inversion.wants_dock_previews(modifiers);
    let (tabs, active) = app
        .global::<DockManager>()
        .and_then(|dock| match dock.workspace.graph.node(pending.tabs) {
            Some(fret_core::DockNode::Tabs { tabs, active }) => Some((tabs.clone(), *active)),
            _ => None,
        })
        .unwrap_or_else(|| (Vec::new(), 0));
    let grab_offset = pending.grab_offset;
    begin_cross_window_dock_drag(
        app,
        pointer_id,
        fret_runtime::DRAG_KIND_DOCK_TABS,
        window,
        pending.start,
        position,
        DockTabsDragPayload {
            source_tabs: pending.tabs,
            tabs,
            active,
            grab_offset,
            tear_off_requested: false,
            tear_off_requested_at_tick: None,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: wants_dock_previews,
        },
        None,
        grab_offset,
    );
}
