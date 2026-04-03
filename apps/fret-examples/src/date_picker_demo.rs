use anyhow::Context as _;
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
use fret_ui_kit::headless::calendar::CalendarMonth;
use fret_ui_kit::ui;
use fret_ui_kit::{OverlayController, Space};
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;
use time::{OffsetDateTime, Weekday};

pub struct DemoWindowState {
    ui: UiTree<App>,
    app_ui_root: AppUiRenderRootState,
    locals: Option<DatePickerDemoLocals>,
}

#[derive(Clone)]
struct DatePickerDemoLocals {
    open: LocalState<bool>,
    month: LocalState<CalendarMonth>,
    selected: LocalState<Option<time::Date>>,
    week_start_monday: LocalState<bool>,
    show_outside_days: LocalState<bool>,
    disable_outside_days: LocalState<bool>,
    disable_weekends: LocalState<bool>,
    disabled: LocalState<bool>,
}

impl DatePickerDemoLocals {
    fn new(cx: &mut fret::AppUi<'_, '_>) -> Self {
        let today = OffsetDateTime::now_utc().date();
        Self {
            open: cx.state().local_init(|| false),
            selected: cx.state().local_init(|| None::<time::Date>),
            month: cx.state().local_init(|| CalendarMonth::from_date(today)),
            week_start_monday: cx.state().local_init(|| true),
            show_outside_days: cx.state().local_init(|| true),
            disable_outside_days: cx.state().local_init(|| true),
            disable_weekends: cx.state().local_init(|| false),
            disabled: cx.state().local_init(|| false),
        }
    }
}

#[derive(Default)]
pub struct DatePickerDemoDriver;

impl DatePickerDemoDriver {
    fn build_ui(_app: &mut App, window: AppWindowId) -> DemoWindowState {
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        DemoWindowState {
            ui,
            app_ui_root: AppUiRenderRootState::default(),
            locals: None,
        }
    }
}

fn create_window_state(
    _driver: &mut DatePickerDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> DemoWindowState {
    DatePickerDemoDriver::build_ui(app, window)
}

fn hot_reload_window(
    _driver: &mut DatePickerDemoDriver,
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
    _driver: &mut DatePickerDemoDriver,
    context: WinitWindowContext<'_, DemoWindowState>,
    changed: &[fret_app::ModelId],
) {
    context
        .state
        .ui
        .propagate_model_changes(context.app, changed);
}

fn handle_global_changes(
    _driver: &mut DatePickerDemoDriver,
    context: WinitWindowContext<'_, DemoWindowState>,
    changed: &[std::any::TypeId],
) {
    context
        .state
        .ui
        .propagate_global_changes(context.app, changed);
}

fn handle_command(
    _driver: &mut DatePickerDemoDriver,
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
        "date_picker_demo.close" => {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
        }
        "date_picker_demo.reset_today" => {
            let Some(locals) = state.locals.as_ref() else {
                return;
            };
            let today = OffsetDateTime::now_utc().date();
            let _ = locals.selected.set_in(app.models_mut(), Some(today));
            let _ = locals
                .month
                .set_in(app.models_mut(), CalendarMonth::from_date(today));
            app.request_redraw(window);
        }
        "date_picker_demo.clear" => {
            let Some(locals) = state.locals.as_ref() else {
                return;
            };
            let _ = locals.selected.set_in(app.models_mut(), None);
            app.request_redraw(window);
        }
        _ => {}
    }
}

