use std::sync::Arc;

use fret_core::{
    AppWindowId, DockGraph, DockLayout, DockNodeId, DockOp, DockWindowPlacement, PanelKey,
    WindowAnchor,
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
            .is_some_and(|dock| dock.graph.window_root(window).is_some())
    }

    pub fn ensure_window_root<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
        make_root: impl FnOnce(&mut DockGraph) -> DockNodeId,
    ) -> bool {
        app.with_global_mut(DockManager::default, |dock, _app| {
            if dock.graph.window_root(window).is_some() {
                return false;
            }
            let root = make_root(&mut dock.graph);
            dock.graph.set_window_root(window, root);
            true
        })
    }

    pub fn import_layout_for_windows<H: UiHost>(
        &self,
        app: &mut H,
        layout: &DockLayout,
        windows: &[(AppWindowId, String)],
    ) -> bool {
        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.graph.import_layout_for_windows(layout, windows)
        })
    }

    pub fn import_layout_for_windows_with_fallback_floatings<H: UiHost>(
        &self,
        app: &mut H,
        layout: &DockLayout,
        windows: &[(AppWindowId, String)],
        fallback_window: AppWindowId,
    ) -> bool {
        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.graph
                .import_layout_for_windows_with_fallback_floatings(layout, windows, fallback_window)
        })
    }

    pub fn export_layout<H: UiHost>(
        &self,
        app: &H,
        windows: &[(AppWindowId, String)],
    ) -> Option<DockLayout> {
        app.global::<DockManager>()
            .map(|dock| dock.graph.export_layout(windows))
    }

    pub fn export_layout_with_placement<H: UiHost>(
        &self,
        app: &H,
        windows: &[(AppWindowId, String)],
        placement: impl Fn(AppWindowId) -> Option<DockWindowPlacement>,
    ) -> Option<DockLayout> {
        app.global::<DockManager>()
            .map(|dock| dock.graph.export_layout_with_placement(windows, placement))
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
        crate::runtime::handle_dock_before_close_window(app, closing_window, self.main_window)
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
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.graph.set_window_root(window_a, tabs);
        });

        assert!(surface.request_float_panel_to_new_window(&mut app, window_a, panel.clone(), None));

        assert!(
            app.take_effects().is_empty(),
            "DockSurface command path should not emit Effect::Dock or WindowRequest::Create"
        );
        let commands = surface.take_runtime_commands(&mut app);
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

        assert!(surface.on_window_created(&mut app, &create, window_b));
        let dock = app.global::<DockManager>().expect("dock manager exists");
        assert!(
            dock.graph.find_panel_in_window(window_b, &panel).is_some(),
            "expected panel to move after completing the queued create command"
        );
        assert!(
            dock.graph.find_panel_in_window(window_a, &panel).is_none(),
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
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.graph.set_window_root(window, tabs);
        });

        assert!(surface.request_float_panel_to_new_window(&mut app, window, panel.clone(), None));
        assert!(surface.request_float_panel_to_new_window(&mut app, window, panel, None));

        let commands = surface.take_runtime_commands(&mut app);
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
    fn dock_surface_on_dock_op_request_uses_runtime_command_queue() {
        let window = AppWindowId::from(KeyData::from_ffi(1));
        let panel = PanelKey::new("test.panel");
        let surface = DockSurface::new(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(DockManager::default());
        surface.register_panel(&mut app, panel.clone(), test_panel("Panel"));
        app.with_global_mut(DockManager::default, |dock, _app| {
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.graph.set_window_root(window, tabs);
        });

        assert!(surface.on_dock_op(
            &mut app,
            DockOp::RequestFloatPanelToNewWindow {
                source_window: window,
                panel: panel.clone(),
                anchor: None,
            },
        ));

        assert!(
            app.take_effects().iter().all(|effect| !matches!(
                effect,
                Effect::Dock(_) | Effect::Window(WindowRequest::Create(_))
            )),
            "DockSurface::on_dock_op should route float requests through docking runtime commands"
        );
        assert_eq!(
            surface
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
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.graph.set_window_root(window, tabs);
        });

        assert!(surface.on_dock_op(
            &mut app,
            DockOp::RequestFloatPanelToNewWindow {
                source_window: window,
                panel: panel.clone(),
                anchor: None,
            },
        ));

        assert_eq!(surface.flush_runtime_commands_to_effects(&mut app), 1);
        assert!(
            surface.take_runtime_commands(&mut app).is_empty(),
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
            surface.flush_runtime_commands_to_effects(&mut app),
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
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.graph.set_window_root(window_a, tabs);
        });

        assert!(surface.request_float_panel_to_new_window(&mut app, window_a, panel.clone(), None));
        let commands = surface.take_runtime_commands(&mut app);
        let DockRuntimeCommand::CreateWindow(create) = commands[0].clone() else {
            panic!("expected create-window docking runtime command");
        };
        assert!(surface.on_dock_op(
            &mut app,
            DockOp::MovePanelToEmptyDockSpace {
                source_window: window_a,
                panel: panel.clone(),
                target_window: window_c,
            },
        ));

        assert!(surface.on_window_created(&mut app, &create, window_b));
        assert_eq!(
            surface.take_runtime_commands(&mut app),
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
            dock.graph.find_panel_in_window(window_c, &panel).is_some(),
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
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.graph.set_window_root(window_a, tabs);
        });

        assert!(surface.request_float_panel_to_new_window(&mut app, window_a, panel.clone(), None));
        let commands = surface.take_runtime_commands(&mut app);
        let DockRuntimeCommand::CreateWindow(create) = commands[0].clone() else {
            panic!("expected create-window docking runtime command");
        };
        app.with_global_mut(DockManager::default, |dock, _app| {
            assert!(
                dock.graph.close_panel(window_a, panel.clone()),
                "test setup should remove the source panel without notifying the tear-off machine"
            );
        });

        assert!(surface.on_window_created(&mut app, &create, window_b));
        assert_eq!(
            surface.take_runtime_commands(&mut app),
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
            dock.graph.find_panel_in_window(window_b, &panel).is_none(),
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
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![placeholder.clone(), panel.clone()],
                active: 1,
            });
            dock.graph.set_window_root(window_a, tabs);
        });

        assert!(surface.request_float_panel_to_new_window(&mut app, window_a, panel.clone(), None));
        let commands = surface.take_runtime_commands(&mut app);
        let DockRuntimeCommand::CreateWindow(create) = commands[0].clone() else {
            panic!("expected create-window docking runtime command");
        };
        assert!(surface.on_window_created(&mut app, &create, window_b));
        assert!(
            surface.take_runtime_commands(&mut app).is_empty(),
            "successful window creation should not queue a close command"
        );

        let target_tabs = app
            .global::<DockManager>()
            .expect("dock manager exists")
            .graph
            .first_tabs_in_window(window_a)
            .expect("source window should still have placeholder tabs");

        assert!(surface.on_dock_op(
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
            surface.take_runtime_commands(&mut app),
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
            dock.graph.find_panel_in_window(window_a, &panel).is_some(),
            "redocked panel should return to the target window"
        );
        assert!(
            dock.graph.collect_panels_in_window(window_b).is_empty(),
            "redocking the last panel should empty the dock-floating window"
        );
    }
}
