use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, Model, WindowRequest};
use fret_core::{AppWindowId, Corners, Edges, Event, Px};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitRunnerConfig, WinitWindowContext,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::declarative;
use fret_ui::element::{
    ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
};
use fret_ui::{Invalidation, UiTree};
use fret_ui_kit::OverlayController;
use fret_ui_kit::declarative::form::FormRegistry;
use fret_ui_kit::headless::form_state::{FormState, FormValidateMode};
use fret_ui_kit::headless::form_validation::{first_error, required_trimmed};
use fret_ui_shadcn::button::{Button, ButtonSize, ButtonVariant};
use fret_ui_shadcn::stack;
use fret_ui_shadcn::{DatePicker, Form, FormField, Input, Select, SelectItem, Space};
use std::sync::Arc;
use time::Date;

struct DemoWindowState {
    ui: UiTree<App>,
    form_state: Model<FormState>,
    registry: FormRegistry,
    name: Model<String>,
    email: Model<String>,
    role: Model<Option<Arc<str>>>,
    role_open: Model<bool>,
    start_date: Model<Option<Date>>,
    status: Model<Arc<str>>,
}

#[derive(Default)]
struct FormDemoDriver;

impl FormDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> DemoWindowState {
        let name = app.models_mut().insert(String::new());
        let email = app.models_mut().insert(String::new());
        let role = app.models_mut().insert(None);
        let role_open = app.models_mut().insert(false);
        let start_date = app.models_mut().insert(None::<Date>);

        let mut form_state = FormState::default();
        form_state.validate_mode = FormValidateMode::OnSubmit;
        let form_state = app.models_mut().insert(form_state);

        let status = app.models_mut().insert(Arc::from("Idle"));

        let mut registry = FormRegistry::new();
        registry.register_field("name", name.clone(), String::new(), |v| {
            required_trimmed(v, "Name is required")
        });
        registry.register_field("email", email.clone(), String::new(), |v| {
            first_error([
                required_trimmed(v, "Email is required"),
                (!v.contains('@')).then(|| Arc::from("Email must contain '@'")),
            ])
        });
        registry.register_field("role", role.clone(), None, |v| {
            v.is_none().then(|| Arc::from("Role is required"))
        });
        registry.register_field("start_date", start_date.clone(), None, |v| {
            v.is_none().then(|| Arc::from("Start date is required"))
        });
        registry.register_into_form_state(app, &form_state);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        DemoWindowState {
            ui,
            form_state,
            registry,
            name,
            email,
            role,
            role_open,
            start_date,
            status,
        }
    }
}

impl WinitAppDriver for FormDemoDriver {
    type WindowState = DemoWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn hot_reload_window(
        &mut self,
        app: &mut App,
        _services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        state: &mut Self::WindowState,
    ) {
        crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
    }

