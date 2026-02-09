use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, Model, WindowRequest};
use fret_core::{AppWindowId, Event, Px};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitRunnerConfig, WinitWindowContext,
};
use fret_render::{Renderer, WgpuContext};
use fret_runtime::PlatformCapabilities;
use fret_ui::declarative;
use fret_ui::element::{ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign};
use fret_ui::{Invalidation, Theme, UiTree};
use fret_ui_shadcn::{DataGrid, DataGridCanvasAxis, DataGridCanvasOutput};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

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

fn parse_env_bool(key: &str) -> bool {
    let Some(raw) = std::env::var_os(key) else {
        return false;
    };
    let value = raw.to_string_lossy().trim().to_ascii_lowercase();
    matches!(value.as_str(), "1" | "true" | "yes" | "on")
}

struct CanvasDataGridStressWindowState {
    ui: UiTree<App>,
    rows: Arc<Vec<u64>>,
    cols: Arc<Vec<u64>>,
    cell_texts: Arc<Vec<Arc<str>>>,
    variable_sizes: Model<bool>,
    clamp_rows: Model<bool>,
    revision: Model<u64>,
    grid_output: Model<DataGridCanvasOutput>,
    grid_hist: VecDeque<DataGridCanvasOutput>,
    grid_hist_window: usize,
    frame: u64,
    exit_after_frames: Option<u64>,
    auto_scroll: bool,
    last_renderer_report: Option<Instant>,
}

#[derive(Default)]
struct CanvasDataGridStressDriver;

impl CanvasDataGridStressDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> CanvasDataGridStressWindowState {
        let row_count = parse_env_usize("FRET_CANVAS_GRID_ROWS").unwrap_or(200_000);
        let col_count = parse_env_usize("FRET_CANVAS_GRID_COLS").unwrap_or(200);

        let rows: Arc<Vec<u64>> = Arc::new((0..row_count as u64).collect());
        let cols: Arc<Vec<u64>> = Arc::new((0..col_count as u64).collect());

        let cell_texts: Arc<Vec<Arc<str>>> = Arc::new(
            (0..256usize)
                .map(|i| Arc::<str>::from(format!("{i:03}")))
                .collect(),
        );

        let variable_sizes = app
            .models_mut()
            .insert(parse_env_bool("FRET_CANVAS_GRID_VARIABLE"));
        let clamp_rows = app
            .models_mut()
            .insert(parse_env_bool("FRET_CANVAS_GRID_CLAMP_ROWS"));
        let revision = app.models_mut().insert(1u64);
        let grid_output = app.models_mut().insert(DataGridCanvasOutput::default());
        let grid_hist_window = parse_env_usize("FRET_CANVAS_GRID_STATS_WINDOW").unwrap_or(120);

        let exit_after_frames = parse_env_u64("FRET_CANVAS_GRID_EXIT_AFTER_FRAMES");
        let auto_scroll = parse_env_bool("FRET_CANVAS_GRID_AUTO_SCROLL");

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        CanvasDataGridStressWindowState {
            ui,
            rows,
            cols,
            cell_texts,
            variable_sizes,
            clamp_rows,
            revision,
            grid_output,
            grid_hist: VecDeque::new(),
            grid_hist_window,
            frame: 0,
            exit_after_frames,
            auto_scroll,
            last_renderer_report: None,
        }
    }
}

impl WinitAppDriver for CanvasDataGridStressDriver {
    type WindowState = CanvasDataGridStressWindowState;

    fn gpu_ready(&mut self, _app: &mut App, _context: &WgpuContext, renderer: &mut Renderer) {
        renderer.set_perf_enabled(true);
    }

