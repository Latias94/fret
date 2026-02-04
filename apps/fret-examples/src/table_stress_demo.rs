use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, Model, WindowRequest};
use fret_core::{AppWindowId, Corners, Edges, Event, Px};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitRunnerConfig, WinitWindowContext,
};
use fret_render::{Renderer, WgpuContext};
use fret_runtime::PlatformCapabilities;
use fret_ui::declarative;
use fret_ui::element::{
    ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
};
use fret_ui::{Invalidation, Theme, UiTree, VirtualListScrollHandle};
use fret_ui_kit::headless::table::{
    ColumnDef, ColumnFilter, ColumnPinningState, RowKey, SortSpec, TableState,
    contains_ascii_case_insensitive, create_column_helper,
};
use serde_json::Value;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::alloc_profile;

fn try_println(args: std::fmt::Arguments<'_>) {
    use std::io::Write as _;
    let mut out = std::io::stdout().lock();
    let _ = out.write_fmt(args);
    let _ = out.write_all(b"\n");
}

macro_rules! try_println {
    ($($tt:tt)*) => {
        try_println(format_args!($($tt)*))
    };
}

fn parse_env_usize(key: &str) -> Option<usize> {
    std::env::var_os(key).and_then(|v| v.to_string_lossy().parse::<usize>().ok())
}

fn parse_env_u64(key: &str) -> Option<u64> {
    std::env::var_os(key).and_then(|v| v.to_string_lossy().parse::<u64>().ok())
}

fn parse_env_bool(key: &str) -> Option<bool> {
    std::env::var_os(key).and_then(|v| {
        let s = v.to_string_lossy();
        match s.as_ref() {
            "1" | "true" | "TRUE" | "True" => Some(true),
            "0" | "false" | "FALSE" | "False" => Some(false),
            _ => None,
        }
    })
}

#[derive(Debug, Clone)]
struct DemoRow {
    id: u32,
    id_text: Arc<str>,
    name: Arc<str>,
    role: Arc<str>,
    score: i32,
    score_text: Arc<str>,
}

struct TableStressWindowState {
    ui: UiTree<App>,
    table_state: Model<TableState>,
    rows: Arc<[DemoRow]>,
    columns: Arc<[ColumnDef<DemoRow>]>,
    items_revision: Model<u64>,
    scroll: VirtualListScrollHandle,
    started_at: Instant,
    frame: u64,
    profile_frames_left: u64,
    exit_after_frames: Option<u64>,
    last_renderer_report: Option<Instant>,
    col_label_id: Arc<str>,
    col_label_name: Arc<str>,
    col_label_role: Arc<str>,
    col_label_score: Arc<str>,
    empty_text: Arc<str>,
    alloc_prev: alloc_profile::AllocProfileSnapshot,
    alloc_last_calls: u64,
    alloc_last_bytes: u64,
    alloc_last_render_calls: u64,
    alloc_last_render_bytes: u64,
    alloc_last_layout_calls: u64,
    alloc_last_layout_bytes: u64,
    alloc_last_paint_calls: u64,
    alloc_last_paint_bytes: u64,
}

#[derive(Default)]
struct TableStressDriver;

impl TableStressDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> TableStressWindowState {
        let started_at = Instant::now();
        let profile_frames_left = parse_env_u64("FRET_TABLE_DEMO_PROFILE_FRAMES").unwrap_or(0);
        let exit_after_frames = parse_env_u64("FRET_TABLE_DEMO_EXIT_AFTER_FRAMES");

        let row_count = parse_env_usize("FRET_TABLE_STRESS_ROWS").unwrap_or(100_000);
        let row_count = row_count.clamp(0, 1_000_000);

        let gen_started = Instant::now();
        let rows: Arc<[DemoRow]> = (0..row_count)
            .map(|i| {
                let id = i as u32;
                let score = ((i * 31) % 997) as i32;
                DemoRow {
                    id,
                    id_text: Arc::from(id.to_string()),
                    name: Arc::from(format!("User {i}")),
                    role: Arc::from(if i % 7 == 0 { "Admin" } else { "Member" }),
                    score,
                    score_text: Arc::from(score.to_string()),
                }
            })
            .collect::<Vec<_>>()
            .into();
        let gen_elapsed = gen_started.elapsed();

