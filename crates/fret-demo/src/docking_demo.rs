use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, WindowRequest};
use fret_components_docking::dock::DockPanelContentService;
use fret_components_docking::{DockManager, DockPanel};
use fret_components_icons::IconRegistry;
use fret_core::{AppWindowId, Event, PlatformCapabilities, Rect, Scene, UiServices};
use fret_runner_winit_wgpu::{WindowCreateSpec, WinitDriver, WinitRunner, WinitRunnerConfig};
use fret_ui::declarative;
use fret_ui::element::{ContainerProps, LayoutStyle, Length};
use fret_ui::{Invalidation, Theme, UiTree};
use std::sync::Arc;
use winit::event_loop::EventLoop;

struct DockingDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
}

#[derive(Default)]
struct DockingDemoDriver;

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
            let node = fret_components_docking::create_dock_space_node(&mut state.ui, window);
            state.ui.set_root(node);
            node
        });

        let theme = Theme::global(&*app).clone();
        let padding = theme.metrics.padding_md;
        let background = theme.colors.surface_background;

        let hierarchy = declarative::render_root(
            &mut state.ui,
            app,
            services,
            window,
            bounds,
            "dock.panel.hierarchy",
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
                    |cx| vec![cx.text("Hierarchy panel (declarative root)")],
                )]
            },
        );

        let inspector = declarative::render_root(
            &mut state.ui,
            app,
            services,
            window,
            bounds,
            "dock.panel.inspector",
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
                    |cx| vec![cx.text("Inspector panel (declarative root)")],
                )]
            },
        );

        state
            .ui
            .set_children(*dock_space, vec![hierarchy, inspector]);

        app.with_global_mut(DockPanelContentService::default, |svc, _app| {
            svc.set(
                window,
                fret_core::PanelKey::new("core.hierarchy"),
                hierarchy,
            );
            svc.set(
                window,
                fret_core::PanelKey::new("core.inspector"),
                inspector,
            );
        });
    }
}

impl WinitDriver for DockingDemoDriver {
    type WindowState = DockingDemoWindowState;

    fn init(&mut self, _app: &mut App, _main_window: AppWindowId) {}

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn handle_model_changes(
        &mut self,
        app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        changed: &[fret_app::ModelId],
    ) {
        state.ui.propagate_model_changes(app, changed);
    }

    fn handle_command(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut Self::WindowState,
        command: CommandId,
    ) {
        if state.ui.dispatch_command(app, services, &command) {
            return;
        }
        if command.as_str() == "dock_demo.close" {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
        }
    }

    fn handle_event(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut Self::WindowState,
        event: &Event,
    ) {
        if matches!(event, Event::WindowCloseRequested) {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }
        state.ui.dispatch_event(app, services, event);
    }

    fn render(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        state: &mut Self::WindowState,
        bounds: Rect,
        scale_factor: f32,
        services: &mut dyn UiServices,
        scene: &mut Scene,
    ) {
        DockingDemoDriver::render_dock(app, services, window, state, bounds);

        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);
        scene.clear();
        state.ui.layout_all(app, services, bounds, scale_factor);
        state
            .ui
            .paint_all(app, services, bounds, scene, scale_factor);
    }

    fn invalidate_ui_layout(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
    ) {
        if let Some(root) = state.root {
            state.ui.invalidate(root, Invalidation::Layout);
        }
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
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
        _new_window: AppWindowId,
    ) {
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

    let event_loop = EventLoop::new().context("create winit event loop")?;
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    app.with_global_mut(IconRegistry::default, |icons, _app| {
        fret_icons_lucide::register_icons(icons);
    });

    let mut config = WinitRunnerConfig {
        main_window_title: "fret-demo docking_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    if let Some(settings) = fret_app::SettingsFileV1::load_json_if_exists(".fret/settings.json")
        .context("load .fret/settings.json")?
    {
        config.text_font_families.ui_sans = settings.fonts.ui_sans;
        config.text_font_families.ui_serif = settings.fonts.ui_serif;
        config.text_font_families.ui_mono = settings.fonts.ui_mono;
    }

    let driver = DockingDemoDriver::default();
    let mut runner = WinitRunner::new(config, app, driver);
    event_loop.run_app(&mut runner)?;
    Ok(())
}