    fn gpu_frame_prepare(
        &mut self,
        app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        _context: &WgpuContext,
        renderer: &mut Renderer,
        _scale_factor: f32,
    ) {
        if !state.auto_scroll && state.exit_after_frames.is_none() {
            return;
        }

        let grid = app
            .models()
            .read(&state.grid_output, |v| *v)
            .unwrap_or_default();
        state.grid_hist.push_back(grid);
        while state.grid_hist.len() > state.grid_hist_window {
            state.grid_hist.pop_front();
        }

        let now = Instant::now();
        let should_report = match state.last_renderer_report {
            None => true,
            Some(last) => now.duration_since(last) >= Duration::from_secs(1),
        };
        if should_report {
            if !state.grid_hist.is_empty() {
                let mut totals_us: Vec<u32> = state
                    .grid_hist
                    .iter()
                    .map(|s| {
                        s.ensure_axes_us
                            .saturating_add(s.apply_overrides_us)
                            .saturating_add(s.compute_viewport_us)
                            .saturating_add(s.build_visible_items_us)
                    })
                    .collect();
                totals_us.sort_unstable();

                let sum_us: u64 = totals_us.iter().map(|v| *v as u64).sum();
                let avg_ms = (sum_us as f64 / totals_us.len() as f64) / 1000.0;
                let p95_idx = ((totals_us.len().saturating_mul(95)).saturating_add(99) / 100)
                    .saturating_sub(1);
                let p95_ms = totals_us.get(p95_idx).copied().unwrap_or_default() as f64 / 1000.0;

                try_println!(
                    "datagrid_canvas_stats: samples={} total_avg={:.3}ms total_p95={:.3}ms",
                    totals_us.len(),
                    avg_ms,
                    p95_ms
                );
            }

            if let Some(snap) = renderer.take_perf_snapshot() {
                if snap.frames != 0 {
                    try_println!(
                        "renderer_perf: frames={} encode={:.2}ms prepare_text={:.2}ms draws={} pipelines={} binds={} scissor={} uniform={}KB instance={}KB vertex={}KB cache_hits={} cache_misses={}",
                        snap.frames,
                        snap.encode_scene_us as f64 / 1000.0,
                        snap.prepare_text_us as f64 / 1000.0,
                        snap.draw_calls,
                        snap.pipeline_switches,
                        snap.bind_group_switches,
                        snap.scissor_sets,
                        snap.uniform_bytes / 1024,
                        snap.instance_bytes / 1024,
                        snap.vertex_bytes / 1024,
                        snap.scene_encoding_cache_hits,
                        snap.scene_encoding_cache_misses
                    );
                }
            }

            try_println!(
                "datagrid_canvas: visible_rows={} visible_cols={} visible_cells={} ensure_axes={:.3}ms overrides={:.3}ms viewport={:.3}ms build={:.3}ms",
                grid.visible_rows,
                grid.visible_cols,
                grid.visible_cells,
                grid.ensure_axes_us as f64 / 1000.0,
                grid.apply_overrides_us as f64 / 1000.0,
                grid.compute_viewport_us as f64 / 1000.0,
                grid.build_visible_items_us as f64 / 1000.0
            );

            state.last_renderer_report = Some(now);
        }
    }

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

