use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, Model, WindowRequest};
use fret_components_icons::IconRegistry;
use fret_core::{
    AppWindowId, Color, DrawOrder, Edges, Event, KeyCode, PlatformCapabilities, Px, Rect, Scene,
    SceneOp, Size, UiServices,
};
use fret_runner_winit_wgpu::{WindowCreateSpec, WinitDriver, WinitRunner, WinitRunnerConfig};
use fret_ui::primitives::{Column, Scroll, Stack, Text};
use fret_ui::{Invalidation, LayoutCx, PaintCx, UiTree, Widget};
use std::sync::Arc;
use std::time::Duration;
use winit::event_loop::EventLoop;

use fret_components_shadcn as shadcn;

#[derive(Debug, Default)]
struct WindowBackground;

impl<H: fret_ui::UiHost> Widget<H> for WindowBackground {
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
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });
    }
}

struct UiKitWindowState {
    ui: UiTree<App>,
    overlays: shadcn::WindowOverlays,
    root: fret_core::NodeId,

    model_checked: Model<bool>,
    model_enabled: Model<bool>,
    model_slider: Model<f32>,
    model_text: Model<String>,
    model_textarea: Model<String>,
    model_tabs: Model<usize>,
    model_select: Model<usize>,
}

#[derive(Default)]
struct UiKitDriver;

impl UiKitDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> UiKitWindowState {
        let model_checked = app.models_mut().insert(true);
        let model_enabled = app.models_mut().insert(false);
        let model_slider = app.models_mut().insert(0.42f32);
        let model_text = app
            .models_mut()
            .insert("Type here and press Enter…".to_string());
        let model_textarea = app
            .models_mut()
            .insert("Multiline Textarea\n\n- IME\n- Selection\n- Scroll\n".to_string());
        let model_tabs = app.models_mut().insert(0usize);
        let model_select = app.models_mut().insert(1usize);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(Stack::new());
        ui.set_root(root);

        let background = ui.create_node(WindowBackground);
        ui.add_child(root, background);

        let scroll = ui.create_node(Scroll::new().overlay_scrollbar(true));
        ui.add_child(root, scroll);

        let content = ui.create_node(Column::new().with_padding(Px(18.0)).with_spacing(Px(12.0)));
        ui.add_child(scroll, content);

        fn push(
            ui: &mut UiTree<App>,
            parent: fret_core::NodeId,
            widget: impl Widget<App> + 'static,
        ) -> fret_core::NodeId {
            let node = ui.create_node(widget);
            ui.add_child(parent, node);
            node
        }

        let title = ui.create_node(Text::new("shadcn/ui showcase (fret-components-shadcn)"));
        ui.add_child(content, title);

        let subtitle = ui.create_node(Text::new(
            "This is a retained-widget demo: overlays (popover/dialog/toast) are window-scoped layers.",
        ));
        ui.add_child(content, subtitle);

        let _ = push(&mut ui, content, shadcn::Separator::default());

        let _ = push(&mut ui, content, Text::new("Buttons"));
        let _ = push(
            &mut ui,
            content,
            shadcn::Button::new("Default")
                .on_click(CommandId::from("demo.toast"))
                .variant(shadcn::ButtonVariant::Default),
        );
        let _ = push(
            &mut ui,
            content,
            shadcn::Button::new("Secondary")
                .on_click(CommandId::from("demo.toast"))
                .variant(shadcn::ButtonVariant::Secondary),
        );
        let _ = push(
            &mut ui,
            content,
            shadcn::Button::new("Outline")
                .on_click(CommandId::from("demo.toast"))
                .variant(shadcn::ButtonVariant::Outline),
        );
        let _ = push(
            &mut ui,
            content,
            shadcn::Button::new("Destructive")
                .on_click(CommandId::from("demo.dialog.open"))
                .variant(shadcn::ButtonVariant::Destructive),
        );
        let _ = push(
            &mut ui,
            content,
            shadcn::Button::new("Ghost (disabled)")
                .disabled(true)
                .variant(shadcn::ButtonVariant::Ghost),
        );
        let _ = push(
            &mut ui,
            content,
            shadcn::Button::new("Link")
                .on_click(CommandId::from("demo.toast"))
                .variant(shadcn::ButtonVariant::Link),
        );

        let _ = push(&mut ui, content, shadcn::Separator::default());

        let _ = push(&mut ui, content, Text::new("Form Controls"));
        let _ = push(
            &mut ui,
            content,
            shadcn::Input::new(model_text)
                .with_submit_command(CommandId::from("demo.input.submit"))
                .with_cancel_command(CommandId::from("demo.input.cancel")),
        );
        let _ = push(
            &mut ui,
            content,
            shadcn::Textarea::new(model_textarea).with_min_height(Px(120.0)),
        );

