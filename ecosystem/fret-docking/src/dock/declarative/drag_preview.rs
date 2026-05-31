use fret_core::AppWindowId;
use fret_ui::UiHost;

use super::super::diagnostics::dock_drag_ghost_snapshot_for_window;
use super::super::manager::DockManager;
use super::super::paint::DockDragGhostPaint;
use super::super::types::{
    DockDragGhostSnapshot, DockDropTarget, DockPanelDragPayload, DockTabsDragPayload,
};
use super::frame::DockSpaceElementFrame;
use super::tab_metrics::prepare_declarative_tab_title;

pub(super) fn dock_drag_ghost_for_window<H: UiHost>(
    app: &H,
    window: AppWindowId,
) -> Option<DockDragGhostSnapshot> {
    let pointer_id = app.find_drag_pointer_id(|drag| {
        (drag.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
            || drag.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
            && drag.current_window == window
    })?;
    app.drag(pointer_id)
        .and_then(|drag| dock_drag_ghost_snapshot_for_window(drag, window))
}

fn dock_drag_source_tabs_for_window<H: UiHost>(
    app: &H,
    window: AppWindowId,
) -> Option<fret_core::DockNodeId> {
    let pointer_id = app.find_drag_pointer_id(|drag| {
        (drag.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
            || drag.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
            && (drag.source_window == window || drag.current_window == window)
    })?;
    let drag = app.drag(pointer_id)?;
    if let Some(payload) = drag.payload::<DockTabsDragPayload>() {
        return Some(payload.source_tabs);
    }
    let payload = drag.payload::<DockPanelDragPayload>()?;
    app.global::<DockManager>()?
        .graph
        .find_panel_in_window(drag.source_window, &payload.panel)
        .map(|(tabs, _active)| tabs)
}

pub(super) fn drag_ghost_title<H: UiHost>(app: &H, ghost: &DockDragGhostSnapshot) -> String {
    app.global::<DockManager>()
        .and_then(|dock| dock.panel(&ghost.panel).map(|panel| panel.title.as_str()))
        .filter(|title| !title.is_empty())
        .unwrap_or(ghost.panel.kind.0.as_str())
        .to_string()
}

pub(super) fn prepare_declarative_drag_ghost(
    services: &mut dyn fret_core::UiServices,
    ghost: &DockDragGhostSnapshot,
    title: &str,
    scale_factor: f32,
) -> DockDragGhostPaint {
    DockDragGhostPaint {
        position: ghost.position,
        grab_offset: ghost.grab_offset,
        title: prepare_declarative_tab_title(services, title, scale_factor),
    }
}

pub(super) fn declarative_tab_insert_preview_title<H: UiHost>(
    app: &H,
    window: AppWindowId,
    frame: &DockSpaceElementFrame,
) -> Option<(String, Option<fret_core::DockNodeId>, usize)> {
    let Some(DockDropTarget::Dock(target)) = frame.hover.as_ref() else {
        return None;
    };
    if target.zone != fret_core::DropZone::Center {
        return None;
    }
    let ghost = frame.dock_drag_ghost.as_ref()?;
    let drag_source_tabs = dock_drag_source_tabs_for_window(app, window);
    let dock = app.global::<DockManager>()?;
    let title = dock
        .panel(&ghost.panel)
        .map(|panel| panel.title.as_str())
        .filter(|title| !title.is_empty())
        .unwrap_or(ghost.panel.kind.0.as_str())
        .to_string();
    let tab_count = match dock.graph.node(target.tabs) {
        Some(fret_core::DockNode::Tabs { tabs, .. }) => tabs.len(),
        _ => 0,
    };
    Some((title, drag_source_tabs, tab_count))
}