        if profile_frames_left > 0 {
            tracing::info!(
                "table_stress_demo: generated {} rows in {:.2}ms",
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
        let items_revision = app.models_mut().insert(1u64);

        let helper = create_column_helper::<DemoRow>();
        let columns: Arc<[ColumnDef<DemoRow>]> = Arc::from(
            vec![
                helper.clone().accessor("id", |r| r.id),
                helper
                    .clone()
                    .accessor("name", |r| r.name.clone())
                    .filter_by(|row, q| contains_ascii_case_insensitive(row.name.as_ref(), q)),
                helper
                    .clone()
                    .accessor("role", |r| r.role.clone())
                    .filter_by(|row, q| contains_ascii_case_insensitive(row.role.as_ref(), q)),
                helper.accessor("score", |r| r.score),
            ]
            .into_boxed_slice(),
        );

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        alloc_profile::reset();
        let alloc_prev = alloc_profile::snapshot();

        TableStressWindowState {
            ui,
            table_state,
            rows,
            columns,
            items_revision,
            scroll: VirtualListScrollHandle::new(),
            started_at,
            frame: 0,
            profile_frames_left,
            exit_after_frames,
            last_renderer_report: None,
            col_label_id: Arc::from("ID"),
            col_label_name: Arc::from("Name"),
            col_label_role: Arc::from("Role"),
            col_label_score: Arc::from("Score"),
            empty_text: Arc::from(""),
            alloc_prev,
            alloc_last_calls: 0,
            alloc_last_bytes: 0,
            alloc_last_render_calls: 0,
            alloc_last_render_bytes: 0,
            alloc_last_layout_calls: 0,
            alloc_last_layout_bytes: 0,
            alloc_last_paint_calls: 0,
            alloc_last_paint_bytes: 0,
        }
    }

    fn toggle_sorting(app: &mut App, state: &Model<TableState>) {
        let _ = app.models_mut().update(state, |st| {
            let next = match st.sorting.first() {
                None => Some(SortSpec {
                    column: "score".into(),
                    desc: false,
                }),
                Some(s) if s.column.as_ref() == "score" && !s.desc => Some(SortSpec {
                    column: "score".into(),
                    desc: true,
                }),
                _ => None,
            };

            st.sorting.clear();
            if let Some(next) = next {
                st.sorting.push(next);
            }
        });
    }

    fn toggle_role_filter(app: &mut App, state: &Model<TableState>) {
        let _ = app.models_mut().update(state, |st| {
            let enabled = st
                .column_filters
                .iter()
                .any(|f| f.column.as_ref() == "role" && f.value.as_str() == Some("Admin"));

            st.column_filters
                .retain(|f| !(f.column.as_ref() == "role" && f.value.as_str() == Some("Admin")));
            if !enabled {
                st.column_filters.push(ColumnFilter {
                    column: "role".into(),
                    value: Value::from("Admin"),
                });
            }

            st.pagination.page_index = 0;
        });
    }

    fn toggle_global_filter(app: &mut App, state: &Model<TableState>) {
        let _ = app.models_mut().update(state, |st| {
            if st.global_filter.is_some() {
                st.global_filter = None;
            } else {
                st.global_filter = Some(Value::from("User 1"));
            }

            st.pagination.page_index = 0;
        });
    }

    fn clear_filters(app: &mut App, state: &Model<TableState>) {
        let _ = app.models_mut().update(state, |st| {
            st.column_filters.clear();
            st.global_filter = None;
            st.pagination.page_index = 0;
        });
    }
}

impl WinitAppDriver for TableStressDriver {
    type WindowState = TableStressWindowState;

    fn gpu_ready(&mut self, _app: &mut App, _context: &WgpuContext, renderer: &mut Renderer) {
        renderer.set_perf_enabled(true);
    }

