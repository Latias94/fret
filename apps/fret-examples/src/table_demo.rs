use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, Model, WindowRequest};
use fret_core::{AppWindowId, Corners, Edges, Event, Px};
use fret_icons::IconRegistry;
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitRunnerConfig, WinitWindowContext, run_app,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::declarative;
use fret_ui::element::{
    ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
};
use fret_ui::{Invalidation, Theme, UiTree, VirtualListScrollHandle};
use fret_ui_kit::headless::table::{ColumnPinningState, RowKey, TableState, create_column_helper};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone)]
struct DemoRow {
    id: u32,
    name: Arc<str>,
    role: Arc<str>,
    score: i32,
}

struct TableDemoWindowState {
    ui: UiTree<App>,
    table_state: Model<TableState>,
    rows: Arc<[DemoRow]>,
    scroll: VirtualListScrollHandle,
    started_at: Instant,
    frame: u64,
    profile_frames_left: u64,
    exit_after_frames: Option<u64>,
}

#[derive(Default)]
struct TableDemoDriver;

impl TableDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> TableDemoWindowState {
        let started_at = Instant::now();
        let profile_frames_left = std::env::var_os("FRET_TABLE_DEMO_PROFILE_FRAMES")
            .and_then(|v| v.to_string_lossy().parse::<u64>().ok())
            .unwrap_or(0);
        let exit_after_frames = std::env::var_os("FRET_TABLE_DEMO_EXIT_AFTER_FRAMES")
            .and_then(|v| v.to_string_lossy().parse::<u64>().ok());

        let gen_started = Instant::now();
        let rows: Arc<[DemoRow]> = (0..10_000)
            .map(|i| DemoRow {
                id: i as u32,
                name: Arc::from(format!("User {i}")),
                role: Arc::from(if i % 7 == 0 { "Admin" } else { "Member" }),
                score: ((i * 31) % 997) as i32,
            })
            .collect::<Vec<_>>()
            .into();
        let gen_elapsed = gen_started.elapsed();

        if profile_frames_left > 0 {
            tracing::info!(
                "table_demo: generated {} rows in {:.2}ms",
                rows.len(),
                gen_elapsed.as_secs_f64() * 1000.0
            );
        }

        let mut table_state = TableState::default();
        table_state.pagination.page_size = rows.len();
        table_state.column_sizing.insert("id".into(), 72.0);
        table_state.column_sizing.insert("name".into(), 200.0);
        table_state.column_sizing.insert("role".into(), 140.0);
        table_state.column_sizing.insert("score".into(), 100.0);
        table_state.column_pinning = ColumnPinningState {
            left: vec!["id".into()],
            right: vec!["score".into()],
        };
        let table_state = app.models_mut().insert(table_state);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        TableDemoWindowState {
            ui,
            table_state,
            rows,
            scroll: VirtualListScrollHandle::new(),
            started_at,
            frame: 0,
            profile_frames_left,
            exit_after_frames,
        }
    }
}

impl WinitAppDriver for TableDemoDriver {
    type WindowState = TableDemoWindowState;

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

        if command.as_str() == "table_demo.close" {
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

        if let Event::KeyDown { key, modifiers, .. } = event {
            if modifiers.ctrl || modifiers.alt || modifiers.shift || modifiers.meta {
                state.ui.dispatch_event(app, services, event);
                return;
            }

            match *key {
                fret_core::KeyCode::Escape => {
                    app.push_effect(Effect::Window(WindowRequest::Close(window)));
                    return;
                }
                fret_core::KeyCode::Home => {
                    state
                        .scroll
                        .scroll_to_item(0, fret_ui::ScrollStrategy::Start);
                    app.request_redraw(window);
                    return;
                }
                fret_core::KeyCode::End => {
                    state.scroll.scroll_to_bottom();
                    app.request_redraw(window);
                    return;
                }
                _ => {}
            }
        }

        state.ui.dispatch_event(app, services, event);
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
            ..
        } = context;

