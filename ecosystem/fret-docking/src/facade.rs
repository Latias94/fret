use std::sync::Arc;

use fret_core::{
    AppWindowId, DockGraph, DockLayout, DockLayoutValidationError, DockNode, DockNodeId, DockOp,
    DockWindowPlacement, PanelKey, WindowAnchor,
};
use fret_runtime::{CreateWindowRequest, Effect, UiHost, WindowRequest};
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;

use crate::dock::{
    DockManager, DockPanel, DockPanelElementRegistry, DockPanelElementRegistryService,
    DockSpaceElementOptions, DockViewportOverlayHooks, DockViewportOverlayHooksService,
    DockingPolicy, DockingPolicyService, dock_space_element_from_registry,
};
use crate::runtime::DockRuntimeCommand;

pub type DockHostOptions = DockSpaceElementOptions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockSurfaceChange {
    Changed,
    Unchanged,
}

impl DockSurfaceChange {
    pub fn changed(self) -> bool {
        matches!(self, Self::Changed)
    }
}

impl From<bool> for DockSurfaceChange {
    fn from(changed: bool) -> Self {
        if changed {
            Self::Changed
        } else {
            Self::Unchanged
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockSurfacePanelPlacement {
    Docked,
    Floating,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockSurfacePanelLocation {
    pub window: AppWindowId,
    pub placement: DockSurfacePanelPlacement,
    pub tab_index: usize,
    pub tab_count: usize,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockSurfacePanelSnapshot {
    pub key: PanelKey,
    pub title: String,
    pub descriptor_only: bool,
    pub location: Option<DockSurfacePanelLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockSurfacePanelOutcome {
    pub panel: PanelKey,
    pub change: DockSurfaceChange,
    pub location: Option<DockSurfacePanelLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockSurfacePanelError {
    DockManagerUnavailable,
    PanelNotRegistered { panel: PanelKey },
    PanelNotOpen { panel: PanelKey },
}

#[derive(Debug, Clone)]
pub struct DockSurfaceSnapshot {
    pub layout: DockLayout,
    pub panels: Vec<DockSurfacePanelSnapshot>,
}

/// App-facing docking surface.
///
/// `DockSurface` is the preferred ordinary entry point for applications. It keeps common app code
/// on facade operations while lower-level manager access stays behind explicit advanced modules.
#[derive(Debug, Clone, Copy)]
pub struct DockSurface {
    main_window: AppWindowId,
}

impl DockSurface {
    pub fn new(main_window: AppWindowId) -> Self {
        Self { main_window }
    }

    pub fn main_window(&self) -> AppWindowId {
        self.main_window
    }

    pub fn register_panel<H: UiHost>(&self, app: &mut H, key: PanelKey, panel: DockPanel) {
        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.insert_panel(key, panel);
        });
    }

    pub fn ensure_panel<H: UiHost>(
        &self,
        app: &mut H,
        key: &PanelKey,
        make: impl FnOnce() -> DockPanel,
    ) {
        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.ensure_panel(key, make);
        });
    }

    pub fn has_window_root<H: UiHost>(&self, app: &H, window: AppWindowId) -> bool {
        app.global::<DockManager>()
            .is_some_and(|dock| dock.workspace.graph.window_root(window).is_some())
    }

    pub fn import_layout_for_windows<H: UiHost>(
        &self,
        app: &mut H,
        layout: &DockLayout,
        windows: &[(AppWindowId, String)],
    ) -> bool {
        self.try_import_layout_for_windows(app, layout, windows)
            .unwrap_or(false)
    }

    pub fn try_import_layout_for_windows<H: UiHost>(
        &self,
        app: &mut H,
        layout: &DockLayout,
        windows: &[(AppWindowId, String)],
    ) -> Result<bool, DockLayoutValidationError> {
        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.workspace
                .import_layout_for_windows_checked(layout, windows)
        })
    }

    pub fn import_layout_for_windows_with_fallback_floatings<H: UiHost>(
        &self,
        app: &mut H,
        layout: &DockLayout,
        windows: &[(AppWindowId, String)],
        fallback_window: AppWindowId,
    ) -> bool {
        self.try_import_layout_for_windows_with_fallback_floatings(
            app,
            layout,
            windows,
            fallback_window,
        )
        .unwrap_or(false)
    }

    pub fn try_import_layout_for_windows_with_fallback_floatings<H: UiHost>(
        &self,
        app: &mut H,
        layout: &DockLayout,
        windows: &[(AppWindowId, String)],
        fallback_window: AppWindowId,
    ) -> Result<bool, DockLayoutValidationError> {
        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.workspace
                .import_layout_for_windows_with_fallback_floatings_checked(
                    layout,
                    windows,
                    fallback_window,
                )
        })
    }

    pub fn export_layout<H: UiHost>(
        &self,
        app: &H,
        windows: &[(AppWindowId, String)],
    ) -> Option<DockLayout> {
        app.global::<DockManager>()
            .map(|dock| dock.workspace.graph.export_layout(windows))
    }

    pub fn export_layout_with_placement<H: UiHost>(
        &self,
        app: &H,
        windows: &[(AppWindowId, String)],
        placement: impl Fn(AppWindowId) -> Option<DockWindowPlacement>,
    ) -> Option<DockLayout> {
        app.global::<DockManager>().map(|dock| {
            dock.workspace
                .graph
                .export_layout_with_placement(windows, placement)
        })
    }

    pub fn snapshot<H: UiHost>(
        &self,
        app: &H,
        windows: &[(AppWindowId, String)],
    ) -> Option<DockSurfaceSnapshot> {
        self.snapshot_with_placement(app, windows, |_| None)
    }

    pub fn snapshot_with_placement<H: UiHost>(
        &self,
        app: &H,
        windows: &[(AppWindowId, String)],
        placement: impl FnMut(AppWindowId) -> Option<DockWindowPlacement>,
    ) -> Option<DockSurfaceSnapshot> {
        app.global::<DockManager>().map(|dock| DockSurfaceSnapshot {
            layout: dock
                .workspace
                .graph
                .export_layout_with_placement(windows, placement),
            panels: registered_panel_snapshots(dock),
        })
    }

    pub fn registered_panels<H: UiHost>(&self, app: &H) -> Vec<DockSurfacePanelSnapshot> {
        app.global::<DockManager>()
            .map(registered_panel_snapshots)
            .unwrap_or_default()
    }

    pub fn panels_in_window<H: UiHost>(
        &self,
        app: &H,
        window: AppWindowId,
    ) -> Vec<DockSurfacePanelSnapshot> {
        let Some(dock) = app.global::<DockManager>() else {
            return Vec::new();
        };
        dock.workspace
            .graph
            .collect_panels_in_window(window)
            .into_iter()
            .map(|panel| panel_snapshot(dock, panel))
            .collect()
    }

    pub fn selected_panel_in_window<H: UiHost>(
        &self,
        app: &H,
        window: AppWindowId,
    ) -> Option<PanelKey> {
        let dock = app.global::<DockManager>()?;
        selected_panel_in_window(&dock.workspace.graph, window)
    }

    pub fn panel_location<H: UiHost>(
        &self,
        app: &H,
        panel: &PanelKey,
    ) -> Option<DockSurfacePanelLocation> {
        let dock = app.global::<DockManager>()?;
        panel_location(dock, panel)
    }

    pub fn open_panel<H: UiHost>(
        &self,
        app: &mut H,
        panel: &PanelKey,
    ) -> Result<DockSurfacePanelOutcome, DockSurfacePanelError> {
        self.open_panel_in_window(app, self.main_window, panel)
    }

    pub fn open_panel_in_window<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
        panel: &PanelKey,
    ) -> Result<DockSurfacePanelOutcome, DockSurfacePanelError> {
        let Some(dock) = app.global::<DockManager>() else {
            return Err(DockSurfacePanelError::DockManagerUnavailable);
        };
        if dock.workspace.panel(panel).is_none() {
            return Err(DockSurfacePanelError::PanelNotRegistered {
                panel: panel.clone(),
            });
        }
        if panel_location(dock, panel).is_some() {
            return self.select_panel(app, panel);
        }

        let changed = self.driver().on_dock_op(
            app,
            DockOp::OpenPanel {
                window,
                panel: panel.clone(),
            },
        );
        self.driver().flush_runtime_commands_to_effects(app);
        Ok(self.panel_outcome(app, panel.clone(), changed))
    }

    pub fn select_panel<H: UiHost>(
        &self,
        app: &mut H,
        panel: &PanelKey,
    ) -> Result<DockSurfacePanelOutcome, DockSurfacePanelError> {
        let Some((_, op)) = app
            .global::<DockManager>()
            .and_then(|dock| dock.activate_panel_tab_best_effort([self.main_window], panel))
        else {
            if app.global::<DockManager>().is_none() {
                return Err(DockSurfacePanelError::DockManagerUnavailable);
            }
            return Err(DockSurfacePanelError::PanelNotOpen {
                panel: panel.clone(),
            });
        };

        let changed = self.driver().on_dock_op(app, op);
        self.driver().flush_runtime_commands_to_effects(app);
        Ok(self.panel_outcome(app, panel.clone(), changed))
    }

    pub fn close_panel<H: UiHost>(
        &self,
        app: &mut H,
        panel: &PanelKey,
    ) -> Result<DockSurfacePanelOutcome, DockSurfacePanelError> {
        let Some(location) = self.panel_location(app, panel) else {
            if app.global::<DockManager>().is_none() {
                return Err(DockSurfacePanelError::DockManagerUnavailable);
            }
            return Err(DockSurfacePanelError::PanelNotOpen {
                panel: panel.clone(),
            });
        };

        let changed = self.driver().on_dock_op(
            app,
            DockOp::ClosePanel {
                window: location.window,
                panel: panel.clone(),
            },
        );
        self.driver().flush_runtime_commands_to_effects(app);
        Ok(self.panel_outcome(app, panel.clone(), changed))
    }

    pub fn install_panel_registry<H: UiHost + 'static>(
        &self,
        app: &mut H,
        registry: Arc<dyn DockPanelElementRegistry<H>>,
    ) {
        app.with_global_mut(
            DockPanelElementRegistryService::<H>::default,
            |service, _app| {
                service.set(registry);
            },
        );
    }

    pub fn install_policy<H: UiHost>(&self, app: &mut H, policy: Arc<dyn DockingPolicy>) {
        app.with_global_mut(DockingPolicyService::default, |service, _app| {
            service.set(policy);
        });
    }

    pub fn install_viewport_overlay_hooks<H: UiHost>(
        &self,
        app: &mut H,
        hooks: Arc<dyn DockViewportOverlayHooks>,
    ) {
        app.with_global_mut(DockViewportOverlayHooksService::default, |service, _app| {
            service.set(hooks);
        });
    }

    pub fn host<H>(
        &self,
        cx: &mut ElementContext<'_, H>,
        window: AppWindowId,
        options: DockHostOptions,
    ) -> AnyElement
    where
        H: UiHost + 'static,
    {
        dock_space_element_from_registry(cx, window, options)
    }

    /// Returns the explicit driver-tier API for runtime and host integration callbacks.
    pub fn driver(&self) -> DockSurfaceDriver {
        DockSurfaceDriver { surface: *self }
    }

    fn panel_outcome<H: UiHost>(
        &self,
        app: &H,
        panel: PanelKey,
        changed: bool,
    ) -> DockSurfacePanelOutcome {
        let location = self.panel_location(app, &panel);
        DockSurfacePanelOutcome {
            panel,
            change: DockSurfaceChange::from(changed),
            location,
        }
    }
}

