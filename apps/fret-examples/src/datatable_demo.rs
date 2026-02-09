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
use fret_ui_kit::headless::table::{ColumnDef, RowKey, TableState, create_column_helper};
use fret_ui_shadcn::button::{Button, ButtonSize, ButtonVariant};
use fret_ui_shadcn::stack;
use fret_ui_shadcn::{
    DataTable, DataTablePagination, DataTableToolbar, DataTableViewOutput, Space,
};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone)]
struct DemoRow {
    id: u64,
    name: Arc<str>,
    role: Arc<str>,
    score: i32,
}

struct DemoWindowState {
    ui: UiTree<App>,
    table_state: Model<TableState>,
    table_output: Model<DataTableViewOutput>,
    rows: Arc<[DemoRow]>,
    started_at: Instant,
    frame: u64,
    profile_frames_left: u64,
    exit_after_frames: Option<u64>,
}

#[derive(Default)]
struct DataTableDemoDriver;

impl DataTableDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> DemoWindowState {
        let profile_frames_left = std::env::var_os("FRET_DATATABLE_DEMO_PROFILE_FRAMES")
            .or_else(|| std::env::var_os("FRET_TANSTACK_DATATABLE_DEMO_PROFILE_FRAMES"))
            .and_then(|v| v.to_string_lossy().parse::<u64>().ok())
            .unwrap_or(0);
        let exit_after_frames = std::env::var_os("FRET_DATATABLE_DEMO_EXIT_AFTER_FRAMES")
            .or_else(|| std::env::var_os("FRET_TANSTACK_DATATABLE_DEMO_EXIT_AFTER_FRAMES"))
            .and_then(|v| v.to_string_lossy().parse::<u64>().ok());

        let rows: Arc<[DemoRow]> = (0..10_000)
            .map(|i| DemoRow {
                id: i as u64,
                name: Arc::from(format!("User {i}")),
                role: Arc::from(if i % 7 == 0 { "Admin" } else { "Member" }),
                score: ((i * 31) % 997) as i32,
            })
            .collect::<Vec<_>>()
            .into();

        let mut table_state = TableState::default();
        table_state.pagination.page_size = 50;
        let table_state = app.models_mut().insert(table_state);
        let table_output = app.models_mut().insert(DataTableViewOutput::default());

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        DemoWindowState {
            ui,
            table_state,
            table_output,
            rows,
            started_at: Instant::now(),
            frame: 0,
            profile_frames_left,
            exit_after_frames,
        }
    }
}

impl WinitAppDriver for DataTableDemoDriver {
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

