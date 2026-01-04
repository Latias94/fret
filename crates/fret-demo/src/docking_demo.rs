use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, WindowRequest};
use fret_core::{
    AppWindowId, Color, Corners, DrawOrder, Edges, Event, Rect, Scene, SceneOp, UiServices,
    geometry::Px,
};
use fret_icons::IconRegistry;
use fret_runner_winit_wgpu::{
    WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitRunnerConfig, WinitWindowContext, run_app,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::declarative;
use fret_ui::element::{ContainerProps, LayoutStyle, Length};
use fret_ui::{Theme, UiTree};
use fret_ui_docking::{
    DockManager, DockPanel, DockPanelRegistry, DockPanelRegistryService, DockViewportOverlayHooks,
    DockViewportOverlayHooksService, handle_dock_before_close_window, handle_dock_op,
    handle_dock_window_created, render_and_bind_dock_panels,
};
use std::sync::Arc;
struct DemoDockPanelRegistry;

impl DockPanelRegistry<App> for DemoDockPanelRegistry {
    fn render_panel(
        &self,
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        panel: &fret_core::PanelKey,
    ) -> Option<fret_core::NodeId> {
        let theme = Theme::global(&*app).clone();
        let padding = theme.metrics.padding_md;
        let background = theme.colors.surface_background;

        let label: &str = match panel.kind.0.as_str() {
            "core.hierarchy" => "Hierarchy panel (declarative root)",
            "core.inspector" => "Inspector panel (declarative root)",
            _ => "Panel (unregistered kind)",
        };

        let root_name = format!("dock_demo.panel.{}", panel.kind.0);
        Some(
            declarative::RenderRootContext::new(ui, app, services, window, bounds).render_root(
                &root_name,
                |cx| {
                    vec![cx.container(
                        ContainerProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Fill;
                                layout
                            },
                            padding: fret_core::Edges::all(padding),
                            background: Some(background),
                            ..Default::default()
                        },
                        |cx| vec![cx.text(label)],
                    )]
                },
            ),
        )
    }
}

struct DemoViewportOverlayHooks;

impl DockViewportOverlayHooks for DemoViewportOverlayHooks {
    fn paint(
        &self,
        theme: fret_ui::ThemeSnapshot,
        _window: AppWindowId,
        _panel: &fret_core::PanelKey,
        _viewport: fret_ui_docking::ViewportPanel,
        _mapping: fret_core::ViewportMapping,
        draw_rect: Rect,
        scene: &mut Scene,
    ) {
        let border_color = Color {
            a: 0.65,
            ..theme.colors.accent
        };
        scene.push(SceneOp::Quad {
            order: DrawOrder(6),
            rect: draw_rect,
            background: Color::TRANSPARENT,
            border: Edges::all(Px(2.0)),
            border_color,
            corner_radii: Corners::all(Px(0.0)),
        });
    }
}

struct DockingDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
}

#[derive(Default)]
struct DockingDemoDriver {
    main_window: Option<AppWindowId>,
}

impl DockingDemoDriver {
    fn build_ui(_app: &mut App, window: AppWindowId) -> DockingDemoWindowState {
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        DockingDemoWindowState { ui, root: None }
    }

    fn ensure_dock_graph(app: &mut App, window: AppWindowId) {
        use fret_core::{Axis, DockNode, PanelKey};

        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.ensure_panel(&PanelKey::new("core.hierarchy"), || DockPanel {
                title: "Hierarchy".to_string(),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });
            dock.ensure_panel(&PanelKey::new("core.inspector"), || DockPanel {
                title: "Inspector".to_string(),
                color: fret_core::Color::TRANSPARENT,
                viewport: None,
            });

            if dock.graph.window_root(window).is_some() {
                return;
            }

            let left = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![PanelKey::new("core.hierarchy")],
                active: 0,
            });
            let right = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![PanelKey::new("core.inspector")],
                active: 0,
            });
            let split = dock.graph.insert_node(DockNode::Split {
                axis: Axis::Horizontal,
                children: vec![left, right],
                fractions: vec![0.5, 0.5],
            });
            dock.graph.set_window_root(window, split);
        });
    }

    fn render_dock(
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut DockingDemoWindowState,
        bounds: Rect,
    ) {
        Self::ensure_dock_graph(app, window);

        let dock_space = state.root.get_or_insert_with(|| {
            let node = fret_ui_docking::create_dock_space_node(&mut state.ui, window);
            state.ui.set_root(node);
            node
        });

        render_and_bind_dock_panels(&mut state.ui, app, services, window, bounds, *dock_space);
    }
}