        if command.as_str() == "canvas_datagrid_stress_demo.close" {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
        }
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
            ..
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
        } = context;

        state.frame = state.frame.wrapping_add(1);

        let variable = app
            .models()
            .read(&state.variable_sizes, |v| *v)
            .unwrap_or(false);
        let clamp_rows = app
            .models()
            .read(&state.clamp_rows, |v| *v)
            .unwrap_or(false);
        let revision = app.models().read(&state.revision, |v| *v).unwrap_or(1);

        let rows = Arc::clone(&state.rows);
        let cols = Arc::clone(&state.cols);
        let cell_texts = Arc::clone(&state.cell_texts);

        let root = declarative::RenderRootContext::new(
            &mut state.ui,
            app,
            services,
            window,
            bounds,
        )
        .render_root("canvas-datagrid-stress", |cx| {
            cx.observe_model(&state.variable_sizes, Invalidation::Layout);
            cx.observe_model(&state.clamp_rows, Invalidation::Layout);
            cx.observe_model(&state.revision, Invalidation::Layout);
            cx.observe_model(&state.grid_output, Invalidation::Layout);

            let theme = Theme::global(&*cx.app).snapshot();
            let padding = theme.metric_required("metric.padding.md");

            let mut root_layout = LayoutStyle::default();
            root_layout.size.width = Length::Fill;
            root_layout.size.height = Length::Fill;

            let grid = cx
                .app
                .models()
                .read(&state.grid_output, |v| *v)
                .unwrap_or_default();
            let header: Arc<str> = Arc::from(format!(
                "CanvasDataGrid stress | rows={} cols={} | visible={}x{} cells={} | compute={:.3}ms | variable={} clamp_rows={} | [Esc]=close",
                rows.len(),
                cols.len(),
                grid.visible_rows,
                grid.visible_cols,
                grid.visible_cells,
                (grid.ensure_axes_us + grid.apply_overrides_us + grid.compute_viewport_us + grid.build_visible_items_us) as f64 / 1000.0,
                variable,
                clamp_rows
            ));

            let mut grid_slot = LayoutStyle::default();
            grid_slot.size.width = Length::Fill;
            grid_slot.size.height = Length::Fill;
            grid_slot.flex.grow = 1.0;
            grid_slot.flex.basis = Length::Px(Px(0.0));

            let rows_axis = {
                let mut axis = DataGridCanvasAxis::new(Arc::clone(&rows), revision, Px(24.0))
                    .gap(Px(0.0))
                    .reset_measurements_on_revision_change(true);
                if clamp_rows {
                    axis = axis.max(Px(72.0));
                }
                if variable {
                    axis = axis.size_override(|row_key| {
                        // Simulate markdown-like variability: some rows are taller.
                        let h = if row_key % 17 == 0 {
                            Px(96.0)
                        } else if row_key % 7 == 0 {
                            Px(60.0)
                        } else {
                            Px(24.0)
                        };
                        Some(h)
                    });
                } else {
                    axis = axis.fixed();
                }
                axis
            };

            let cols_axis = {
                let mut axis = DataGridCanvasAxis::new(Arc::clone(&cols), revision, Px(120.0))
                    .gap(Px(0.0))
                    .reset_measurements_on_revision_change(true);
                if variable {
                    axis = axis.size_override(|col_key| {
                        let w = if col_key % 9 == 0 {
                            Px(260.0)
                        } else if col_key % 5 == 0 {
                            Px(180.0)
                        } else {
                            Px(120.0)
                        };
                        Some(w)
                    });
                } else {
                    axis = axis.fixed();
                }
                axis
            };

            let grid = DataGrid::new(rows_axis, cols_axis)
                .overscan_rows(8)
                .overscan_cols(4)
                .output_model(state.grid_output.clone())
                .into_element(cx, move |r, c| {
                    let idx = ((r ^ (c.wrapping_mul(31))) & 255) as usize;
                    Arc::clone(&cell_texts[idx])
                });

            vec![cx.container(
                ContainerProps {
                    layout: root_layout,
                    background: Some(theme.color_required("background")),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.flex(
                        FlexProps {
                            layout: root_layout,
                            direction: fret_core::Axis::Vertical,
                            gap: Px(8.0),
                            padding: fret_core::Edges::all(padding),
                            justify: MainAlign::Start,
                            align: CrossAlign::Stretch,
                            wrap: false,
                        },
                        |cx| {
                            vec![
                                cx.text(header),
                                cx.container(
                                    ContainerProps {
                                        layout: grid_slot,
                                        ..Default::default()
                                    },
                                    |_| vec![grid],
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
        frame.layout_all();
        frame.paint_all(scene);

        if let Some(limit) = state.exit_after_frames {
            if state.frame >= limit {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
                return;
            }
        }

        if state.auto_scroll || state.exit_after_frames.is_some() {
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
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap())
        })
        .try_init();

    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());

    let config = WinitRunnerConfig {
        main_window_title: "fret-demo canvas_datagrid_stress_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(1200.0, 780.0),
        ..Default::default()
    };

    crate::run_native_demo(config, app, CanvasDataGridStressDriver::default())
        .context("run canvas_datagrid_stress_demo app")
}