fn registered_panel_snapshots(dock: &DockManager) -> Vec<DockSurfacePanelSnapshot> {
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

fn panel_snapshot(dock: &DockManager, panel: PanelKey) -> DockSurfacePanelSnapshot {
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

fn panel_location(dock: &DockManager, panel: &PanelKey) -> Option<DockSurfacePanelLocation> {
    for window in dock.workspace.graph.windows() {
        if let Some(location) = panel_location_in_window(&dock.workspace.graph, window, panel) {
            return Some(location);
        }
    }
    None
}

fn panel_location_in_window(
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

fn selected_panel_in_window(graph: &DockGraph, window: AppWindowId) -> Option<PanelKey> {
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

/// Explicit host/runtime driver for docking surface integration.
///
/// Ordinary app code should prefer [`DockSurface`] methods. This tier is intentionally separate
/// because it deals in graph construction callbacks, dock operations, runtime commands, and window
/// lifecycle handshakes.
#[derive(Debug, Clone, Copy)]
pub struct DockSurfaceDriver {
    surface: DockSurface,
}

impl DockSurfaceDriver {
    pub fn main_window(&self) -> AppWindowId {
        self.surface.main_window
    }

    pub fn ensure_window_root<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
        make_root: impl FnOnce(&mut DockGraph) -> DockNodeId,
    ) -> bool {
        app.with_global_mut(DockManager::default, |dock, _app| {
            if dock.workspace.graph.window_root(window).is_some() {
                return false;
            }
            let root = make_root(&mut dock.workspace.graph);
            dock.workspace.graph.set_window_root(window, root);
            true
        })
    }

    pub fn request_float_panel_to_new_window<H: UiHost>(
        &self,
        app: &mut H,
        source_window: AppWindowId,
        panel: PanelKey,
        anchor: Option<WindowAnchor>,
    ) -> bool {
        crate::runtime::request_float_panel_to_new_window(app, source_window, panel, anchor)
    }

    pub fn request_float_tabs_to_new_window<H: UiHost>(
        &self,
        app: &mut H,
        source_window: AppWindowId,
        source_tabs: DockNodeId,
        panel: PanelKey,
        anchor: Option<WindowAnchor>,
    ) -> bool {
        crate::runtime::request_float_tabs_to_new_window(
            app,
            source_window,
            source_tabs,
            panel,
            anchor,
        )
    }

    pub fn take_runtime_commands<H: UiHost>(&self, app: &mut H) -> Vec<DockRuntimeCommand> {
        crate::runtime::take_runtime_commands(app)
    }

    pub fn flush_runtime_commands_to_effects<H: UiHost>(&self, app: &mut H) -> usize {
        let commands = self.take_runtime_commands(app);
        let count = commands.len();
        for command in commands {
            match command {
                DockRuntimeCommand::CreateWindow(request) => {
                    app.push_effect(Effect::Window(WindowRequest::Create(request)));
                }
                DockRuntimeCommand::CloseWindow(window) => {
                    app.push_effect(Effect::Window(WindowRequest::Close(window)));
                }
            }
        }
        count
    }

    pub fn on_dock_op<H: UiHost>(&self, app: &mut H, op: DockOp) -> bool {
        crate::runtime::handle_dock_op_with_runtime_commands(app, op)
    }

    pub fn on_window_created<H: UiHost>(
        &self,
        app: &mut H,
        request: &CreateWindowRequest,
        new_window: AppWindowId,
    ) -> bool {
        crate::runtime::complete_queued_window_created(app, request, new_window)
    }

    pub fn before_close_window<H: UiHost>(&self, app: &mut H, closing_window: AppWindowId) -> bool {
        crate::runtime::handle_dock_before_close_window(
            app,
            closing_window,
            self.surface.main_window,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_host::TestHost;
    use fret_core::{DockNode, DropZone, PanelKey};
    use fret_runtime::{CreateWindowKind, Effect, PlatformCapabilities, WindowRequest};
    use slotmap::KeyData;

    fn test_panel(title: &str) -> DockPanel {
        DockPanel {
            title: title.to_string(),
            color: fret_core::Color::TRANSPARENT,
            viewport: None,
        }
    }

    #[test]
    fn dock_surface_panel_commands_return_typed_outcomes() {
        let window = AppWindowId::from(KeyData::from_ffi(1));
        let panel_a = PanelKey::new("test.a");
        let panel_b = PanelKey::new("test.b");
        let surface = DockSurface::new(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(DockManager::default());
        surface.register_panel(&mut app, panel_a.clone(), test_panel("A"));
        surface.register_panel(&mut app, panel_b.clone(), test_panel("B"));

        let opened_a = surface
            .open_panel(&mut app, &panel_a)
            .expect("open panel a");
        assert_eq!(opened_a.panel, panel_a);
        assert_eq!(opened_a.change, DockSurfaceChange::Changed);
        assert_eq!(
            opened_a.location,
            Some(DockSurfacePanelLocation {
                window,
                placement: DockSurfacePanelPlacement::Docked,
                tab_index: 0,
                tab_count: 1,
                active: true,
            })
        );

        let opened_b = surface
            .open_panel(&mut app, &panel_b)
            .expect("open panel b");
        assert_eq!(opened_b.change, DockSurfaceChange::Changed);
        assert_eq!(
            surface.selected_panel_in_window(&app, window),
            Some(panel_b.clone())
        );

        let selected_a = surface
            .select_panel(&mut app, &panel_a)
            .expect("select existing panel");
        assert!(
            selected_a
                .location
                .as_ref()
                .is_some_and(|location| location.active && location.tab_index == 0)
        );
        assert_eq!(
            surface.selected_panel_in_window(&app, window),
            Some(panel_a.clone())
        );

        let closed_a = surface
            .close_panel(&mut app, &panel_a)
            .expect("close selected panel");
        assert_eq!(closed_a.change, DockSurfaceChange::Changed);
        assert_eq!(closed_a.location, None);
        assert_eq!(
            surface.close_panel(&mut app, &panel_a),
            Err(DockSurfacePanelError::PanelNotOpen { panel: panel_a })
        );
    }

    #[test]
    fn dock_surface_registered_panels_include_locations_and_descriptor_flags() {
        let window = AppWindowId::from(KeyData::from_ffi(1));
        let panel_a = PanelKey::new("test.a");
        let panel_b = PanelKey::new("test.b");
        let surface = DockSurface::new(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(DockManager::default());
        surface.register_panel(&mut app, panel_b.clone(), test_panel("B"));
        surface.register_panel(&mut app, panel_a.clone(), test_panel("A"));
        surface
            .open_panel(&mut app, &panel_b)
            .expect("open panel b");

        let panels = surface.registered_panels(&app);
        assert_eq!(
            panels.iter().map(|panel| &panel.key).collect::<Vec<_>>(),
            vec![&panel_a, &panel_b],
            "registered panel snapshots should be stable-sorted by key"
        );
        assert_eq!(panels[0].title, "A");
        assert!(!panels[0].descriptor_only);
        assert_eq!(panels[0].location, None);
        assert!(
            panels[1]
                .location
                .as_ref()
                .is_some_and(|location| location.window == window && location.active)
        );
    }

    #[test]
    fn dock_surface_snapshot_exports_layout_and_panel_facts() {
        let window = AppWindowId::from(KeyData::from_ffi(1));
        let panel = PanelKey::new("test.panel");
        let surface = DockSurface::new(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(DockManager::default());
        surface.register_panel(&mut app, panel.clone(), test_panel("Panel"));
        surface.open_panel(&mut app, &panel).expect("open panel");

        let snapshot = surface
            .snapshot_with_placement(&app, &[(window, "main".to_string())], |candidate| {
                (candidate == window).then_some(DockWindowPlacement {
                    width: 1200,
                    height: 800,
                    x: Some(10),
                    y: Some(20),
                    monitor_hint: Some("primary".to_string()),
                })
            })
            .expect("snapshot");

        assert_eq!(snapshot.layout.windows.len(), 1);
        assert_eq!(snapshot.layout.windows[0].logical_window_id, "main");
        assert!(snapshot.layout.windows[0].placement.is_some());
        assert_eq!(snapshot.panels.len(), 1);
        assert_eq!(snapshot.panels[0].key, panel);
        assert!(snapshot.panels[0].location.is_some());
    }

    #[test]
    fn dock_surface_request_float_panel_uses_runtime_command_queue() {
        let window_a = AppWindowId::from(KeyData::from_ffi(1));
        let window_b = AppWindowId::from(KeyData::from_ffi(2));
        let panel = PanelKey::new("test.panel");
        let surface = DockSurface::new(window_a);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(DockManager::default());
        surface.register_panel(&mut app, panel.clone(), test_panel("Panel"));
        app.with_global_mut(DockManager::default, |dock, _app| {
            let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.workspace.graph.set_window_root(window_a, tabs);
        });

        assert!(surface.driver().request_float_panel_to_new_window(
            &mut app,
            window_a,
            panel.clone(),
            None
        ));

        assert!(
            app.take_effects().is_empty(),
            "DockSurface command path should not emit Effect::Dock or WindowRequest::Create"
        );
        let commands = surface.driver().take_runtime_commands(&mut app);
        assert_eq!(commands.len(), 1);
        let DockRuntimeCommand::CreateWindow(create) = commands[0].clone() else {
            panic!("expected create-window docking runtime command");
        };
        assert!(matches!(
            create.kind,
            CreateWindowKind::DockFloating {
                source_window,
                panel: ref requested,
            }
                if source_window == window_a && requested == &panel
        ));

        assert!(
            surface
                .driver()
                .on_window_created(&mut app, &create, window_b)
        );
        let dock = app.global::<DockManager>().expect("dock manager exists");
        assert!(
            dock.workspace
                .graph
                .find_panel_in_window(window_b, &panel)
                .is_some(),
            "expected panel to move after completing the queued create command"
        );
        assert!(
            dock.workspace
                .graph
                .find_panel_in_window(window_a, &panel)
                .is_none(),
            "expected panel to leave the source window after queued create completion"
        );
    }

    #[test]
    fn dock_surface_runtime_command_queue_deduplicates_until_window_created() {
        let window = AppWindowId::from(KeyData::from_ffi(1));
        let panel = PanelKey::new("test.panel");
        let surface = DockSurface::new(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(DockManager::default());
        surface.register_panel(&mut app, panel.clone(), test_panel("Panel"));
        app.with_global_mut(DockManager::default, |dock, _app| {
            let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.workspace.graph.set_window_root(window, tabs);
        });

        assert!(surface.driver().request_float_panel_to_new_window(
            &mut app,
            window,
            panel.clone(),
            None
        ));
        assert!(
            surface
                .driver()
                .request_float_panel_to_new_window(&mut app, window, panel, None)
        );

        let commands = surface.driver().take_runtime_commands(&mut app);
        assert_eq!(
            commands
                .iter()
                .filter(|command| matches!(command, DockRuntimeCommand::CreateWindow(_)))
                .count(),
            1,
            "DockSurface command queue should preserve runtime request idempotency"
        );
        assert!(
            app.take_effects().iter().all(|effect| !matches!(
                effect,
                Effect::Dock(_) | Effect::Window(WindowRequest::Create(_))
            )),
            "DockSurface command queue should not mirror create requests through Effect::Dock"
        );
    }

    #[test]
    fn dock_surface_request_float_panel_does_not_emit_host_effects_before_flush() {
        let window = AppWindowId::from(KeyData::from_ffi(1));
        let panel = PanelKey::new("test.panel");
        let surface = DockSurface::new(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(DockManager::default());
        surface.register_panel(&mut app, panel.clone(), test_panel("Panel"));
        app.with_global_mut(DockManager::default, |dock, _app| {
            let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.workspace.graph.set_window_root(window, tabs);
        });

        assert!(surface.driver().request_float_panel_to_new_window(
            &mut app,
            window,
            panel.clone(),
            None
        ));

        assert!(
            app.take_effects().iter().all(|effect| !matches!(
                effect,
                Effect::Dock(_) | Effect::Window(WindowRequest::Create(_))
            )),
            "DockSurface request helpers should route float requests through docking runtime commands"
        );
        assert_eq!(
            surface
                .driver()
                .take_runtime_commands(&mut app)
                .iter()
                .filter(|command| matches!(command, DockRuntimeCommand::CreateWindow(_)))
                .count(),
            1
        );
    }

    #[test]
    fn dock_surface_flushes_runtime_commands_to_host_effects() {
        let window = AppWindowId::from(KeyData::from_ffi(1));
        let panel = PanelKey::new("test.panel");
        let surface = DockSurface::new(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(DockManager::default());
        surface.register_panel(&mut app, panel.clone(), test_panel("Panel"));
        app.with_global_mut(DockManager::default, |dock, _app| {
            let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.workspace.graph.set_window_root(window, tabs);
        });

        assert!(surface.driver().request_float_panel_to_new_window(
            &mut app,
            window,
            panel.clone(),
            None
        ));

        assert_eq!(
            surface.driver().flush_runtime_commands_to_effects(&mut app),
            1
        );
        assert!(
            surface.driver().take_runtime_commands(&mut app).is_empty(),
            "flushed commands should be drained from the docking runtime queue"
        );
        let effects = app.take_effects();
        assert_eq!(
            effects
                .iter()
                .filter(|effect| matches!(effect, Effect::Window(WindowRequest::Create(_))))
                .count(),
            1
        );
        let Effect::Window(WindowRequest::Create(create)) = &effects[0] else {
            panic!("expected flushed docking runtime command to become WindowRequest::Create");
        };
        assert!(matches!(
            create.kind,
            CreateWindowKind::DockFloating {
                source_window,
                panel: ref requested,
            }
                if source_window == window && requested == &panel
        ));
        assert_eq!(
            surface.driver().flush_runtime_commands_to_effects(&mut app),
            0,
            "flushing an empty runtime command queue should be a no-op"
        );
    }

    #[test]
    fn dock_surface_window_created_for_stale_source_queues_close_command() {
        let window_a = AppWindowId::from(KeyData::from_ffi(1));
        let window_b = AppWindowId::from(KeyData::from_ffi(2));
        let window_c = AppWindowId::from(KeyData::from_ffi(3));
        let panel = PanelKey::new("test.panel");
        let surface = DockSurface::new(window_a);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(DockManager::default());
        surface.register_panel(&mut app, panel.clone(), test_panel("Panel"));
        app.with_global_mut(DockManager::default, |dock, _app| {
            let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.workspace.graph.set_window_root(window_a, tabs);
        });

        assert!(surface.driver().request_float_panel_to_new_window(
            &mut app,
            window_a,
            panel.clone(),
            None
        ));
        let commands = surface.driver().take_runtime_commands(&mut app);
        let DockRuntimeCommand::CreateWindow(create) = commands[0].clone() else {
            panic!("expected create-window docking runtime command");
        };
        assert!(surface.driver().on_dock_op(
            &mut app,
            DockOp::MovePanelToEmptyDockSpace {
                source_window: window_a,
                panel: panel.clone(),
                target_window: window_c,
            },
        ));

        assert!(
            surface
                .driver()
                .on_window_created(&mut app, &create, window_b)
        );
        assert_eq!(
            surface.driver().take_runtime_commands(&mut app),
            vec![DockRuntimeCommand::CloseWindow(window_b)]
        );
        assert!(
            app.take_effects()
                .iter()
                .all(|effect| !matches!(effect, Effect::Window(WindowRequest::Close(_)))),
            "DockSurface stale create cleanup should stay on the docking command queue"
        );
        let dock = app.global::<DockManager>().expect("dock manager exists");
        assert!(
            dock.workspace
                .graph
                .find_panel_in_window(window_c, &panel)
                .is_some(),
            "stale queued window completion must preserve the current graph owner"
        );
    }

    #[test]
    fn dock_surface_window_created_graph_commit_failure_queues_close_command() {
        let window_a = AppWindowId::from(KeyData::from_ffi(1));
        let window_b = AppWindowId::from(KeyData::from_ffi(2));
        let panel = PanelKey::new("test.panel");
        let surface = DockSurface::new(window_a);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(DockManager::default());
        surface.register_panel(&mut app, panel.clone(), test_panel("Panel"));
        app.with_global_mut(DockManager::default, |dock, _app| {
            let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.workspace.graph.set_window_root(window_a, tabs);
        });

        assert!(surface.driver().request_float_panel_to_new_window(
            &mut app,
            window_a,
            panel.clone(),
            None
        ));
        let commands = surface.driver().take_runtime_commands(&mut app);
        let DockRuntimeCommand::CreateWindow(create) = commands[0].clone() else {
            panic!("expected create-window docking runtime command");
        };
        app.with_global_mut(DockManager::default, |dock, _app| {
            assert!(
                dock.workspace.graph.close_panel(window_a, panel.clone()),
                "test setup should remove the source panel without notifying the tear-off machine"
            );
        });

        assert!(
            surface
                .driver()
                .on_window_created(&mut app, &create, window_b)
        );
        assert_eq!(
            surface.driver().take_runtime_commands(&mut app),
            vec![DockRuntimeCommand::CloseWindow(window_b)]
        );
        assert!(
            app.take_effects()
                .iter()
                .all(|effect| !matches!(effect, Effect::Window(WindowRequest::Close(_)))),
            "DockSurface graph commit failure cleanup should stay on the docking command queue"
        );
        let dock = app.global::<DockManager>().expect("dock manager exists");
        assert!(
            dock.workspace
                .graph
                .find_panel_in_window(window_b, &panel)
                .is_none(),
            "failed graph commit must not invent the panel in the new window"
        );
    }

    #[test]
    fn dock_surface_redock_auto_close_uses_runtime_command_queue() {
        let window_a = AppWindowId::from(KeyData::from_ffi(1));
        let window_b = AppWindowId::from(KeyData::from_ffi(2));
        let panel = PanelKey::new("test.panel");
        let placeholder = PanelKey::new("test.placeholder");
        let surface = DockSurface::new(window_a);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(DockManager::default());
        surface.register_panel(&mut app, panel.clone(), test_panel("Panel"));
        surface.register_panel(&mut app, placeholder.clone(), test_panel("Placeholder"));
        app.with_global_mut(DockManager::default, |dock, _app| {
            let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
                tabs: vec![placeholder.clone(), panel.clone()],
                active: 1,
            });
            dock.workspace.graph.set_window_root(window_a, tabs);
        });

        assert!(surface.driver().request_float_panel_to_new_window(
            &mut app,
            window_a,
            panel.clone(),
            None
        ));
        let commands = surface.driver().take_runtime_commands(&mut app);
        let DockRuntimeCommand::CreateWindow(create) = commands[0].clone() else {
            panic!("expected create-window docking runtime command");
        };
        assert!(
            surface
                .driver()
                .on_window_created(&mut app, &create, window_b)
        );
        assert!(
            surface.driver().take_runtime_commands(&mut app).is_empty(),
            "successful window creation should not queue a close command"
        );

        let target_tabs = app
            .global::<DockManager>()
            .expect("dock manager exists")
            .workspace
            .graph
            .first_tabs_in_window(window_a)
            .expect("source window should still have placeholder tabs");

        assert!(surface.driver().on_dock_op(
            &mut app,
            DockOp::MovePanel {
                source_window: window_b,
                panel: panel.clone(),
                target_window: window_a,
                target_tabs,
                zone: DropZone::Center,
                insert_index: None,
            },
        ));

        assert_eq!(
            surface.driver().take_runtime_commands(&mut app),
            vec![DockRuntimeCommand::CloseWindow(window_b)]
        );
        assert!(
            app.take_effects()
                .iter()
                .all(|effect| !matches!(effect, Effect::Window(WindowRequest::Close(_)))),
            "DockSurface auto-close should stay on the docking command queue"
        );
        let dock = app.global::<DockManager>().expect("dock manager exists");
        assert!(
            dock.workspace
                .graph
                .find_panel_in_window(window_a, &panel)
                .is_some(),
            "redocked panel should return to the target window"
        );
        assert!(
            dock.workspace
                .graph
                .collect_panels_in_window(window_b)
                .is_empty(),
            "redocking the last panel should empty the dock-floating window"
        );
    }
}
