use anyhow::Context as _;
use fret::advanced;
use fret::advanced::prelude::{LocalState, TrackedStateExt as _};
use fret::advanced::view::{AppUiRenderRootState, render_root_with_app_ui};
use fret_app::{App, CommandId, Effect, WindowRequest};
use fret_core::{AppWindowId, Corners, Edges, Event, Px};
use fret_launch::{
    FnDriver, WindowCreateSpec, WinitCommandContext, WinitEventContext, WinitHotReloadContext,
    WinitRenderContext, WinitRunnerConfig, WinitWindowContext,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
use fret_ui::declarative;
use fret_ui::element::{
    ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
};
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::declarative::form::FormRegistry;
use fret_ui_kit::headless::calendar::CalendarMonth;
use fret_ui_kit::headless::form_state::{FormState, FormValidateMode};
use fret_ui_kit::headless::form_validation::{first_error, required_trimmed};
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, OverlayController, Space};
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;
use time::{Date, OffsetDateTime};

pub struct DemoWindowState {
    ui: UiTree<App>,
    app_ui_root: AppUiRenderRootState,
    form_state: LocalState<FormState>,
    registry: FormRegistry,
    name: LocalState<String>,
    email: LocalState<String>,
    role: LocalState<Option<Arc<str>>>,
    role_open: LocalState<bool>,
    start_date_open: LocalState<bool>,
    start_date_month: LocalState<CalendarMonth>,
    start_date: LocalState<Option<Date>>,
    status: LocalState<Arc<str>>,
}

#[derive(Default)]
pub struct FormDemoDriver;

impl FormDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> DemoWindowState {
        let today = OffsetDateTime::now_utc().date();
        let name = LocalState::new_in(app.models_mut(), String::new());
        let email = LocalState::new_in(app.models_mut(), String::new());
        let role = LocalState::new_in(app.models_mut(), None::<Arc<str>>);
        let role_open = LocalState::new_in(app.models_mut(), false);
        let start_date_open = LocalState::new_in(app.models_mut(), false);
        let start_date_month =
            LocalState::new_in(app.models_mut(), CalendarMonth::from_date(today));
        let start_date = LocalState::new_in(app.models_mut(), None::<Date>);

        let mut form_state = FormState::default();
        form_state.validate_mode = FormValidateMode::OnSubmit;
        let form_state = LocalState::new_in(app.models_mut(), form_state);

        let status = LocalState::new_in(app.models_mut(), Arc::from("Idle"));

        let mut registry = FormRegistry::new();
        registry.register_field("name", &name, String::new(), |v| {
            required_trimmed(v, "Name is required")
        });
        registry.register_field("email", &email, String::new(), |v| {
            first_error([
                required_trimmed(v, "Email is required"),
                (!v.contains('@')).then(|| Arc::from("Email must contain '@'")),
            ])
        });
        registry.register_field("role", &role, None, |v| {
            v.is_none().then(|| Arc::from("Role is required"))
        });
        registry.register_field("start_date", &start_date, None, |v| {
            v.is_none().then(|| Arc::from("Start date is required"))
        });
        registry.register_into_form_state(app, form_state.model());

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        DemoWindowState {
            ui,
            app_ui_root: AppUiRenderRootState::default(),
            form_state,
            registry,
            name,
            email,
            role,
            role_open,
            start_date_open,
            start_date_month,
            start_date,
            status,
        }
    }
}

fn create_window_state(
    _driver: &mut FormDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> DemoWindowState {
    FormDemoDriver::build_ui(app, window)
}

fn hot_reload_window(
    _driver: &mut FormDemoDriver,
    context: WinitHotReloadContext<'_, DemoWindowState>,
) {
    let WinitHotReloadContext {
        app,
        services: _,
        window,
        state,
    } = context;
    crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
}

fn handle_model_changes(
    _driver: &mut FormDemoDriver,
    context: WinitWindowContext<'_, DemoWindowState>,
    changed: &[fret_app::ModelId],
) {
    context.state.registry.handle_model_changes(
        context.app,
        context.state.form_state.model(),
        changed,
    );
    context
        .state
        .ui
        .propagate_model_changes(context.app, changed);
}

fn handle_global_changes(
    _driver: &mut FormDemoDriver,
    context: WinitWindowContext<'_, DemoWindowState>,
    changed: &[std::any::TypeId],
) {
    context
        .state
        .ui
        .propagate_global_changes(context.app, changed);
}

fn handle_command(
    _driver: &mut FormDemoDriver,
    context: WinitCommandContext<'_, DemoWindowState>,
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
            let _ = state
                .name
                .update_in(app.models_mut(), |v: &mut String| v.clear());
            let _ = state
                .email
                .update_in(app.models_mut(), |v: &mut String| v.clear());
            let _ = state.role.set_in(app.models_mut(), None);
            let _ = state.role_open.set_in(app.models_mut(), false);
            let _ = state.start_date_open.set_in(app.models_mut(), false);
            let _ = state.start_date_month.set_in(
                app.models_mut(),
                CalendarMonth::from_date(OffsetDateTime::now_utc().date()),
            );
            let _ = state.start_date.set_in(app.models_mut(), None);
            let _ = state
                .form_state
                .update_in(app.models_mut(), |st: &mut FormState| st.reset());
            state
                .registry
                .register_into_form_state(app, state.form_state.model());
            let _ = state.status.set_in(app.models_mut(), Arc::from("Reset"));
            app.request_redraw(window);
        }
        "form_demo.submit" => {
            let ok = state.registry.submit(app, state.form_state.model());
            let msg = if ok {
                "Submitted (valid)"
            } else {
                "Submitted (errors)"
            };
            let _ = state.status.set_in(app.models_mut(), Arc::from(msg));
            app.request_redraw(window);
        }
        _ => {}
    }
}

