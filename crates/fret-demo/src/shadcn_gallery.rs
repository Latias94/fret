use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, Model, WindowRequest};
use fret_components_icons::IconRegistry;
use fret_core::{
    AppWindowId, Color, Corners, DrawOrder, Edges, Event, KeyCode, Modifiers, Px, Rect, Scene,
    SceneOp, Size, UiServices, geometry::Point,
};
use fret_runner_winit_wgpu::{WindowCreateSpec, WinitDriver, WinitRunner, WinitRunnerConfig};
use fret_ui::{Invalidation, LayoutCx, PaintCx, UiHost, UiTree, Widget};
use std::sync::Arc;
use winit::event_loop::EventLoop;

use fret_components_shadcn as shadcn;
use fret_components_ui::{ContextMenuRequest, ContextMenuService, WindowOverlays};

#[derive(Debug, Default)]
struct WindowBackground;

impl<H: UiHost> Widget<H> for WindowBackground {
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
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

#[derive(Debug, Default)]
struct RootStack;

impl<H: UiHost> Widget<H> for RootStack {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        for &child in cx.children {
            let _ = cx.layout_in(child, cx.bounds);
        }
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for &child in cx.children {
            cx.paint(child, cx.bounds);
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct FlowColumn {
    spacing: Px,
    padding: Px,
}

impl FlowColumn {
    fn new() -> Self {
        Self {
            spacing: Px(0.0),
            padding: Px(0.0),
        }
    }

    fn with_spacing(mut self, spacing: Px) -> Self {
        self.spacing = spacing;
        self
    }

    fn with_padding(mut self, padding: Px) -> Self {
        self.padding = padding;
        self
    }
}

impl Default for FlowColumn {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: UiHost> Widget<H> for FlowColumn {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let pad = self.padding.0.max(0.0);
        let inner_origin = Point::new(
            Px(cx.bounds.origin.x.0 + pad),
            Px(cx.bounds.origin.y.0 + pad),
        );
        let inner_width = Px((cx.available.width.0 - pad * 2.0).max(0.0));

        let mut y = inner_origin.y;
        let mut content_height = Px(0.0);

        for (index, &child) in cx.children.iter().enumerate() {
            if index > 0 {
                let spacing = self.spacing.0.max(0.0);
                y = Px(y.0 + spacing);
                content_height = Px(content_height.0 + spacing);
            }

            let child_origin = Point::new(inner_origin.x, y);
            let probe = Rect::new(child_origin, Size::new(inner_width, Px(1.0e9)));
            let child_size = cx.layout_in(child, probe);
            let final_bounds = Rect::new(child_origin, Size::new(inner_width, child_size.height));
            let _ = cx.layout_in(child, final_bounds);

            y = Px(y.0 + child_size.height.0);
            content_height = Px(content_height.0 + child_size.height.0);
        }

        Size::new(cx.available.width, Px(content_height.0 + pad * 2.0))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for &child in cx.children {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }
    }
}

struct ShadcnGalleryWindowState {
    ui: UiTree<App>,
    overlays: WindowOverlays,
    root: fret_core::NodeId,

    _model_select: Model<usize>,
    _model_combobox_items: Model<Vec<String>>,
    _model_combobox_selection: Model<Option<usize>>,
    _model_combobox_query: Model<String>,
}

#[derive(Default)]
struct ShadcnGalleryDriver;

impl ShadcnGalleryDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> ShadcnGalleryWindowState {
        let model_select = app.models_mut().insert(1usize);
        let model_combobox_items = app.models_mut().insert(vec![
            "Apple".to_string(),
            "Banana".to_string(),
            "Cherry".to_string(),
            "Durian".to_string(),
        ]);
        let model_combobox_selection = app.models_mut().insert(None::<usize>);
        let model_combobox_query = app.models_mut().insert(String::new());

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(RootStack);
        ui.set_root(root);

        let background = ui.create_node(WindowBackground);
        ui.add_child(root, background);

        let overlays = WindowOverlays::install(&mut ui);

        let content = ui.create_node(FlowColumn::new().with_padding(Px(18.0)).with_spacing(Px(12.0)));
        ui.add_child(root, content);

        fn push(ui: &mut UiTree<App>, parent: fret_core::NodeId, widget: impl Widget<App> + 'static) {
            let node = ui.create_node(widget);
            ui.add_child(parent, node);
        }

        push(
            &mut ui,
            content,
            shadcn::Button::new("Open context menu")
                .on_click(CommandId::from("demo.context_menu.open"))
                .variant(shadcn::ButtonVariant::Outline),
        );

        push(
            &mut ui,
            content,
            shadcn::select::Select::new(
                model_select,
                vec![
                    shadcn::select::SelectOption::new("Apple"),
                    shadcn::select::SelectOption::new("Banana"),
                    shadcn::select::SelectOption::new("Cherry"),
                ],
            ),
        );

        push(
            &mut ui,
            content,
            shadcn::combobox::Combobox::new(
                model_combobox_items,
                model_combobox_selection,
                model_combobox_query,
            ),
        );

        ShadcnGalleryWindowState {
            ui,
            overlays,
            root,
            _model_select: model_select,
            _model_combobox_items: model_combobox_items,
            _model_combobox_selection: model_combobox_selection,
            _model_combobox_query: model_combobox_query,
        }
    }
}

impl WinitDriver for ShadcnGalleryDriver {
    type WindowState = ShadcnGalleryWindowState;