    fn handle_model_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[fret_app::ModelId],
    ) {
        context.state.registry.handle_model_changes(
            context.app,
            &context.state.form_state,
            changed,
        );
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

        match command.as_str() {
            "form_demo.close" => {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
            }
            "form_demo.reset" => {
                let _ = app.models_mut().update(&state.name, |v| v.clear());
                let _ = app.models_mut().update(&state.email, |v| v.clear());
                let _ = app.models_mut().update(&state.role, |v| *v = None);
                let _ = app.models_mut().update(&state.role_open, |v| *v = false);
                let _ = app.models_mut().update(&state.start_date, |v| *v = None);
                let _ = app.models_mut().update(&state.form_state, |st| st.reset());
                state
                    .registry
                    .register_into_form_state(app, &state.form_state);
                let _ = app
                    .models_mut()
                    .update(&state.status, |v| *v = Arc::from("Reset"));
                app.request_redraw(window);
            }
            "form_demo.submit" => {
                let ok = state.registry.submit(app, &state.form_state);
                let msg = if ok {
                    "Submitted (valid)"
                } else {
                    "Submitted (errors)"
                };
                let _ = app
                    .models_mut()
                    .update(&state.status, |v| *v = Arc::from(msg));
                app.request_redraw(window);
            }
            _ => {}
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

        if let Event::KeyDown { key, modifiers, .. } = event {
            if !modifiers.ctrl && !modifiers.alt && !modifiers.shift && !modifiers.meta {
                if *key == fret_core::KeyCode::Escape {
                    app.push_effect(Effect::Window(WindowRequest::Close(window)));
                    return;
                }
            }
        }

        state.ui.dispatch_event(app, services, event);
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let scale_factor = context.scale_factor;
        let WinitRenderContext {
            app,
            services,
            window,
            state,
            bounds,
            scene,
            ..
        } = context;

        OverlayController::begin_frame(app, window);

        let name = state.name.clone();
        let email = state.email.clone();
        let role = state.role.clone();
        let role_open = state.role_open.clone();
        let start_date = state.start_date.clone();
        let form_state = state.form_state.clone();
        let status = state.status.clone();
        let root =
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                .render_root("form-demo", move |cx| {
                    cx.observe_model(&form_state, Invalidation::Layout);
                    cx.observe_model(&name, Invalidation::Layout);
                    cx.observe_model(&email, Invalidation::Layout);
                    cx.observe_model(&role, Invalidation::Layout);
                    cx.observe_model(&start_date, Invalidation::Layout);
                    cx.observe_model(&status, Invalidation::Layout);

                    let theme = cx.theme_snapshot();
                    let padding = theme.metric_required("metric.padding.md");

                    let (submit_count, valid, dirty) = cx
                        .app
                        .models()
                        .read(&form_state, |st| {
                            (st.submit_count, st.is_valid(), st.is_dirty())
                        })
                        .unwrap_or((0, true, false));

                    let status_text = cx
                        .app
                        .models()
                        .read(&status, |v| Arc::clone(v))
                        .unwrap_or_else(|_| Arc::from("Idle"));

                    let mut root_layout = LayoutStyle::default();
                    root_layout.size.width = Length::Fill;
                    root_layout.size.height = Length::Fill;

                    let header = stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                            Button::new("Close")
                                .variant(ButtonVariant::Outline)
                                .size(ButtonSize::Sm)
                                .on_click(CommandId::from("form_demo.close"))
                                .into_element(cx),
                            Button::new("Reset")
                                .variant(ButtonVariant::Outline)
                                .size(ButtonSize::Sm)
                                .on_click(CommandId::from("form_demo.reset"))
                                .into_element(cx),
                            Button::new("Submit")
                                .size(ButtonSize::Sm)
                                .on_click(CommandId::from("form_demo.submit"))
                                .into_element(cx),
                            cx.text(Arc::from(format!(
                                "submit_count={submit_count} valid={valid} dirty={dirty} status={}",
                                status_text.as_ref()
                            ))),
                        ]
                        },
                    );

                    let form = {
                        Form::new(vec![
                            FormField::new(
                                form_state.clone(),
                                "name",
                                vec![Input::new(name.clone()).into_element(cx)],
                            )
                            .label("Name")
                            .into_element(cx),
                            FormField::new(
                                form_state.clone(),
                                "email",
                                vec![Input::new(email.clone()).into_element(cx)],
                            )
                            .label("Email")
                            .into_element(cx),
                            FormField::new(
                                form_state.clone(),
                                "role",
                                vec![
                                    Select::new(role.clone(), role_open.clone())
                                        .a11y_label("Role")
                                        .placeholder("Pick a role")
                                        .items([
                                            SelectItem::new("admin", "Admin"),
                                            SelectItem::new("editor", "Editor"),
                                            SelectItem::new("viewer", "Viewer"),
                                        ])
                                        .into_element(cx),
                                ],
                            )
                            .label("Role")
                            .into_element(cx),
                            FormField::new(
                                form_state.clone(),
                                "start_date",
                                vec![
                                    DatePicker::new_controllable(
                                        cx,
                                        Some(start_date.clone()),
                                        None,
                                        None,
                                        false,
                                    )
                                    .placeholder("Pick a start date")
                                    .into_element(cx),
                                ],
                            )
                            .label("Start date")
                            .into_element(cx),
                        ])
                        .into_element(cx)
                    };

                    vec![cx.container(
                        ContainerProps {
                            layout: root_layout,
                            background: Some(theme.color_required("background")),
                            ..Default::default()
                        },
                        move |cx| {
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
                                move |cx| {
                                    vec![
                                        header.clone(),
                                        cx.container(
                                            ContainerProps {
                                                layout: LayoutStyle {
                                                    overflow: Overflow::Clip,
                                                    ..Default::default()
                                                },
                                                border: Edges::all(Px(1.0)),
                                                border_color: Some(theme.color_required("border")),
                                                corner_radii: Corners::all(
                                                    theme.metric_required("metric.radius.md"),
                                                ),
                                                ..Default::default()
                                            },
                                            move |_cx| vec![form.clone()],
                                        ),
                                    ]
                                },
                            )]
                        },
                    )]
                });

        state.ui.set_root(root);
        OverlayController::render(&mut state.ui, app, services, window, bounds);
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
}

pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());

    let config = WinitRunnerConfig {
        main_window_title: "fret-demo form_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(880.0, 560.0),
        ..Default::default()
    };

    fret_kit::run_native_demo(config, app, FormDemoDriver::default()).context("run form_demo app")
}
