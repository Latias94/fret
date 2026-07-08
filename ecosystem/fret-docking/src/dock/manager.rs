// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::DockViewportLayout;
use super::prelude_core::*;
use super::prelude_runtime::*;
use super::services::DockFocusRequestService;
use fret_ui::UiHost;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockPanelCatalogError {
    DuplicatePanelKey { key: PanelKey },
}

impl std::fmt::Display for DockPanelCatalogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicatePanelKey { key } => {
                write!(f, "duplicate dock panel key: {:?}", key)
            }
        }
    }
}

impl std::error::Error for DockPanelCatalogError {}

#[derive(Default)]
pub struct DockPanelCatalog {
    panels: HashMap<PanelKey, DockPanel>,
    descriptor_only_panels: HashSet<PanelKey>,
}

impl DockPanelCatalog {
    pub fn register_panel(
        &mut self,
        key: PanelKey,
        panel: DockPanel,
    ) -> Result<(), DockPanelCatalogError> {
        if self.panels.contains_key(&key) {
            return Err(DockPanelCatalogError::DuplicatePanelKey { key });
        }
        self.descriptor_only_panels.remove(&key);
        self.panels.insert(key, panel);
        Ok(())
    }

    pub fn ensure_panel(&mut self, key: &PanelKey, make: impl FnOnce() -> DockPanel) -> bool {
        if self.panels.contains_key(key) {
            return false;
        }
        self.descriptor_only_panels.remove(key);
        self.panels.insert(key.clone(), make());
        true
    }

    pub fn ensure_descriptor_only(&mut self, key: &PanelKey) -> bool {
        if self.panels.contains_key(key) {
            return false;
        }
        self.descriptor_only_panels.insert(key.clone());
        self.panels.insert(
            key.clone(),
            DockPanel {
                title: key.kind.0.clone(),
                color: Color::TRANSPARENT,
                viewport: None,
            },
        );
        true
    }

    pub fn panel(&self, key: &PanelKey) -> Option<&DockPanel> {
        self.panels.get(key)
    }

    pub fn panel_mut(&mut self, key: &PanelKey) -> Option<&mut DockPanel> {
        self.panels.get_mut(key)
    }

    pub fn panels(&self) -> &HashMap<PanelKey, DockPanel> {
        &self.panels
    }

    pub fn descriptor_only_panels(&self) -> impl Iterator<Item = &PanelKey> {
        self.descriptor_only_panels.iter()
    }

    pub fn is_descriptor_only(&self, key: &PanelKey) -> bool {
        self.descriptor_only_panels.contains(key)
    }

    fn panels_mut(&mut self) -> &mut HashMap<PanelKey, DockPanel> {
        &mut self.panels
    }
}

#[derive(Default)]
pub struct DockWorkspace {
    pub graph: DockGraph,
    panel_catalog: DockPanelCatalog,
}

impl DockWorkspace {
    pub fn register_panel(
        &mut self,
        key: PanelKey,
        panel: DockPanel,
    ) -> Result<(), DockPanelCatalogError> {
        self.panel_catalog.register_panel(key, panel)
    }

    pub fn ensure_panel(&mut self, key: &PanelKey, make: impl FnOnce() -> DockPanel) -> bool {
        self.panel_catalog.ensure_panel(key, make)
    }

    pub fn panel(&self, key: &PanelKey) -> Option<&DockPanel> {
        self.panel_catalog.panel(key)
    }

    pub fn panel_mut(&mut self, key: &PanelKey) -> Option<&mut DockPanel> {
        self.panel_catalog.panel_mut(key)
    }

    pub fn panels(&self) -> &HashMap<PanelKey, DockPanel> {
        self.panel_catalog.panels()
    }

    pub fn panel_catalog(&self) -> &DockPanelCatalog {
        &self.panel_catalog
    }

    pub fn import_layout_for_windows_checked(
        &mut self,
        layout: &fret_core::DockLayout,
        windows: &[(fret_core::AppWindowId, String)],
    ) -> Result<bool, fret_core::DockLayoutValidationError> {
        layout.validate()?;
        self.reconcile_panel_descriptors_from_layout(layout);
        self.graph
            .import_layout_for_windows_checked(layout, windows)
    }

