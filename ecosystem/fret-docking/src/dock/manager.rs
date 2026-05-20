// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::DockViewportLayout;
use super::prelude_core::*;
use super::prelude_runtime::*;
use super::services::DockFocusRequestService;
use fret_ui::UiHost;

pub struct DockManager {
    pub graph: DockGraph,
    pub panels: HashMap<PanelKey, DockPanel>,
    pub(super) dock_space_nodes: HashMap<fret_core::AppWindowId, NodeId>,
    pub(super) hover: Option<DockDropTarget>,
    pub(super) viewport_layouts:
        HashMap<(fret_core::AppWindowId, RenderTargetId), DockViewportLayout>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ActivatePanelOptions {
    pub focus: bool,
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

impl Default for DockManager {
    fn default() -> Self {
        Self {
            graph: DockGraph::new(),
            panels: HashMap::new(),
            dock_space_nodes: HashMap::new(),
            hover: None,
            viewport_layouts: HashMap::new(),
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

    /// Legacy API: returns the unclipped content rect for a viewport panel.
    ///
    /// Prefer `viewport_layout(...)` (or `viewport_mapping(...)` / `viewport_draw_rect(...)`) for
    /// new code so callers share a single, stable mapping contract.
    pub fn viewport_content_rect(
        &self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
    ) -> Option<Rect> {
        self.viewport_layouts
            .get(&(window, target))
            .map(|layout| layout.content_rect)
    }

    pub fn viewport_draw_rect(
        &self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
    ) -> Option<Rect> {
        self.viewport_layouts
            .get(&(window, target))
            .map(|layout| layout.draw_rect)
    }

    pub fn viewport_mapping(
        &self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
    ) -> Option<ViewportMapping> {
        self.viewport_layouts
            .get(&(window, target))
            .map(|layout| layout.mapping)
    }

    pub fn viewport_layout(
        &self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
    ) -> Option<DockViewportLayout> {
        self.viewport_layouts.get(&(window, target)).copied()
    }

    pub fn clear_viewport_layout_for_window(&mut self, window: fret_core::AppWindowId) {
        self.viewport_layouts.retain(|(w, _), _| *w != window);
    }

    /// Reconciles the viewport layouts observed during a render pass for one window.
    ///
    /// This is intentionally idempotent for identical layout sets. Dock hosts call it from
    /// layout/prepaint code, so repeated frames with unchanged viewport geometry should not churn
    /// cached viewport mappings. Graph/runtime mutations can still use
    /// `clear_viewport_layout_for_window(...)` as an explicit invalidation path.
    pub fn sync_viewport_layouts_for_window(
        &mut self,
        window: fret_core::AppWindowId,
        layouts: impl IntoIterator<Item = (RenderTargetId, DockViewportLayout)>,
    ) -> bool {
        let mut changed = false;
        let mut live_targets = Vec::new();

        for (target, layout) in layouts {
            if !live_targets.contains(&target) {
                live_targets.push(target);
            }
            let key = (window, target);
            match self.viewport_layouts.get_mut(&key) {
                Some(existing) if *existing == layout => {}
                Some(existing) => {
                    *existing = layout;
                    changed = true;
                }
                None => {
                    self.viewport_layouts.insert(key, layout);
                    changed = true;
                }
            }
        }

        let before = self.viewport_layouts.len();
        self.viewport_layouts
            .retain(|(w, target), _| *w != window || live_targets.contains(target));
        changed || self.viewport_layouts.len() != before
    }

    /// Legacy API: records only a content rect and leaves mapping details unspecified.
    ///
    /// Prefer `set_viewport_layout(...)` so the cached entry includes `ViewportMapping` and the
    /// resulting `draw_rect` for consistent hit testing / input forwarding.
    pub fn set_viewport_content_rect(
        &mut self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
        rect: Rect,
    ) {
        self.viewport_layouts.insert(
            (window, target),
            DockViewportLayout {
                content_rect: rect,
                mapping: ViewportMapping {
                    content_rect: rect,
                    target_px_size: (1, 1),
                    fit: ViewportFit::Stretch,
                },
                draw_rect: rect,
            },
        );
    }

    pub fn set_viewport_layout(
        &mut self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
        layout: DockViewportLayout,
    ) {
        self.viewport_layouts.insert((window, target), layout);
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

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::KeyData;

    fn window(raw: u64) -> fret_core::AppWindowId {
        fret_core::AppWindowId::from(KeyData::from_ffi(raw))
    }

    fn target(raw: u64) -> RenderTargetId {
        RenderTargetId::from(KeyData::from_ffi(raw))
    }

    fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(width), Px(height)))
    }

    fn layout(rect: Rect) -> DockViewportLayout {
        let mapping = ViewportMapping {
            content_rect: rect,
            target_px_size: (320, 240),
            fit: ViewportFit::Stretch,
        };
        DockViewportLayout {
            content_rect: rect,
            mapping,
            draw_rect: mapping.map().draw_rect,
        }
    }

    #[test]
    fn sync_viewport_layouts_for_window_is_unchanged_for_identical_layouts() {
        let mut dock = DockManager::default();
        let window = window(1);
        let target = target(1);
        let layout = layout(rect(10.0, 20.0, 300.0, 200.0));

        assert!(dock.sync_viewport_layouts_for_window(window, [(target, layout)]));
        assert_eq!(dock.viewport_layouts.len(), 1);

        assert!(!dock.sync_viewport_layouts_for_window(window, [(target, layout)]));
        assert_eq!(dock.viewport_layouts.len(), 1);
        assert_eq!(dock.viewport_layout(window, target), Some(layout));
    }

    #[test]
    fn sync_viewport_layouts_for_window_removes_stale_entries_for_that_window_only() {
        let mut dock = DockManager::default();
        let window_a = window(1);
        let window_b = window(2);
        let target_a = target(1);
        let target_b = target(2);
        let target_other = target(3);
        let layout_a = layout(rect(0.0, 0.0, 300.0, 200.0));
        let layout_b = layout(rect(300.0, 0.0, 300.0, 200.0));
        let layout_other = layout(rect(0.0, 0.0, 100.0, 100.0));

        assert!(dock.sync_viewport_layouts_for_window(
            window_a,
            [(target_a, layout_a), (target_b, layout_b)]
        ));
        assert!(dock.sync_viewport_layouts_for_window(window_b, [(target_other, layout_other)]));

        assert!(dock.sync_viewport_layouts_for_window(window_a, [(target_b, layout_b)]));

        assert_eq!(dock.viewport_layout(window_a, target_a), None);
        assert_eq!(dock.viewport_layout(window_a, target_b), Some(layout_b));
        assert_eq!(
            dock.viewport_layout(window_b, target_other),
            Some(layout_other)
        );
    }
}
