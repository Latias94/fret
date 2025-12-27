use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, WindowRequest};
use fret_components_app::tree::AppTreeRowRenderer;
use fret_components_icons::IconRegistry;
use fret_components_ui::tree::{TreeItem, TreeState, TreeViewHandles, create_tree_view};
use fret_core::{
    AppWindowId, Color, Corners, DrawOrder, Edges, Event, KeyCode, PlatformCapabilities, Px, Rect,
    Scene, SceneOp, Size, UiServices,
};
use fret_runner_winit_wgpu::{WindowCreateSpec, WinitDriver, WinitRunner, WinitRunnerConfig};
use fret_ui::declarative;
use fret_ui::element::{ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign};
use fret_ui::{Invalidation, LayoutCx, PaintCx, Theme, UiHost, UiTree, Widget};
use std::sync::Arc;
use winit::event_loop::EventLoop;

#[derive(Debug, Default)]
struct WindowBackground;

impl<H: UiHost> Widget<H> for WindowBackground {
    fn hit_test(&self, _bounds: Rect, _position: fret_core::Point) -> bool {
        false
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        let bg = cx.theme().colors.surface_background;
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: bg,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });
    }
}

struct ComponentsGalleryWindowState {
    ui: UiTree<App>,
    root: fret_core::NodeId,
    tree: TreeViewHandles,
}

#[derive(Default)]
struct ComponentsGalleryDriver;

impl ComponentsGalleryDriver {
    fn sample_tree_items() -> Vec<TreeItem> {
        vec![
            TreeItem::new(1, "src")
                .child(TreeItem::new(2, "components").child(TreeItem::new(3, "tree.rs")))
                .child(TreeItem::new(4, "theme.rs"))
                .child(TreeItem::new(5, "virtual_list.rs")),
            TreeItem::new(10, "crates")
                .child(TreeItem::new(11, "fret-ui"))
                .child(TreeItem::new(12, "fret-components-ui"))
                .child(TreeItem::new(13, "fret-demo").disabled(true)),
            TreeItem::new(20, "docs").child(TreeItem::new(21, "adr")),
        ]
    }

    fn build_ui(app: &mut App, window: AppWindowId) -> ComponentsGalleryWindowState {
        let items = app.models_mut().insert(Self::sample_tree_items());

        let mut initial_state = TreeState::default();
        initial_state.selected = Some(1);
        initial_state.expanded.insert(1);
        initial_state.expanded.insert(10);
        initial_state.expanded.insert(20);
        let state = app.models_mut().insert(initial_state);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(fret_ui::primitives::Stack::new());
        ui.set_root(root);

        let background = ui.create_node(WindowBackground);
        ui.add_child(root, background);

        let tree = create_tree_view(&mut ui, root, items, state);

        ComponentsGalleryWindowState { ui, root, tree }
    }

    fn render_gallery(
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut ComponentsGalleryWindowState,
        bounds: Rect,
    ) {
        let items = state.tree.items;
        let tree_state = state.tree.state;

        let root = declarative::render_root(
            &mut state.ui,
            app,
            services,
            window,
            bounds,
            "components-gallery",
            |cx| {
                let theme = Theme::global(&*cx.app);
                let selected = cx
                    .app
                    .models()
                    .get(tree_state)
                    .map(|s| s.selected)
                    .unwrap_or(None);

                let title: Arc<str> = Arc::from("components_gallery");
                let subtitle: Arc<str> = Arc::from(format!(
                    "Tree MVP: arrows navigate, left/right collapses, click selects. Selected: {}",
                    selected
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| "<none>".to_string())
                ));

                let mut root_layout = LayoutStyle::default();
                root_layout.size.width = Length::Fill;
                root_layout.size.height = Length::Fill;

                let mut tree_slot_layout = LayoutStyle::default();
                tree_slot_layout.size.width = Length::Fill;
                tree_slot_layout.size.height = Length::Fill;
                tree_slot_layout.flex.grow = 1.0;
                tree_slot_layout.flex.basis = Length::Px(Px(0.0));

                let padding = theme.metrics.padding_md;

                vec![cx.flex(
                    FlexProps {
                        layout: root_layout,
                        direction: fret_core::Axis::Vertical,
                        gap: Px(12.0),
                        padding: Edges::all(padding),
                        justify: MainAlign::Start,
                        align: CrossAlign::Stretch,
                        wrap: false,
                    },
                    |cx| {
                        let mut renderer = AppTreeRowRenderer;
                        vec![
                            cx.text(title),
                            cx.text(subtitle),
                            cx.container(
                                ContainerProps {
                                    layout: tree_slot_layout,
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![fret_components_ui::declarative::tree::tree_view_with_renderer(
                                        cx,
                                        items,
                                        tree_state,
                                        fret_components_ui::Size::Medium,
                                        &mut renderer,
                                    )]
                                },
                            ),
                        ]
                    },
                )]
            },
        );

        state.ui.set_children(state.tree.list_mount, vec![root]);
    }
}

impl WinitDriver for ComponentsGalleryDriver {
    type WindowState = ComponentsGalleryWindowState;

    fn init(&mut self, _app: &mut App, _main_window: AppWindowId) {}

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        let mut state = Self::build_ui(app, window);
        state.ui.set_focus(Some(state.tree.tree_root));
        state
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

        if let Some(id) = command.as_str().strip_prefix("gallery.tree.action.") {
            tracing::info!(%id, "gallery tree row action");
            return;
        }

        if let Some(id) = command.as_str().strip_prefix("app.tree.action.") {
            tracing::info!(%id, "app tree row action");
            return;
        }

        match command.as_str() {
            "gallery.close" => {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
            }
            _ => {}
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
        if let Event::KeyDown {
            key: KeyCode::Escape,
            repeat: false,
            ..
        } = event
        {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

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
        ComponentsGalleryDriver::render_gallery(app, services, window, state, bounds);

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
        state.ui.invalidate(state.root, Invalidation::Layout);
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

    let config = WinitRunnerConfig {
        main_window_title: "fret-demo components_gallery".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    let driver = ComponentsGalleryDriver::default();
    let mut runner = WinitRunner::new(config, app, driver);
    event_loop.run_app(&mut runner)?;
    Ok(())
}