    pub fn import_layout_for_windows(
        &mut self,
        layout: &fret_core::DockLayout,
        windows: &[(fret_core::AppWindowId, String)],
    ) -> bool {
        self.import_layout_for_windows_checked(layout, windows)
            .unwrap_or(false)
    }

    pub fn import_layout_for_windows_with_fallback_floatings_checked(
        &mut self,
        layout: &fret_core::DockLayout,
        windows: &[(fret_core::AppWindowId, String)],
        fallback_window: fret_core::AppWindowId,
    ) -> Result<bool, fret_core::DockLayoutValidationError> {
        layout.validate()?;
        self.reconcile_panel_descriptors_from_layout(layout);
        self.graph
            .import_layout_for_windows_with_fallback_floatings_checked(
                layout,
                windows,
                fallback_window,
            )
    }

    pub fn import_layout_for_windows_with_fallback_floatings(
        &mut self,
        layout: &fret_core::DockLayout,
        windows: &[(fret_core::AppWindowId, String)],
        fallback_window: fret_core::AppWindowId,
    ) -> bool {
        self.import_layout_for_windows_with_fallback_floatings_checked(
            layout,
            windows,
            fallback_window,
        )
        .unwrap_or(false)
    }

    fn reconcile_panel_descriptors_from_layout(&mut self, layout: &fret_core::DockLayout) {
        for node in &layout.nodes {
            let fret_core::DockLayoutNode::Tabs { tabs, .. } = node else {
                continue;
            };
            for panel in tabs {
                self.panel_catalog.ensure_descriptor_only(panel);
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct DockPresentationState {
    pub(super) dock_space_nodes: HashMap<fret_core::AppWindowId, NodeId>,
    pub(super) hover: Option<DockDropTarget>,
    pub(super) viewport_layouts:
        HashMap<(fret_core::AppWindowId, RenderTargetId), DockViewportLayout>,
}

#[derive(Default)]
pub struct DockManager {
    pub workspace: DockWorkspace,
    pub(crate) presentation: DockPresentationState,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ActivatePanelOptions {
    pub focus: bool,
}

impl DockManager {
    pub fn workspace(&self) -> &DockWorkspace {
        &self.workspace
    }

    pub fn workspace_mut(&mut self) -> &mut DockWorkspace {
        &mut self.workspace
    }

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
            if let Some((tabs, active)) = self.workspace.graph.find_panel_in_window(*w, panel) {
                return Some((*w, fret_core::DockOp::SetActiveTab { tabs, active }));
            }
        }

        for w in self.workspace.graph.windows() {
            if seen.contains(&w) {
                continue;
            }
            if let Some((tabs, active)) = self.workspace.graph.find_panel_in_window(w, panel) {
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

impl DockManager {
    pub fn dock_space_node(&self, window: fret_core::AppWindowId) -> Option<NodeId> {
        self.presentation.dock_space_nodes.get(&window).copied()
    }

    pub fn register_dock_space_node(&mut self, window: fret_core::AppWindowId, node: NodeId) {
        self.presentation.dock_space_nodes.insert(window, node);
    }

    pub fn register_panel(
        &mut self,
        key: PanelKey,
        panel: DockPanel,
    ) -> Result<(), DockPanelCatalogError> {
        self.workspace.register_panel(key, panel)
    }

    pub fn insert_panel(&mut self, key: PanelKey, panel: DockPanel) {
        self.register_panel(key, panel)
            .expect("dock panel registration should not use duplicate panel keys");
    }

    pub fn ensure_panel(&mut self, key: &PanelKey, make: impl FnOnce() -> DockPanel) {
        self.workspace.ensure_panel(key, make);
    }

    pub fn panel(&self, key: &PanelKey) -> Option<&DockPanel> {
        self.workspace.panel(key)
    }

    pub fn panel_mut(&mut self, key: &PanelKey) -> Option<&mut DockPanel> {
        self.workspace.panel_mut(key)
    }

    pub fn panels(&self) -> &HashMap<PanelKey, DockPanel> {
        self.workspace.panels()
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
        self.presentation
            .viewport_layouts
            .get(&(window, target))
            .map(|layout| layout.content_rect)
    }

    pub fn viewport_draw_rect(
        &self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
    ) -> Option<Rect> {
        self.presentation
            .viewport_layouts
            .get(&(window, target))
            .map(|layout| layout.draw_rect)
    }

    pub fn viewport_mapping(
        &self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
    ) -> Option<ViewportMapping> {
        self.presentation
            .viewport_layouts
            .get(&(window, target))
            .map(|layout| layout.mapping)
    }

    pub fn viewport_layout(
        &self,
        window: fret_core::AppWindowId,
        target: RenderTargetId,
    ) -> Option<DockViewportLayout> {
        self.presentation
            .viewport_layouts
            .get(&(window, target))
            .copied()
    }

    pub fn clear_viewport_layout_for_window(&mut self, window: fret_core::AppWindowId) {
        self.presentation
            .viewport_layouts
            .retain(|(w, _), _| *w != window);
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
            match self.presentation.viewport_layouts.get_mut(&key) {
                Some(existing) if *existing == layout => {}
                Some(existing) => {
                    *existing = layout;
                    changed = true;
                }
                None => {
                    self.presentation.viewport_layouts.insert(key, layout);
                    changed = true;
                }
            }
        }

        let before = self.presentation.viewport_layouts.len();
        self.presentation
            .viewport_layouts
            .retain(|(w, target), _| *w != window || live_targets.contains(target));
        changed || self.presentation.viewport_layouts.len() != before
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
        self.presentation.viewport_layouts.insert(
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
        self.presentation
            .viewport_layouts
            .insert((window, target), layout);
    }

    pub fn update_viewport_target_px_size(
        &mut self,
        target: RenderTargetId,
        target_px_size: (u32, u32),
    ) {
        for panel in self.workspace.panel_catalog.panels_mut().values_mut() {
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

    fn test_panel(title: &str) -> DockPanel {
        DockPanel {
            title: title.to_string(),
            color: Color::TRANSPARENT,
            viewport: None,
        }
    }

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

    fn layout_window(logical_window_id: &str, root: u32) -> fret_core::DockLayoutWindow {
        fret_core::DockLayoutWindow {
            logical_window_id: logical_window_id.to_string(),
            root,
            placement: None,
            floatings: Vec::new(),
        }
    }

    fn tabs_node(id: u32, tabs: Vec<PanelKey>) -> fret_core::DockLayoutNode {
        fret_core::DockLayoutNode::Tabs {
            id,
            tabs,
            active: 0,
        }
    }

    #[test]
    fn panel_catalog_rejects_duplicate_keys() {
        let mut catalog = DockPanelCatalog::default();
        let key = PanelKey::new("test.duplicate");

        assert!(
            catalog
                .register_panel(key.clone(), test_panel("First"))
                .is_ok()
        );
        let err = catalog
            .register_panel(key.clone(), test_panel("Second"))
            .expect_err("duplicate panel keys should fail fast");

        assert_eq!(err, DockPanelCatalogError::DuplicatePanelKey { key });
    }

    #[test]
    fn workspace_import_keeps_unknown_panels_as_descriptor_only_entries() {
        let mut workspace = DockWorkspace::default();
        let window = window(1);
        let known = PanelKey::new("test.known");
        let unknown = PanelKey::new("test.unknown");
        workspace
            .register_panel(known.clone(), test_panel("Known"))
            .expect("known panel registers");

        let layout = fret_core::DockLayout::new(
            vec![layout_window("main", 1)],
            vec![tabs_node(1, vec![known.clone(), unknown.clone()])],
        );

        assert_eq!(
            workspace.import_layout_for_windows_checked(&layout, &[(window, "main".to_string())]),
            Ok(true)
        );

        assert!(!workspace.panel_catalog().is_descriptor_only(&known));
        assert!(workspace.panel_catalog().is_descriptor_only(&unknown));
        let unknown_panel = workspace
            .panel(&unknown)
            .expect("unknown restored panel has descriptor-only catalog entry");
        assert_eq!(unknown_panel.title, unknown.kind.0);
        assert!(unknown_panel.viewport.is_none());
        assert!(
            workspace
                .graph
                .find_panel_in_window(window, &unknown)
                .is_some(),
            "core graph should preserve unknown panel keys during restore"
        );
    }

    #[test]
    fn workspace_import_rejects_invalid_core_layout_before_descriptor_reconcile() {
        let mut workspace = DockWorkspace::default();
        let window = window(1);
        let invalid = PanelKey::new("test.invalid");
        let layout = fret_core::DockLayout::new(
            vec![layout_window("main", 1)],
            vec![fret_core::DockLayoutNode::Tabs {
                id: 1,
                tabs: Vec::new(),
                active: 0,
            }],
        );

        let err = workspace
            .import_layout_for_windows_checked(&layout, &[(window, "main".to_string())])
            .expect_err("empty tabs layout should fail core validation");

        assert!(matches!(
            err.kind,
            fret_core::DockLayoutValidationErrorKind::EmptyTabs { id: 1 }
        ));
        assert!(workspace.graph.window_root(window).is_none());
        assert!(workspace.panel(&invalid).is_none());
    }

    #[test]
    fn workspace_import_degrades_unmapped_windows_into_fallback_floatings() {
        let mut workspace = DockWorkspace::default();
        let main_window = window(1);
        let main_panel = PanelKey::new("test.main");
        let aux_panel = PanelKey::new("test.aux");
        workspace
            .register_panel(main_panel.clone(), test_panel("Main"))
            .expect("main panel registers");

        let layout = fret_core::DockLayout::new(
            vec![layout_window("main", 1), layout_window("aux", 2)],
            vec![
                tabs_node(1, vec![main_panel.clone()]),
                tabs_node(2, vec![aux_panel.clone()]),
            ],
        );

        assert_eq!(
            workspace.import_layout_for_windows_with_fallback_floatings_checked(
                &layout,
                &[(main_window, "main".to_string())],
                main_window,
            ),
            Ok(true)
        );

        assert_eq!(workspace.graph.floating_windows(main_window).len(), 1);
        assert!(workspace.panel_catalog().is_descriptor_only(&aux_panel));
        assert!(
            workspace
                .graph
                .collect_panels_in_window(main_window)
                .contains(&aux_panel),
            "unmapped logical windows should degrade into fallback floatings without dropping panels"
        );
    }

    #[test]
    fn workspace_panel_catalog_survives_graph_close_and_reopen() {
        let mut workspace = DockWorkspace::default();
        let window = window(1);
        let panel = PanelKey::new("test.reopen");
        workspace
            .register_panel(panel.clone(), test_panel("Reopen"))
            .expect("panel registers");
        let tabs = workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        workspace.graph.set_window_root(window, tabs);

        assert!(workspace.graph.close_panel(window, panel.clone()));
        assert_eq!(
            workspace.panel(&panel).map(|panel| panel.title.as_str()),
            Some("Reopen"),
            "closing graph nodes must not remove panel catalog descriptors"
        );

        let tabs = workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        workspace.graph.set_window_root(window, tabs);
        assert!(
            workspace
                .graph
                .find_panel_in_window(window, &panel)
                .is_some()
        );
    }

    #[test]
    fn sync_viewport_layouts_for_window_is_unchanged_for_identical_layouts() {
        let mut dock = DockManager::default();
        let window = window(1);
        let target = target(1);
        let layout = layout(rect(10.0, 20.0, 300.0, 200.0));

        assert!(dock.sync_viewport_layouts_for_window(window, [(target, layout)]));
        assert_eq!(dock.presentation.viewport_layouts.len(), 1);

        assert!(!dock.sync_viewport_layouts_for_window(window, [(target, layout)]));
        assert_eq!(dock.presentation.viewport_layouts.len(), 1);
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