        let frame_started = Instant::now();
        let root =
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                .render_root("table-demo", |cx| {
                    cx.observe_model(&state.table_state, Invalidation::Layout);

                    let (selected, sorting) = cx
                        .app
                        .models()
                        .read(&state.table_state, |st| {
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

                    let theme = Theme::global(&*cx.app).clone();

                    let mut root_layout = LayoutStyle::default();
                    root_layout.size.width = Length::Fill;
                    root_layout.size.height = Length::Fill;

                    let mut table_slot = LayoutStyle::default();
                    table_slot.size.width = Length::Fill;
                    table_slot.size.height = Length::Fill;
                    table_slot.flex.grow = 1.0;
                    table_slot.flex.basis = Length::Px(Px(0.0));
                    table_slot.overflow = Overflow::Clip;

                    let helper = create_column_helper::<DemoRow>();
                    let columns = vec![
                        helper.clone().accessor("id", |r| r.id),
                        helper.clone().accessor("name", |r| r.name.clone()),
                        helper.clone().accessor("role", |r| r.role.clone()),
                        helper.accessor("score", |r| r.score),
                    ];

                    let scroll = state.scroll.clone();
                    let table_state = state.table_state.clone();
                    let rows = state.rows.clone();

                    vec![cx.container(
                        ContainerProps {
                            layout: root_layout,
                            background: Some(theme.color_required("background")),
                            padding: Edges::all(theme.metric_required("metric.padding.md")),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![cx.flex(
                                FlexProps {
                                    layout: root_layout,
                                    direction: fret_core::Axis::Vertical,
                                    gap: Px(8.0),
                                    padding: Edges::all(Px(0.0)),
                                    justify: MainAlign::Start,
                                    align: CrossAlign::Stretch,
                                    wrap: false,
                                },
                                move |cx| {
                                    let header: Arc<str> = Arc::from(format!(
                                        "Table demo (virtualized) | rows={} | selected={} | sorting={} | click header to sort | drag handle to resize | [Home]/[End] scroll | [Esc] close",
                                        rows.len(),
                                        selected,
                                        sorting
                                    ));

                                    vec![
                                        cx.text(header),
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
                                            move |cx| {
                                                vec![fret_ui_kit::declarative::table::table_virtualized(
                                                    cx,
                                                    &rows,
                                                    columns.clone(),
                                                    table_state.clone(),
                                                    &scroll,
                                                    1,
                                                    &|row: &DemoRow, _i| RowKey(row.id as u64),
                                                    fret_ui_kit::declarative::table::TableViewProps {
                                                        overscan: 8,
                                                        ..Default::default()
                                                    },
                                                    |_row| None,
                                                    |cx, col, _sort| {
                                                        let label: Arc<str> = match col.id.as_ref() {
                                                            "id" => Arc::from("ID"),
                                                            "name" => Arc::from("Name"),
                                                            "role" => Arc::from("Role"),
                                                            "score" => Arc::from("Score"),
                                                            _ => Arc::from(col.id.as_ref()),
                                                        };
                                                        vec![cx.text(label)]
                                                    },
                                                    |cx, row, col| {
                                                        let text = match col.id.as_ref() {
                                                            "id" => row.original.id.to_string(),
                                                            "name" => row.original.name.as_ref().to_string(),
                                                            "role" => row.original.role.as_ref().to_string(),
                                                            "score" => row.original.score.to_string(),
                                                            _ => "".to_string(),
                                                        };
                                                        vec![cx.text(text)]
                                                    },
                                                )]
                                            },
                                        ),
                                    ]
                                },
                            )]
                        },
                    )]
                });

        state.ui.set_root(root);
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
                "table_demo: frame={} since_start={:.2}ms total={:.2}ms layout={:.2}ms paint={:.2}ms",
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
        .with_env_filter({
            let mut filter = tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap());

            if std::env::var_os("FRET_TABLE_DEMO_PROFILE_FRAMES").is_some()
                || std::env::var_os("FRET_TABLE_DEMO_EXIT_AFTER_FRAMES").is_some()
            {
                filter = filter
                    .add_directive("fret_examples=info".parse().unwrap())
                    .add_directive("fret_ui_kit=info".parse().unwrap());
            }

            filter
        })
        .try_init();

    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    app.with_global_mut(IconRegistry::default, |icons, _app| {
        fret_icons_lucide::register_icons(icons);
    });

    let mut config = WinitRunnerConfig {
        main_window_title: "fret-demo table_demo".to_string(),
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

    run_app(config, app, TableDemoDriver::default()).map_err(anyhow::Error::from)
}
