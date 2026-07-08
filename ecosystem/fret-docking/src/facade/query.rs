use fret_core::{AppWindowId, DockGraph, DockNode, DockNodeId, PanelKey};

use crate::dock::DockManager;

use super::{DockSurfacePanelLocation, DockSurfacePanelPlacement, DockSurfacePanelSnapshot};

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
    for window in dock.workspace.graph.windows() {
        if let Some(location) = panel_location_in_window(&dock.workspace.graph, window, panel) {
            return Some(location);
        }
    }
    None
}

pub(super) fn panel_location_in_window(
    graph: &DockGraph,
    window: AppWindowId,
    panel: &PanelKey,
) -> Option<DockSurfacePanelLocation> {
    if let Some(root) = graph.window_root(window)
        && let Some(location) = panel_location_in_node(
            graph,
            window,
            DockSurfacePanelPlacement::Docked,
            root,
            panel,
        )
    {
        return Some(location);
    }

    for floating in graph.floating_windows(window) {
        if let Some(location) = panel_location_in_node(
            graph,
            window,
            DockSurfacePanelPlacement::Floating,
            floating.floating,
            panel,
        ) {
            return Some(location);
        }
    }
    None
}

fn panel_location_in_node(
    graph: &DockGraph,
    window: AppWindowId,
    placement: DockSurfacePanelPlacement,
    node: DockNodeId,
    panel: &PanelKey,
) -> Option<DockSurfacePanelLocation> {
    match graph.node(node)? {
        DockNode::Tabs { tabs, active } => tabs
            .iter()
            .position(|candidate| candidate == panel)
            .map(|tab_index| DockSurfacePanelLocation {
                window,
                placement,
                tab_index,
                tab_count: tabs.len(),
                active: *active == tab_index,
            }),
        DockNode::Split { children, .. } => children
            .iter()
            .copied()
            .find_map(|child| panel_location_in_node(graph, window, placement, child, panel)),
        DockNode::Floating { child } => panel_location_in_node(
            graph,
            window,
            DockSurfacePanelPlacement::Floating,
            *child,
            panel,
        ),
    }
}

pub(super) fn selected_panel_in_window(graph: &DockGraph, window: AppWindowId) -> Option<PanelKey> {
    if let Some(root) = graph.window_root(window)
        && let Some(panel) = selected_panel_in_node(graph, root)
    {
        return Some(panel);
    }
    graph
        .floating_windows(window)
        .iter()
        .find_map(|floating| selected_panel_in_node(graph, floating.floating))
}

fn selected_panel_in_node(graph: &DockGraph, node: DockNodeId) -> Option<PanelKey> {
    match graph.node(node)? {
        DockNode::Tabs { tabs, active } => tabs.get(*active).cloned(),
        DockNode::Split { children, .. } => children
            .iter()
            .copied()
            .find_map(|child| selected_panel_in_node(graph, child)),
        DockNode::Floating { child } => selected_panel_in_node(graph, *child),
    }
}