        let _ = push(
            &mut ui,
            content,
            shadcn::checkbox::Checkbox::new(model_checked, "Checkbox (Model<bool>)"),
        );
        let _ = push(&mut ui, content, shadcn::switch::Switch::new(model_enabled));

        let _ = push(&mut ui, content, Text::new("Slider + Progress"));
        let _ = push(
            &mut ui,
            content,
            shadcn::slider::Slider::new(model_slider).range(0.0, 1.0),
        );
        let _ = push(
            &mut ui,
            content,
            shadcn::progress::Progress::new(model_slider).range(0.0, 1.0),
        );

        let _ = push(&mut ui, content, shadcn::Separator::default());

        let _ = push(&mut ui, content, Text::new("Tabs"));
        let _ = push(
            &mut ui,
            content,
            shadcn::tabs::Tabs::new(model_tabs, vec!["Account", "Password", "Settings"]),
        );

        let _ = push(&mut ui, content, shadcn::Separator::default());

        let _ = push(&mut ui, content, Text::new("Select (Popover overlay)"));
        let _ = push(
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

        let _ = push(&mut ui, content, shadcn::Separator::default());

        let _ = push(
            &mut ui,
            content,
            shadcn::Button::new("Reset models")
                .on_click(CommandId::from("demo.reset"))
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm),
        );

        let overlays = shadcn::WindowOverlays::install(&mut ui);

        UiKitWindowState {
            ui,
            overlays,
            root,
            model_checked,
            model_enabled,
            model_slider,
            model_text,
            model_textarea,
            model_tabs,
            model_select,
        }
    }

    fn toast(app: &mut App, window: AppWindowId, title: impl Into<Arc<str>>, description: &str) {
        app.with_global_mut(shadcn::toast::ToastService::default, |service, app| {
            service.push(
                app,
                window,
                shadcn::toast::ToastRequest::new(title)
                    .description(description)
                    .kind(shadcn::toast::ToastKind::Success)
                    .duration(Duration::from_secs(3)),
            );
        });
    }
}

impl WinitDriver for UiKitDriver {
    type WindowState = UiKitWindowState;

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
        if state
            .overlays
            .handle_command(app, &mut state.ui, services, window, &command)
        {
            return;
        }

        match command.as_str() {
            "demo.toast" => {
                Self::toast(app, window, "Toast", "Triggered by Button::on_click.");
            }
            "demo.input.submit" => {
                let value = app
                    .models()
                    .get(state.model_text)
                    .cloned()
                    .unwrap_or_default();
                let desc = format!("Submitted: {value}");
                Self::toast(app, window, "Input submit", &desc);
            }
            "demo.input.cancel" => {
                Self::toast(
                    app,
                    window,
                    "Input cancel",
                    "Escape / cancel command fired.",
                );
            }
            "demo.dialog.open" => {
                let owner = state.root;
                app.with_global_mut(shadcn::dialog::DialogService::default, |service, _app| {
                    service.set_request(
                        window,
                        shadcn::dialog::DialogRequest {
                            owner,
                            title: "Dialog".into(),
                            message: "This dialog is rendered by WindowOverlays + DialogOverlay."
                                .into(),
                            actions: vec![
                                shadcn::dialog::DialogAction::new(
                                    "OK",
                                    CommandId::from("demo.dialog.ok"),
                                ),
                                shadcn::dialog::DialogAction::cancel("Cancel"),
                            ],
                            default_action: Some(0),
                            cancel_command: Some(CommandId::from("demo.dialog.cancel")),
                        },
                    );
                });
                let _ = state.overlays.handle_command(
                    app,
                    &mut state.ui,
                    services,
                    window,
                    &CommandId::from("dialog.open"),
                );
            }
            "demo.dialog.ok" => {
                Self::toast(app, window, "Dialog", "OK action selected.");
            }
            "demo.dialog.cancel" => {
                Self::toast(app, window, "Dialog", "Canceled.");
            }
            "demo.reset" => {
                let _ = app.models_mut().update(state.model_checked, |v| *v = true);
                let _ = app.models_mut().update(state.model_enabled, |v| *v = false);
                let _ = app.models_mut().update(state.model_slider, |v| *v = 0.42);
                let _ = app.models_mut().update(state.model_text, |v| v.clear());
                let _ = app.models_mut().update(state.model_textarea, |v| v.clear());
                let _ = app.models_mut().update(state.model_tabs, |v| *v = 0);
                let _ = app.models_mut().update(state.model_select, |v| *v = 1);

                Self::toast(app, window, "Reset", "All demo models were reset.");
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
        main_window_title: "fret-demo ui_kit (shadcn)".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    let driver = UiKitDriver::default();
    let mut runner = WinitRunner::new(config, app, driver);
    event_loop.run_app(&mut runner)?;
    Ok(())
}
