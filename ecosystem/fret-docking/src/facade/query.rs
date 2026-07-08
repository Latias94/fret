use fret_core::{AppWindowId, DockGraph, PanelKey};

use crate::dock::DockManager;

use super::{DockSurfacePanelLocation, DockSurfacePanelSnapshot};

pub(super) fn registered_panel_snapshots(dock: &DockManager) -> Vec<DockSurfacePanelSnapshot> {
    let mut panels: Vec<PanelKey> = dock.workspace.panels().keys().cloned().collect();
    panels.sort_by(|a, b| {
        a.kind
            .0
            .cmp(&b.kind.0)
            .then_with(|| a.instance.cmp(&b.instance))
    });
    panels
        .into_iter()
        .map(|panel| panel_snapshot(dock, panel))
        .collect()
}

pub(super) fn panel_snapshot(dock: &DockManager, panel: PanelKey) -> DockSurfacePanelSnapshot {
    let title = dock
        .workspace
        .panel(&panel)
        .map(|panel| panel.title.clone())
        .unwrap_or_else(|| panel.kind.0.clone());
    let descriptor_only = dock.workspace.panel_catalog().is_descriptor_only(&panel);
    let location = panel_location(dock, &panel);
    DockSurfacePanelSnapshot {
        key: panel,
        title,
        descriptor_only,
        location,
    }
}

pub(super) fn panel_location(
    dock: &DockManager,
    panel: &PanelKey,
) -> Option<DockSurfacePanelLocation> {
    dock.workspace.graph.panel_location(panel).map(Into::into)
}

pub(super) fn panel_location_in_window(
    graph: &DockGraph,
    window: AppWindowId,
    panel: &PanelKey,
) -> Option<DockSurfacePanelLocation> {
    graph
        .panel_location_in_window(window, panel)
        .map(Into::into)
}

pub(super) fn selected_panel_in_window(graph: &DockGraph, window: AppWindowId) -> Option<PanelKey> {
    graph.selected_panel_in_window(window)
}
