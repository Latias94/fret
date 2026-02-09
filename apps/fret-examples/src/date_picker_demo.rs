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
use fret_ui_kit::headless::calendar::CalendarMonth;
use fret_ui_shadcn::button::{Button, ButtonSize, ButtonVariant};
use fret_ui_shadcn::stack;
use fret_ui_shadcn::{Calendar, DatePicker, Space, Switch};
use std::sync::Arc;
use time::{OffsetDateTime, Weekday};

struct DemoWindowState {
    ui: UiTree<App>,
    open: Model<bool>,
    month: Model<CalendarMonth>,
    selected: Model<Option<time::Date>>,
    week_start_monday: Model<bool>,
    show_outside_days: Model<bool>,
    disable_outside_days: Model<bool>,
    disable_weekends: Model<bool>,
    disabled: Model<bool>,
}

#[derive(Default)]
struct DatePickerDemoDriver;

impl DatePickerDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> DemoWindowState {
        let today = OffsetDateTime::now_utc().date();
        let open = app.models_mut().insert(false);
        let selected = app.models_mut().insert(None::<time::Date>);
        let month = app.models_mut().insert(CalendarMonth::from_date(today));

        let week_start_monday = app.models_mut().insert(true);
        let show_outside_days = app.models_mut().insert(true);
        let disable_outside_days = app.models_mut().insert(true);
        let disable_weekends = app.models_mut().insert(false);
        let disabled = app.models_mut().insert(false);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        DemoWindowState {
            ui,
            open,
            month,
            selected,
            week_start_monday,
            show_outside_days,
            disable_outside_days,
            disable_weekends,
            disabled,
        }
    }
}

impl WinitAppDriver for DatePickerDemoDriver {
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
            "date_picker_demo.close" => {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
            }
            "date_picker_demo.reset_today" => {
                let today = OffsetDateTime::now_utc().date();
                let _ = app
                    .models_mut()
                    .update(&state.selected, |v| *v = Some(today));
                let _ = app
                    .models_mut()
                    .update(&state.month, |m| *m = CalendarMonth::from_date(today));
                app.request_redraw(window);
            }
            "date_picker_demo.clear" => {
                let _ = app.models_mut().update(&state.selected, |v| *v = None);
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

        let open = state.open.clone();
        let month = state.month.clone();
        let selected = state.selected.clone();
        let week_start_monday = state.week_start_monday.clone();
        let show_outside_days = state.show_outside_days.clone();
        let disable_outside_days = state.disable_outside_days.clone();
        let disable_weekends = state.disable_weekends.clone();
        let disabled = state.disabled.clone();

        let root =
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                .render_root("date-picker-demo", move |cx| {
                    cx.observe_model(&open, Invalidation::Layout);
                    cx.observe_model(&month, Invalidation::Layout);
                    cx.observe_model(&selected, Invalidation::Layout);
                    cx.observe_model(&week_start_monday, Invalidation::Layout);
                    cx.observe_model(&show_outside_days, Invalidation::Layout);
                    cx.observe_model(&disable_outside_days, Invalidation::Layout);
                    cx.observe_model(&disable_weekends, Invalidation::Layout);
                    cx.observe_model(&disabled, Invalidation::Layout);

                    let theme = cx.theme_snapshot();
                    let padding = theme.metric_required("metric.padding.md");

                    let open_value = cx.app.models().get_copied(&open).unwrap_or(false);
                    let selected_value = cx
                        .app
                        .models()
                        .read(&selected, |s| *s)
                        .ok()
                        .flatten();

                    let month_label = cx
                        .app
                        .models()
                        .read(&month, |m| format!("{:?} {}", m.month, m.year))
                        .ok();

                    let selected_label: Arc<str> = selected_value
                        .map(|d| Arc::from(d.to_string()))
                        .unwrap_or_else(|| Arc::from("<none>"));
                    let month_label: Arc<str> = month_label
                        .map(|s| Arc::from(s))
                        .unwrap_or_else(|| Arc::from("<unknown>"));

                    let week_start_monday_value = cx
                        .app
                        .models()
                        .get_copied(&week_start_monday)
                        .unwrap_or(true);
                    let show_outside_days_value = cx
                        .app
                        .models()
                        .get_copied(&show_outside_days)
                        .unwrap_or(true);
                    let disable_outside_days_value = cx
                        .app
                        .models()
                        .get_copied(&disable_outside_days)
                        .unwrap_or(true);
                    let disable_weekends_value = cx
                        .app
                        .models()
                        .get_copied(&disable_weekends)
                        .unwrap_or(false);
                    let disabled_value = cx.app.models().get_copied(&disabled).unwrap_or(false);

                    let header = stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                Button::new("Close")
                                    .variant(ButtonVariant::Outline)
                                    .size(ButtonSize::Sm)
                                    .on_click(CommandId::from("date_picker_demo.close"))
                                    .into_element(cx),
                                Button::new("Today")
                                    .variant(ButtonVariant::Outline)
                                    .size(ButtonSize::Sm)
                                    .on_click(CommandId::from("date_picker_demo.reset_today"))
                                    .into_element(cx),
                                Button::new("Clear")
                                    .variant(ButtonVariant::Outline)
                                    .size(ButtonSize::Sm)
                                    .on_click(CommandId::from("date_picker_demo.clear"))
                                    .into_element(cx),
                                cx.text(Arc::from(format!(
                                    "DatePicker | open={} selected={} month={}",
                                    if open_value { "true" } else { "false" },
                                    selected_label.as_ref(),
                                    month_label.as_ref(),
                                ))),
                            ]
                        },
                    );

                    let toggles = cx.flex(
                        FlexProps {
                            layout: LayoutStyle::default(),
                            direction: fret_core::Axis::Horizontal,
                            gap: Px(12.0),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                            wrap: true,
                        },
                        |cx| {
                            vec![
                                cx.text("week start monday"),
                                Switch::new(week_start_monday.clone())
                                    .a11y_label("Week starts on Monday")
                                    .into_element(cx),
                                cx.text("show outside days"),
                                Switch::new(show_outside_days.clone())
                                    .a11y_label("Show outside days")
                                    .into_element(cx),
                                cx.text("disable outside days"),
                                Switch::new(disable_outside_days.clone())
                                    .a11y_label("Disable outside days")
                                    .into_element(cx),
                                cx.text("disable weekends"),
                                Switch::new(disable_weekends.clone())
                                    .a11y_label("Disable weekends")
                                    .into_element(cx),
                                cx.text("disabled"),
                                Switch::new(disabled.clone())
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

                        let mut picker = DatePicker::new(open.clone(), month.clone(), selected.clone())
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

                        let mut cal = Calendar::new(month.clone(), selected.clone())
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
                                        toggles.clone(),
                                        instructions.clone(),
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
                                                padding: Edges::all(Px(12.0)),
                                                ..Default::default()
                                            },
                                            move |_cx| vec![picker.clone()],
                                        ),
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
                                                padding: Edges::all(Px(12.0)),
                                                ..Default::default()
                                            },
                                            move |_cx| vec![calendar.clone()],
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
        main_window_title: "fret-demo date_picker_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(880.0, 640.0),
        ..Default::default()
    };

    crate::run_native_demo(config, app, DatePickerDemoDriver::default())
        .context("run date_picker_demo app")
}