fn handle_event(
    _driver: &mut FormDemoDriver,
    context: WinitEventContext<'_, DemoWindowState>,
    event: &Event,
) {
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

fn render(_driver: &mut FormDemoDriver, context: WinitRenderContext<'_, DemoWindowState>) {
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
    let start_date_open = state.start_date_open.clone();
    let start_date_month = state.start_date_month.clone();
    let start_date = state.start_date.clone();
    let form_state = state.form_state.clone();
    let status = state.status.clone();
    let root = render_root_with_app_ui(
        declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds),
        "form-demo",
        &mut state.app_ui_root,
        move |cx| {
            let theme = cx.theme_snapshot();
            let padding = theme.metric_token("metric.padding.md");

            let (submit_count, valid, dirty) = form_state
                .layout(cx)
                .read_ref(|st| (st.submit_count, st.is_valid(), st.is_dirty()))
                .unwrap_or((0, true, false));

            let status_text = status.layout_value(cx);

            let mut root_layout = LayoutStyle::default();
            root_layout.size.width = Length::Fill;
            root_layout.size.height = Length::Fill;
            let cx = cx.elements();

            let header = ui::h_row(|cx| {
                [
                    shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .on_click(CommandId::from("form_demo.close"))
                        .into_element(cx),
                    shadcn::Button::new("Reset")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .on_click(CommandId::from("form_demo.reset"))
                        .into_element(cx),
                    shadcn::Button::new("Submit")
                        .size(shadcn::ButtonSize::Sm)
                        .on_click(CommandId::from("form_demo.submit"))
                        .into_element(cx),
                    cx.text(Arc::from(format!(
                        "submit_count={submit_count} valid={valid} dirty={dirty} status={}",
                        status_text.as_ref()
                    ))),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx);

            let form = {
                shadcn::Form::new(vec![
                    shadcn::FormField::new(
                        &form_state,
                        "name",
                        vec![shadcn::Input::new(&name).into_element(cx)],
                    )
                    .label("Name")
                    .required(true)
                    .into_element(cx),
                    shadcn::FormField::new(
                        &form_state,
                        "email",
                        vec![shadcn::Input::new(&email).into_element(cx)],
                    )
                    .label("Email")
                    .required(true)
                    .into_element(cx),
                    shadcn::FormField::new(
                        &form_state,
                        "role",
                        vec![
                            shadcn::Select::new(&role, &role_open)
                                .a11y_label("Role")
                                .value(shadcn::SelectValue::new().placeholder("Pick a role"))
                                .items([
                                    shadcn::SelectItem::new("admin", "Admin"),
                                    shadcn::SelectItem::new("editor", "Editor"),
                                    shadcn::SelectItem::new("viewer", "Viewer"),
                                ])
                                .into_element(cx),
                        ],
                    )
                    .label("Role")
                    .required(true)
                    .into_element(cx),
                    shadcn::FormField::new(
                        &form_state,
                        "start_date",
                        vec![
                            shadcn::DatePicker::new(
                                &start_date_open,
                                &start_date_month,
                                &start_date,
                            )
                            .placeholder("Pick a start date")
                            .refine_layout(LayoutRefinement::default().w_full())
                            .into_element(cx),
                        ],
                    )
                    .label("Start date")
                    .required(true)
                    .into_element(cx),
                ])
                .into_element(cx)
            };

            vec![cx.container(
                ContainerProps {
                    layout: root_layout,
                    background: Some(theme.color_token("background")),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.flex(
                        FlexProps {
                            layout: root_layout,
                            direction: fret_core::Axis::Vertical,
                            gap: fret_ui::element::SpacingLength::Px(Px(12.0)),
                            padding: Edges::all(padding).into(),
                            justify: MainAlign::Start,
                            align: CrossAlign::Stretch,
                            wrap: false,
                        },
                        move |cx| {
                            vec![
                                header,
                                cx.container(
                                    ContainerProps {
                                        layout: LayoutStyle {
                                            overflow: Overflow::Clip,
                                            ..Default::default()
                                        },
                                        border: Edges::all(Px(1.0)),
                                        border_color: Some(theme.color_token("border")),
                                        corner_radii: Corners::all(
                                            theme.metric_token("metric.radius.md"),
                                        ),
                                        ..Default::default()
                                    },
                                    move |_cx| vec![form],
                                ),
                            ]
                        },
                    )]
                },
            )]
            .into()
        },
    );

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
    _driver: &mut FormDemoDriver,
    _app: &mut App,
    _request: &fret_app::CreateWindowRequest,
) -> Option<WindowCreateSpec> {
    None
}

fn configure_fn_driver_hooks(
    hooks: &mut fret_launch::FnDriverHooks<FormDemoDriver, DemoWindowState>,
) {
    hooks.hot_reload_window = Some(hot_reload_window);
    hooks.handle_model_changes = Some(handle_model_changes);
    hooks.handle_global_changes = Some(handle_global_changes);
    hooks.handle_command = Some(handle_command);
    hooks.window_create_spec = Some(window_create_spec);
}

pub fn build_fn_driver() -> FnDriver<FormDemoDriver, DemoWindowState> {
    FnDriver::new(
        FormDemoDriver::default(),
        create_window_state,
        handle_event,
        render,
    )
    .with_hooks(configure_fn_driver_hooks)
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
        main_window_size: fret_launch::WindowLogicalSize::new(880.0, 560.0),
        ..Default::default()
    };

    advanced::run_native_with_fn_driver_with_hooks(
        config,
        app,
        FormDemoDriver::default(),
        create_window_state,
        handle_event,
        render,
        configure_fn_driver_hooks,
    )
    .context("run form_demo app")
}