    fn gpu_frame_prepare(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        _context: &WgpuContext,
        renderer: &mut Renderer,
        _scale_factor: f32,
    ) {
        if state.profile_frames_left == 0 && state.exit_after_frames.is_none() {
            return;
        }

        let now = Instant::now();
        let should_report = match state.last_renderer_report {
            None => true,
            Some(last) => now.duration_since(last) >= Duration::from_secs(1),
        };
        if should_report {
            if let Some(snap) = renderer.take_perf_snapshot() {
                if snap.frames != 0 {
                    let pipeline_breakdown =
                        std::env::var_os("FRET_RENDERER_PERF_PIPELINES").is_some();
                    try_println!(
                        "renderer_perf: frames={} encode={:.2}ms prepare_svg={:.2}ms prepare_text={:.2}ms draws={} (quad={} viewport={} image={} text={} path={} mask={} fs={} clipmask={}) pipelines={} binds={} (ubinds={} tbinds={}) scissor={} uniform={}KB instance={}KB vertex={}KB cache_hits={} cache_misses={}",
                        snap.frames,
                        snap.encode_scene_us as f64 / 1000.0,
                        snap.prepare_svg_us as f64 / 1000.0,
                        snap.prepare_text_us as f64 / 1000.0,
                        snap.draw_calls,
                        snap.quad_draw_calls,
                        snap.viewport_draw_calls,
                        snap.image_draw_calls,
                        snap.text_draw_calls,
                        snap.path_draw_calls,
                        snap.mask_draw_calls,
                        snap.fullscreen_draw_calls,
                        snap.clip_mask_draw_calls,
                        snap.pipeline_switches,
                        snap.bind_group_switches,
                        snap.uniform_bind_group_switches,
                        snap.texture_bind_group_switches,
                        snap.scissor_sets,
                        snap.uniform_bytes / 1024,
                        snap.instance_bytes / 1024,
                        snap.vertex_bytes / 1024,
                        snap.scene_encoding_cache_hits,
                        snap.scene_encoding_cache_misses
                    );
                    if pipeline_breakdown {
                        try_println!(
                            "renderer_perf_pipelines: quad={} viewport={} mask={} text_mask={} text_color={} path={} path_msaa={} composite={} fullscreen={} clip_mask={}",
                            snap.pipeline_switches_quad,
                            snap.pipeline_switches_viewport,
                            snap.pipeline_switches_mask,
                            snap.pipeline_switches_text_mask,
                            snap.pipeline_switches_text_color,
                            snap.pipeline_switches_path,
                            snap.pipeline_switches_path_msaa,
                            snap.pipeline_switches_composite,
                            snap.pipeline_switches_fullscreen,
                            snap.pipeline_switches_clip_mask,
                        );
                    }
                }
            }
            state.last_renderer_report = Some(now);
        }
    }

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