    fn init(&mut self, _app: &mut App, _main_window: AppWindowId) {}

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        let mut state = Self::build_ui(app, window);
        state.ui.set_focus(Some(state.root));
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
        if state
            .overlays
            .handle_command(app, &mut state.ui, services, window, &command)
        {
            return;
        }

        if command.as_str() == "demo.context_menu.open" {
            let menu = fret_app::Menu {
                title: Arc::from("Context Menu"),
                items: vec![
                    fret_app::MenuItem::Command {
                        command: CommandId::from("demo.context_menu.action.hello"),
                        when: None,
                    },
                    fret_app::MenuItem::Separator,
                    fret_app::MenuItem::Command {
                        command: CommandId::from("demo.context_menu.action.close"),
                        when: None,
                    },
                ],
            };

            let request = ContextMenuRequest {
                position: Point::new(Px(80.0), Px(80.0)),
                menu,
                input_ctx: fret_app::InputContext::default(),
                menu_bar: None,
            };
            app.with_global_mut(ContextMenuService::default, |service, _app| {
                service.set_request(window, request);
            });
            app.request_redraw(window);
            return;
        }

        if command.as_str() == "demo.context_menu.action.close" {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        if command.as_str().starts_with("demo.context_menu.action.") {
            tracing::info!(command = %command.as_str(), "context menu action");
            return;
        }

        if state.ui.dispatch_command(app, services, &command) {
            return;
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

        if let Event::KeyDown {
            key: KeyCode::Escape,
            repeat: false,
            modifiers: Modifiers { ctrl: true, .. },
        } = event
        {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        state.ui.dispatch_event(app, services, event);
    }

    fn render(
        &mut self,
        app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        bounds: Rect,
        scale_factor: f32,
        services: &mut dyn UiServices,
        scene: &mut Scene,
    ) {
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
    app.set_global(fret_core::PlatformCapabilities::default());
    app.with_global_mut(IconRegistry::default, |icons, _app| {
        fret_icons_lucide::register_icons(icons);
    });

    let config = WinitRunnerConfig {
        main_window_title: "fret-demo shadcn_gallery".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(720.0, 520.0),
        ..Default::default()
    };

    let driver = ShadcnGalleryDriver;
    let mut runner = WinitRunner::new(config, app, driver);
    event_loop.run_app(&mut runner)?;
    Ok(())
}
