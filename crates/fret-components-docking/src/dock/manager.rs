// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::*;

pub struct DockManager {
    pub graph: DockGraph,
    pub panels: HashMap<PanelKey, DockPanel>,
    pub(super) dock_space_nodes: HashMap<fret_core::AppWindowId, NodeId>,
    pub(super) hover: Option<DockDropTarget>,
    pub(super) viewport_content_rects: HashMap<(fret_core::AppWindowId, RenderTargetId), Rect>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ActivatePanelOptions {
    pub focus: bool,
}

#[derive(Default)]
pub(super) struct DockFocusRequestService {
    per_window: HashMap<fret_core::AppWindowId, PanelKey>,
}

impl DockFocusRequestService {
    pub(super) fn request(&mut self, window: fret_core::AppWindowId, panel: PanelKey) {
        self.per_window.insert(window, panel);
    }

    pub(super) fn take(&mut self, window: fret_core::AppWindowId) -> Option<PanelKey> {
        self.per_window.remove(&window)
    }
}

impl DockManager {
    pub fn activate_panel_tab_best_effort(
        &self,
        preferred_windows: impl IntoIterator<Item = fret_core::AppWindowId>,
        panel: &PanelKey,
    ) -> Option<(fret_core::AppWindowId, fret_core::DockOp)> {
        let mut preferred: Vec<fret_core::AppWindowId> = Vec::new();
        let mut seen: std::collections::HashSet<fret_core::AppWindowId> =
            std::collections::HashSet::new();
        for w in preferred_windows {
            if seen.insert(w) {
                preferred.push(w);
            }
        }

        for w in &preferred {
            if let Some((tabs, active)) = self.graph.find_panel_in_window(*w, panel) {
                return Some((*w, fret_core::DockOp::SetActiveTab { tabs, active }));
            }
        }

        for w in self.graph.windows() {
            if seen.contains(&w) {
                continue;
            }
            if let Some((tabs, active)) = self.graph.find_panel_in_window(w, panel) {
                return Some((w, fret_core::DockOp::SetActiveTab { tabs, active }));
            }
        }
        None
    }

    pub fn request_activate_panel<H: UiHost>(
        host: &mut H,
        sender: fret_core::AppWindowId,
        preferred_windows: impl IntoIterator<Item = fret_core::AppWindowId>,
        panel: PanelKey,
        options: ActivatePanelOptions,
    ) -> bool {
        let preferred: Vec<fret_core::AppWindowId> = preferred_windows.into_iter().collect();
        let Some((target_window, op)) = host
            .global::<DockManager>()
            .and_then(|dock| dock.activate_panel_tab_best_effort(preferred, &panel))
        else {
            return false;
        };

        host.push_effect(Effect::Dock(op));
        if options.focus {
            host.with_global_mut(DockFocusRequestService::default, |service, _host| {
                service.request(target_window, panel.clone());
            });
            host.push_effect(Effect::Command {
                window: Some(target_window),
                command: CommandId::from("dock.focus_requested_panel"),
            });
        }
        if target_window != sender {
            host.push_effect(Effect::Window(WindowRequest::Raise {
                window: target_window,
                sender: Some(sender),
            }));
        }
        true
    }
}

#[derive(Default)]
pub struct DockPanelContentService {
    per_window: HashMap<fret_core::AppWindowId, HashMap<PanelKey, NodeId>>,
}

impl DockPanelContentService {
    pub fn set(&mut self, window: fret_core::AppWindowId, panel: PanelKey, node: NodeId) {
        self.per_window
            .entry(window)
            .or_default()
            .insert(panel, node);
    }

    pub fn get(&self, window: fret_core::AppWindowId, panel: &PanelKey) -> Option<NodeId> {
        self.per_window
            .get(&window)
            .and_then(|m| m.get(panel))
            .copied()
    }

    pub fn panel_nodes(&self, window: fret_core::AppWindowId) -> Vec<(PanelKey, NodeId)> {
        self.per_window
            .get(&window)
            .map(|m| m.iter().map(|(k, v)| (k.clone(), *v)).collect())
            .unwrap_or_default()
    }
}

impl Default for DockManager {
    fn default() -> Self {
        Self {
            graph: DockGraph::new(),
            panels: HashMap::new(),
            dock_space_nodes: HashMap::new(),
            hover: None,
            viewport_content_rects: HashMap::new(),
        }
    }
}

impl DockManager {
    pub fn dock_space_node(&self, window: fret_core::AppWindowId) -> Option<NodeId> {
        self.dock_space_nodes.get(&window).copied()
    }

    pub fn register_dock_space_node(&mut self, window: fret_core::AppWindowId, node: NodeId) {
        self.dock_space_nodes.insert(window, node);
    }

    pub fn insert_panel(&mut self, key: PanelKey, panel: DockPanel) {
        self.panels.insert(key, panel);
    }

    pub fn ensure_panel(&mut self, key: &PanelKey, make: impl FnOnce() -> DockPanel) {
        self.panels.entry(key.clone()).or_insert_with(make);
    }

    pub fn panel(&self, key: &PanelKey) -> Option<&DockPanel> {
        self.panels.get(key)
    }

    pub fn viewport_content_rect(
        &self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
    ) -> Option<Rect> {
        self.viewport_content_rects.get(&(window, target)).copied()
    }

    pub fn clear_viewport_layout_for_window(&mut self, window: fret_core::AppWindowId) {
        self.viewport_content_rects.retain(|(w, _), _| *w != window);
    }

    pub fn set_viewport_content_rect(
        &mut self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
        rect: Rect,
    ) {
        self.viewport_content_rects.insert((window, target), rect);
    }

    pub fn update_viewport_target_px_size(
        &mut self,
        target: RenderTargetId,
        target_px_size: (u32, u32),
    ) {
        for panel in self.panels.values_mut() {
            let Some(mut vp) = panel.viewport else {
                continue;
            };
            if vp.target != target {
                continue;
            }
            vp.target_px_size = target_px_size;
            panel.viewport = Some(vp);
        }
    }
}