        if command.as_str() == "datatable_demo.close" {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
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
            if modifiers.ctrl || modifiers.alt || modifiers.shift || modifiers.meta {
                state.ui.dispatch_event(app, services, event);
                return;
            }

            if *key == fret_core::KeyCode::Escape {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
                return;
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
        let frame_started = Instant::now();

        let rows = Arc::clone(&state.rows);
        let table_state = state.table_state.clone();
        let table_output = state.table_output.clone();
        let root =
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                .render_root("datatable-demo", move |cx| {
                    cx.observe_model(&table_state, Invalidation::Layout);
                    cx.observe_model(&table_output, Invalidation::Layout);

                    let theme = cx.theme_snapshot();
                    let padding = theme.metric_required("metric.padding.md");

                    let (selected, sorting) = cx
                        .app
                        .models()
                        .read(&table_state, |st| {
                            let selected = st.row_selection.len();
                            let sorting = st
                                .sorting
                                .first()
                                .map(|s| {
                                    format!(
                                        "{}:{}",
                                        s.column.as_ref(),
                                        if s.desc { "desc" } else { "asc" }
                                    )
                                })
                                .unwrap_or_else(|| "<none>".to_string());
                            (selected, sorting)
                        })
                        .unwrap_or((0, "<none>".to_string()));

                    let helper = create_column_helper::<DemoRow>();
                    let columns: Vec<ColumnDef<DemoRow>> = vec![
                        helper.clone().accessor("id", |r| r.id),
                        helper.clone().accessor_str("name", |r| r.name.as_ref()),
                        helper.clone().accessor_str("role", |r| r.role.as_ref()),
                        helper.accessor("score", |r| r.score),
                    ];
                    let columns: Arc<[ColumnDef<DemoRow>]> = columns.into();
                    let columns_for_menu: Arc<[(Arc<str>, Arc<str>)]> = Arc::from([
                        (Arc::from("id"), Arc::from("ID")),
                        (Arc::from("name"), Arc::from("Name")),
                        (Arc::from("role"), Arc::from("Role")),
                        (Arc::from("score"), Arc::from("Score")),
                    ]);

                    let mut root_layout = LayoutStyle::default();
                    root_layout.size.width = Length::Fill;
                    root_layout.size.height = Length::Fill;

                    let mut table_slot = LayoutStyle::default();
                    table_slot.size.width = Length::Fill;
                    table_slot.size.height = Length::Fill;
                    table_slot.flex.grow = 1.0;
                    table_slot.flex.basis = Length::Px(Px(0.0));
                    table_slot.overflow = Overflow::Clip;

                    let header = stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                Button::new("Close")
                                    .variant(ButtonVariant::Outline)
                                    .size(ButtonSize::Sm)
                                    .on_click(CommandId::from("datatable_demo.close"))
                                    .into_element(cx),
                                cx.text(Arc::from(format!(
                                    "DataTable | selected={selected} sort={sorting}"
                                ))),
                            ]
                        },
                    );

                    let columns_for_header: Arc<[(Arc<str>, Arc<str>)]> =
                        Arc::clone(&columns_for_menu);
                    let columns_for_toolbar = Arc::clone(&columns_for_header);
                    let toolbar = DataTableToolbar::new(
                        table_state.clone(),
                        Arc::clone(&columns),
                        move |col| {
                            columns_for_toolbar
                                .iter()
                                .find_map(|(id, label)| {
                                    (id.as_ref() == col.id.as_ref()).then(|| Arc::clone(label))
                                })
                                .unwrap_or_else(|| Arc::clone(&col.id))
                        },
                    )
                    .into_element(cx);
                    let pagination =
                        DataTablePagination::new(table_state.clone(), table_output.clone())
                            .into_element(cx);

                    let data_table = DataTable::new()
                        .output_model(table_output.clone())
                        .into_element(
                            cx,
                            Arc::clone(&rows),
                            1,
                            table_state.clone(),
                            Arc::clone(&columns),
                            |row, _i, _parent| RowKey(row.id),
                            move |col| {
                                columns_for_header
                                    .iter()
                                    .find_map(|(id, label)| {
                                        if id.as_ref() == col.id.as_ref() {
                                            Some(Arc::clone(label))
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap_or_else(|| Arc::clone(&col.id))
                            },
                            |cx, col, row| match col.id.as_ref() {
                                "id" => cx.text(Arc::from(row.id.to_string())),
                                "name" => cx.text(Arc::clone(&row.name)),
                                "role" => cx.text(Arc::clone(&row.role)),
                                "score" => cx.text(Arc::from(row.score.to_string())),
                                _ => cx.text(Arc::from("")),
                            },
                        );

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
                                    gap: Px(8.0),
                                    padding: Edges::all(padding),
                                    justify: MainAlign::Start,
                                    align: CrossAlign::Stretch,
                                    wrap: false,
                                },
                                move |cx| {
                                    vec![
                                        header.clone(),
                                        toolbar.clone(),
                                        cx.container(
                                            ContainerProps {
                                                layout: table_slot,
                                                background: Some(theme.color_required("card")),
                                                border: Edges::all(Px(1.0)),
                                                border_color: Some(theme.color_required("border")),
                                                corner_radii: Corners::all(
                                                    theme.metric_required("metric.radius.md"),
                                                ),
                                                ..Default::default()
                                            },
                                            move |_cx| vec![data_table.clone()],
                                        ),
                                        pagination.clone(),
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
        let layout_started = Instant::now();
        frame.layout_all();
        let layout_elapsed = layout_started.elapsed();
        let paint_started = Instant::now();
        frame.paint_all(scene);
        let paint_elapsed = paint_started.elapsed();

        state.frame = state.frame.saturating_add(1);
        if state.profile_frames_left > 0 {
            state.profile_frames_left = state.profile_frames_left.saturating_sub(1);
            let since_start = state.started_at.elapsed();
            let frame_elapsed = frame_started.elapsed();
            tracing::info!(
                "datatable_demo: frame={} since_start={:.2}ms total={:.2}ms layout={:.2}ms paint={:.2}ms",
                state.frame,
                since_start.as_secs_f64() * 1000.0,
                frame_elapsed.as_secs_f64() * 1000.0,
                layout_elapsed.as_secs_f64() * 1000.0,
                paint_elapsed.as_secs_f64() * 1000.0
            );
        }

        if let Some(limit) = state.exit_after_frames {
            if state.frame >= limit {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
                return;
            }
        }

        if state.profile_frames_left > 0 || state.exit_after_frames.is_some() {
            app.request_redraw(window);
        }
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
        main_window_title: "fret-demo datatable_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    crate::run_native_demo(config, app, DataTableDemoDriver::default())
        .context("run datatable_demo app")
}
