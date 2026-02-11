use super::{DockViewportOverlayHooks, DockingPolicy};
use fret_core::{AppWindowId, NodeId, PanelKey};
use std::collections::HashMap;
use std::sync::Arc;

/// Stores app/editor-owned viewport overlay hooks.
#[derive(Default)]
pub struct DockViewportOverlayHooksService {
    hooks: Option<Arc<dyn DockViewportOverlayHooks>>,
}

impl DockViewportOverlayHooksService {
    pub fn set(&mut self, hooks: Arc<dyn DockViewportOverlayHooks>) {
        self.hooks = Some(hooks);
    }

    pub fn clear(&mut self) {
        self.hooks = None;
    }

    pub fn hooks(&self) -> Option<Arc<dyn DockViewportOverlayHooks>> {
        self.hooks.clone()
    }
}

/// Stores app/editor-owned docking policy hooks (min sizes, drop masks, locks, etc.).
#[derive(Default)]
pub struct DockingPolicyService {
    policy: Option<Arc<dyn DockingPolicy>>,
}

impl DockingPolicyService {
    pub fn set(&mut self, policy: Arc<dyn DockingPolicy>) {
        self.policy = Some(policy);
    }

    pub fn clear(&mut self) {
        self.policy = None;
    }

    pub fn policy(&self) -> Option<Arc<dyn DockingPolicy>> {
        self.policy.clone()
    }
}

#[derive(Default)]
pub struct DockPanelContentService {
    per_window: HashMap<AppWindowId, HashMap<PanelKey, NodeId>>,
}

impl DockPanelContentService {
    pub fn set(&mut self, window: AppWindowId, panel: PanelKey, node: NodeId) {
        self.per_window
            .entry(window)
            .or_default()
            .insert(panel, node);
    }

    pub fn get(&self, window: AppWindowId, panel: &PanelKey) -> Option<NodeId> {
        self.per_window
            .get(&window)
            .and_then(|m| m.get(panel))
            .copied()
    }

    pub fn panel_nodes(&self, window: AppWindowId) -> Vec<(PanelKey, NodeId)> {
        self.per_window
            .get(&window)
            .map(|m| m.iter().map(|(k, v)| (k.clone(), *v)).collect())
            .unwrap_or_default()
    }

    pub fn clear_window(&mut self, window: AppWindowId) {
        self.per_window.remove(&window);
    }

    pub fn replace_window(
        &mut self,
        window: AppWindowId,
        nodes: impl IntoIterator<Item = (PanelKey, NodeId)>,
    ) {
        let mut map: HashMap<PanelKey, NodeId> = HashMap::new();
        for (panel, node) in nodes {
            map.insert(panel, node);
        }
        if map.is_empty() {
            self.per_window.remove(&window);
        } else {
            self.per_window.insert(window, map);
        }
    }
}

#[derive(Default)]
pub(super) struct DockFocusRequestService {
    per_window: HashMap<AppWindowId, PanelKey>,
}

impl DockFocusRequestService {
    pub(super) fn request(&mut self, window: AppWindowId, panel: PanelKey) {
        self.per_window.insert(window, panel);
    }

    pub(super) fn take(&mut self, window: AppWindowId) -> Option<PanelKey> {
        self.per_window.remove(&window)
    }
}