impl WinitAppDriver for DockingDemoDriver {
    type WindowState = DockingDemoWindowState;

    fn init(&mut self, _app: &mut App, main_window: AppWindowId) {
        self.main_window = Some(main_window);
    }

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn handle_model_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[fret_app::ModelId],
    ) {
        context
            .state
            .ui
            .propagate_model_changes(context.app, changed);
    }

    fn handle_global_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[std::any::TypeId],
    ) {
        context
            .state
            .ui
            .propagate_global_changes(context.app, changed);
    }

    fn handle_command(
        &mut self,
        context: WinitCommandContext<'_, Self::WindowState>,
        command: CommandId,
    ) {
        let WinitCommandContext {
            app,
            services,
            window,
            state,
        } = context;

        if state.ui.dispatch_command(app, services, &command) {
            return;
        }
        if command.as_str() == "dock_demo.close" {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
        }
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
        } = context;
        if matches!(event, Event::WindowCloseRequested) {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }
        state.ui.dispatch_event(app, services, event);
    }

    fn dock_op(&mut self, app: &mut App, op: fret_core::DockOp) {
        let _ = handle_dock_op(app, op);
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let WinitRenderContext {
            app,
            services,
            window,
            state,
            bounds,
            scale_factor,
            scene,
        } = context;

        DockingDemoDriver::render_dock(app, services, window, state, bounds);

        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);
        scene.clear();
        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();
        frame.paint_all(scene);
    }

    fn window_create_spec(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        None
    }

    fn window_created(
        &mut self,
        app: &mut App,
        request: &fret_app::CreateWindowRequest,
        new_window: AppWindowId,
    ) {
        let _ = handle_dock_window_created(app, request, new_window);
    }

    fn before_close_window(&mut self, app: &mut App, window: AppWindowId) -> bool {
        if let Some(main_window) = self.main_window {
            let _ = handle_dock_before_close_window(app, window, main_window);
        }
        true
    }

    fn accessibility_snapshot(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
    ) -> Option<Arc<fret_core::SemanticsSnapshot>> {
        state.ui.semantics_snapshot_arc()
    }

    fn accessibility_focus(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
    ) {
        state.ui.set_focus(Some(target));
    }

    fn accessibility_invoke(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
    ) {
        fret_ui_app::accessibility_actions::invoke(&mut state.ui, app, services, target);
    }

    fn accessibility_set_value_text(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: &str,
    ) {
        fret_ui_app::accessibility_actions::set_value_text(
            &mut state.ui,
            app,
            services,
            target,
            value,
        );
    }

    fn accessibility_set_value_numeric(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: f64,
    ) {
        fret_ui_app::accessibility_actions::set_value_numeric(
            &mut state.ui,
            app,
            services,
            target,
            value,
        );
    }

    fn accessibility_set_text_selection(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        anchor: u32,
        focus: u32,
    ) {
        fret_ui_app::accessibility_actions::set_text_selection(
            &mut state.ui,
            app,
            services,
            target,
            anchor,
            focus,
        );
    }

    fn accessibility_replace_selected_text(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: &str,
    ) {
        fret_ui_app::accessibility_actions::replace_selected_text(
            &mut state.ui,
            app,
            services,
            target,
            value,
        );
    }
}

pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_runner_winit_wgpu=info".parse().unwrap()),
        )
        .try_init();

    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    app.with_global_mut(IconRegistry::default, |icons, _app| {
        fret_icons_lucide::register_icons(icons);
    });
    app.with_global_mut(DockPanelRegistryService::<App>::default, |svc, _app| {
        svc.set(Arc::new(DemoDockPanelRegistry));
    });
    app.with_global_mut(DockViewportOverlayHooksService::default, |svc, _app| {
        svc.set(Arc::new(DemoViewportOverlayHooks));
    });

    let mut config = WinitRunnerConfig {
        main_window_title: "fret-demo docking_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    if let Some(settings) = fret_app::SettingsFileV1::load_json_if_exists(".fret/settings.json")
        .context("load .fret/settings.json")?
    {
        app.set_global(settings.docking_interaction_settings());
        config.text_font_families.ui_sans = settings.fonts.ui_sans;
        config.text_font_families.ui_serif = settings.fonts.ui_serif;
        config.text_font_families.ui_mono = settings.fonts.ui_mono;
    }

    let driver = DockingDemoDriver::default();
    run_app(config, app, driver).map_err(anyhow::Error::from)
}