fn handle_event(
    _driver: &mut DatePickerDemoDriver,
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

fn render(_driver: &mut DatePickerDemoDriver, context: WinitRenderContext<'_, DemoWindowState>) {
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
    let locals = &mut state.locals;

    let root = render_root_with_app_ui(
        declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds),
        "date-picker-demo",
        &mut state.app_ui_root,
        move |cx| {
            if locals.is_none() {
                *locals = Some(DatePickerDemoLocals::new(cx));
            }
            let DatePickerDemoLocals {
                open,
                month,
                selected,
                week_start_monday,
                show_outside_days,
                disable_outside_days,
                disable_weekends,
                disabled,
            } = locals
                .as_ref()
                .expect("date picker locals should exist")
                .clone();
            let theme = cx.theme_snapshot();
            let padding = theme.metric_token("metric.padding.md");

            let open_value = open.layout_value(cx);
            let selected_value = selected.layout_value(cx);
            let month_label: Arc<str> = month
                .layout(cx)
                .read_ref(|m| Arc::from(format!("{:?} {}", m.month, m.year)))
                .unwrap_or_else(|_| Arc::from("<unknown>"));

            let selected_label: Arc<str> = selected_value
                .map(|d| Arc::from(d.to_string()))
                .unwrap_or_else(|| Arc::from("<none>"));

            let week_start_monday_value = week_start_monday.layout_value(cx);
            let show_outside_days_value = show_outside_days.layout_value(cx);
            let disable_outside_days_value = disable_outside_days.layout_value(cx);
            let disable_weekends_value = disable_weekends.layout_value(cx);
            let disabled_value = disabled.layout_value(cx);

            let header = ui::h_row(|cx| {
                [
                    shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .on_click(CommandId::from("date_picker_demo.close"))
                        .into_element(cx),
                    shadcn::Button::new("Today")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .on_click(CommandId::from("date_picker_demo.reset_today"))
                        .into_element(cx),
                    shadcn::Button::new("Clear")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .on_click(CommandId::from("date_picker_demo.clear"))
                        .into_element(cx),
                    cx.text(Arc::from(format!(
                        "DatePicker | open={} selected={} month={}",
                        if open_value { "true" } else { "false" },
                        selected_label.as_ref(),
                        month_label.as_ref(),
                    ))),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx);

            let toggles = cx.flex(
                FlexProps {
                    layout: LayoutStyle::default(),
                    direction: fret_core::Axis::Horizontal,
                    gap: fret_ui::element::SpacingLength::Px(Px(12.0)),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Center,
                    wrap: true,
                },
                |cx| {
                    vec![
                        cx.text("week start monday"),
                        shadcn::Switch::new(&week_start_monday)
                            .a11y_label("Week starts on Monday")
                            .into_element(cx),
                        cx.text("show outside days"),
                        shadcn::Switch::new(&show_outside_days)
                            .a11y_label("Show outside days")
                            .into_element(cx),
                        cx.text("disable outside days"),
                        shadcn::Switch::new(&disable_outside_days)
                            .a11y_label("Disable outside days")
                            .into_element(cx),
                        cx.text("disable weekends"),
                        shadcn::Switch::new(&disable_weekends)
                            .a11y_label("Disable weekends")
                            .into_element(cx),
                        cx.text("disabled"),
                        shadcn::Switch::new(&disabled)
                            .a11y_label("Disable date picker")
                            .into_element(cx),
                    ]
                },
            );

            let picker = {
                let week_start = if week_start_monday_value {
                    Weekday::Monday
                } else {
                    Weekday::Sunday
                };

                let mut picker = shadcn::DatePicker::new(&open, &month, &selected)
                    .week_start(week_start)
                    .show_outside_days(show_outside_days_value)
                    .disable_outside_days(disable_outside_days_value)
                    .disabled(disabled_value);

                if disable_weekends_value {
                    picker = picker.disabled_by(|d| {
                        matches!(d.weekday(), Weekday::Saturday | Weekday::Sunday)
                    });
                }

                picker.into_element(cx)
            };

            let calendar = {
                let week_start = if week_start_monday_value {
                    Weekday::Monday
                } else {
                    Weekday::Sunday
                };

                let mut cal = shadcn::Calendar::new(&month, &selected)
                    .week_start(week_start)
                    .show_outside_days(show_outside_days_value)
                    .disable_outside_days(disable_outside_days_value);

                if disable_weekends_value {
                    cal = cal.disabled_by(|d| {
                        matches!(d.weekday(), Weekday::Saturday | Weekday::Sunday)
                    });
                }

                cal.into_element(cx)
            };

            let instructions = cx.text(Arc::from(
                "Try: Tab to focus the picker, Enter/Space to open, Arrow keys to navigate days, Enter to select, Esc to close.",
            ));

            let mut root_layout = LayoutStyle::default();
            root_layout.size.width = Length::Fill;
            root_layout.size.height = Length::Fill;

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
                                toggles,
                                instructions,
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
                                        padding: Edges::all(Px(12.0)).into(),
                                        ..Default::default()
                                    },
                                    move |_cx| vec![picker],
                                ),
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
                                        padding: Edges::all(Px(12.0)).into(),
                                        ..Default::default()
                                    },
                                    move |_cx| vec![calendar],
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
    _driver: &mut DatePickerDemoDriver,
    _app: &mut App,
    _request: &fret_app::CreateWindowRequest,
) -> Option<WindowCreateSpec> {
    None
}

fn configure_fn_driver_hooks(
    hooks: &mut fret_launch::FnDriverHooks<DatePickerDemoDriver, DemoWindowState>,
) {
    hooks.hot_reload_window = Some(hot_reload_window);
    hooks.handle_model_changes = Some(handle_model_changes);
    hooks.handle_global_changes = Some(handle_global_changes);
    hooks.handle_command = Some(handle_command);
    hooks.window_create_spec = Some(window_create_spec);
}

pub fn build_fn_driver() -> FnDriver<DatePickerDemoDriver, DemoWindowState> {
    FnDriver::new(
        DatePickerDemoDriver::default(),
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
        main_window_title: "fret-demo date_picker_demo".to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(880.0, 640.0),
        ..Default::default()
    };

    crate::run_native_with_fn_driver_with_hooks(
        config,
        app,
        DatePickerDemoDriver::default(),
        create_window_state,
        handle_event,
        render,
        configure_fn_driver_hooks,
    )
    .context("run date_picker_demo app")
}