        if command.as_str() == "table_stress_demo.close" {
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
                fret_core::KeyCode::KeyS => {
                    Self::toggle_sorting(app, &state.table_state);
                    app.request_redraw(window);
                    return;
                }
                fret_core::KeyCode::KeyF => {
                    Self::toggle_role_filter(app, &state.table_state);
                    app.request_redraw(window);
                    return;
                }
                fret_core::KeyCode::KeyG => {
                    Self::toggle_global_filter(app, &state.table_state);
                    app.request_redraw(window);
                    return;
                }
                fret_core::KeyCode::KeyC => {
                    Self::clear_filters(app, &state.table_state);
                    app.request_redraw(window);
                    return;
                }
                fret_core::KeyCode::KeyR => {
                    let _ = app
                        .models_mut()
                        .update(&state.items_revision, |v| *v = v.wrapping_add(1));
                    app.request_redraw(window);
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
        let alloc_before_frame = alloc_profile::snapshot();

        let render_started = Instant::now();
        let columns = state.columns.clone();
        let col_label_id = state.col_label_id.clone();
        let col_label_name = state.col_label_name.clone();
        let col_label_role = state.col_label_role.clone();
        let col_label_score = state.col_label_score.clone();
        let empty_text = state.empty_text.clone();
        let root =
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                .render_root("table-stress-demo", |cx| {
                    cx.observe_model(&state.table_state, Invalidation::Layout);
                    cx.observe_model(&state.items_revision, Invalidation::Layout);

                    let (selected, sorting, role_filter, global_filter) = cx
                        .app
                        .models()
                        .read(&state.table_state, |st| {
                            let selected = st.row_selection.len();
                            let sorting = st.sorting.first().map(|s| (s.column.clone(), s.desc));
                            let role_filter = st
                                .column_filters
                                .iter()
                                .find(|f| f.column.as_ref() == "role")
                                .map(|f| f.value.clone());
                            let global_filter = st.global_filter.clone();
                            (selected, sorting, role_filter, global_filter)
                        })
                        .unwrap_or((
                            0,
                            None,
                            None,
                            None,
                        ));
                    let items_revision = cx
                        .app
                        .models()
                        .read(&state.items_revision, |v| *v)
                        .unwrap_or(0);

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

                    let scroll = state.scroll.clone();
                    let table_state = state.table_state.clone();
                    let rows = state.rows.clone();

                    let sorting_col = sorting.as_ref().map(|(c, _)| c.as_ref()).unwrap_or("<none>");
                    let sorting_dir = sorting
                        .as_ref()
                        .map(|(_, desc)| if *desc { "desc" } else { "asc" })
                        .unwrap_or("");
                    let sorting_sep = if sorting.is_some() { ":" } else { "" };
                    let role_filter = role_filter
                        .as_ref()
                        .map(|v| v.as_str().unwrap_or("<non-string>"))
                        .unwrap_or("<none>");
                    let global_filter = global_filter
                        .as_ref()
                        .and_then(|v| v.as_str())
                        .unwrap_or("<none>");

                    let header: Arc<str> = Arc::from(format!(
                        "Table stress demo | rows={} | selected={} | sorting={}{}{} | role_filter={} | global_filter={} | items_rev={} | alloc/frame={} ({} B) render={} ({} B) layout={} ({} B) paint={} ({} B) | [S]=toggle sort | [F]=toggle role filter | [G]=toggle global filter | [C]=clear filters | [R]=bump items_rev | [Home]/[End] | [Esc]=close",
                        rows.len(),
                        selected,
                        sorting_col,
                        sorting_sep,
                        sorting_dir,
                        role_filter,
                        global_filter,
                        items_revision,
                        state.alloc_last_calls,
                        state.alloc_last_bytes,
                        state.alloc_last_render_calls,
                        state.alloc_last_render_bytes,
                        state.alloc_last_layout_calls,
                        state.alloc_last_layout_bytes,
                        state.alloc_last_paint_calls,
                        state.alloc_last_paint_bytes
                    ));

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
                                                    columns.as_ref(),
                                                    table_state.clone(),
                                                    &scroll,
                                                    items_revision,
                                                    &|row: &DemoRow, _i| RowKey(row.id as u64),
                                                    None,
                                                    fret_ui_kit::declarative::table::TableViewProps {
                                                        overscan: 8,
                                                        column_resize_mode:
                                                            fret_ui_kit::headless::table::ColumnResizeMode::OnEnd,
                                                        optimize_paint_order: true,
                                                        optimize_grid_lines: parse_env_bool(
                                                            "FRET_TABLE_OPTIMIZE_GRID_LINES",
                                                        )
                                                        // Debug-only/experimental: see the `optimize_grid_lines`
                                                        // doc comment on `TableViewProps` for caveats.
                                                        .unwrap_or(false),
                                                        ..Default::default()
                                                    },
                                                    |_row| None,
                                                    |cx, col, sort_state| {
                                                        let label_base: Arc<str> = match col.id.as_ref() {
                                                            "id" => col_label_id.clone(),
                                                            "name" => col_label_name.clone(),
                                                            "role" => col_label_role.clone(),
                                                            "score" => col_label_score.clone(),
                                                            _ => Arc::from(col.id.as_ref()),
                                                        };
                                                        let label: Arc<str> = if let Some(desc) = sort_state {
                                                            let mut s = label_base.to_string();
                                                            s.push_str(if desc { " ▼" } else { " ▲" });
                                                            Arc::from(s)
                                                        } else {
                                                            label_base
                                                        };
                                                        vec![cx.text(label)]
                                                    },
                                                    |cx, row, col| {
                                                        let text: Arc<str> = match col.id.as_ref() {
                                                            "id" => row.original.id_text.clone(),
                                                            "name" => row.original.name.clone(),
                                                            "role" => row.original.role.clone(),
                                                            "score" => row.original.score_text.clone(),
                                                            _ => empty_text.clone(),
                                                        };
                                                        vec![cx.text(text)]
                                                    },
                                                    None,
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
        let render_elapsed = render_started.elapsed();
        let alloc_after_render = alloc_profile::snapshot();

        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);

        let layout_started = Instant::now();
        frame.layout_all();
        let layout_elapsed = layout_started.elapsed();
        let alloc_after_layout = alloc_profile::snapshot();

        let paint_started = Instant::now();
        frame.paint_all(scene);
        let paint_elapsed = paint_started.elapsed();
        let alloc_after_paint = alloc_profile::snapshot();

        state.alloc_last_render_calls = alloc_after_render
            .alloc_calls
            .saturating_sub(alloc_before_frame.alloc_calls);
        state.alloc_last_render_bytes = alloc_after_render
            .alloc_bytes
            .saturating_sub(alloc_before_frame.alloc_bytes);
        state.alloc_last_layout_calls = alloc_after_layout
            .alloc_calls
            .saturating_sub(alloc_after_render.alloc_calls);
        state.alloc_last_layout_bytes = alloc_after_layout
            .alloc_bytes
            .saturating_sub(alloc_after_render.alloc_bytes);
        state.alloc_last_paint_calls = alloc_after_paint
            .alloc_calls
            .saturating_sub(alloc_after_layout.alloc_calls);
        state.alloc_last_paint_bytes = alloc_after_paint
            .alloc_bytes
            .saturating_sub(alloc_after_layout.alloc_bytes);

        state.alloc_prev = alloc_after_paint;
        state.alloc_last_calls = alloc_after_paint
            .alloc_calls
            .saturating_sub(alloc_before_frame.alloc_calls);
        state.alloc_last_bytes = alloc_after_paint
            .alloc_bytes
            .saturating_sub(alloc_before_frame.alloc_bytes);

        state.frame = state.frame.saturating_add(1);
        if state.profile_frames_left > 0 {
            state.profile_frames_left = state.profile_frames_left.saturating_sub(1);
            let since_start = state.started_at.elapsed();
            let frame_elapsed = frame_started.elapsed();
            tracing::info!(
                "table_stress_demo: frame={} since_start={:.2}ms total={:.2}ms render={:.2}ms layout={:.2}ms paint={:.2}ms alloc={} ({} B) render_alloc={} ({} B) layout_alloc={} ({} B) paint_alloc={} ({} B)",
                state.frame,
                since_start.as_secs_f64() * 1000.0,
                frame_elapsed.as_secs_f64() * 1000.0,
                render_elapsed.as_secs_f64() * 1000.0,
                layout_elapsed.as_secs_f64() * 1000.0,
                paint_elapsed.as_secs_f64() * 1000.0,
                state.alloc_last_calls,
                state.alloc_last_bytes,
                state.alloc_last_render_calls,
                state.alloc_last_render_bytes,
                state.alloc_last_layout_calls,
                state.alloc_last_layout_bytes,
                state.alloc_last_paint_calls,
                state.alloc_last_paint_bytes
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

    let config = WinitRunnerConfig {
        main_window_title: "fret-demo table_stress_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(1200.0, 780.0),
        ..Default::default()
    };

    crate::run_native_demo(config, app, TableStressDriver::default())
        .context("run table_stress_demo app")
}
