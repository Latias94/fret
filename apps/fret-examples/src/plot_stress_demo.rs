use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
    WinitWindowContext,
};
use fret_plot::cartesian::{DataPoint, DataRect};
use fret_plot::retained::{LinePlotCanvas, LinePlotModel, LinePlotStyle, LineSeries};
use fret_plot::series::Series;
use fret_render::{Renderer, WgpuContext};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
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

const DEFAULT_POINTS: usize = 200_000;
const DEFAULT_SERIES: usize = 3;

struct PlotStressWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: fret_runtime::Model<LinePlotModel>,
    animate: fret_runtime::Model<bool>,
    max_frames: Option<u64>,
    frame: u64,
    last_report: Option<Instant>,
    last_renderer_report: Option<Instant>,
    render_time_accum: Duration,
    render_frames_accum: u64,
}

#[derive(Default)]
struct PlotStressDriver {
    points: usize,
    series: usize,
    max_frames: Option<u64>,
}

impl PlotStressDriver {
    fn print_help() {
        try_println!("plot_stress_demo controls:");
        try_println!("  Space: toggle animated bounds (forces path rebuild)");
        try_println!("  H: print this help");
        try_println!("  Esc: close");
    }

    fn build_series(points: usize, series_index: usize) -> Series {
        let dt = 1.0 / 120.0;
        let x_max = (points.saturating_sub(1) as f64) * dt;
        let y_amp = 1.0 + (series_index as f64) * 0.15;

        let bounds = DataRect {
            x_min: 0.0,
            x_max,
            y_min: -2.5 * y_amp,
            y_max: 2.5 * y_amp,
        };

        let get = move |i: usize| -> Option<DataPoint> {
            if i >= points {
                return None;
            }

            // Insert deterministic discontinuities to validate segmentation.
            if i != 0 && i % 10_000 == 0 {
                return None;
            }

            let x = i as f64 * dt;
            let base = (x * (1.0 + series_index as f64 * 0.17)).sin()
                + (x * (0.31 + series_index as f64 * 0.07)).cos() * 0.25;
            let mut y = base * y_amp;

            // Insert deterministic spikes that should survive decimation.
            if i % 4096 == 0 {
                y += 2.0 * y_amp;
            }
            if i % 4096 == 2048 {
                y -= 2.0 * y_amp;
            }

            Some(DataPoint { x, y })
        };

        // Getter-backed to avoid allocating/copying large datasets in UI code.
        Series::new(
            fret_plot::series::GetterSeriesData::new(points, get)
                .sorted_by_x(true)
                .bounds_hint(bounds),
        )
    }

    fn build_plot_model(points: usize, series: usize) -> LinePlotModel {
        let series_count = series.max(1);
        let mut out: Vec<LineSeries> = Vec::with_capacity(series_count);

        for i in 0..series_count {
            let label: Arc<str> = Arc::from(format!("signal {i}"));
            let data = Self::build_series(points, i);
            out.push(LineSeries::new(label, data));
        }

        // Use the known analytical bounds to avoid scanning getter-backed data.
        let dt = 1.0 / 120.0;
        let x_max = (points.saturating_sub(1) as f64) * dt;
        let y_amp = 1.0 + ((series_count - 1) as f64) * 0.15;
        let bounds = DataRect {
            x_min: 0.0,
            x_max,
            y_min: -2.5 * y_amp,
            y_max: 2.5 * y_amp,
        };

        LinePlotModel::from_series_with_bounds(out, bounds)
    }

    fn maybe_animate_bounds(state: &mut PlotStressWindowState, app: &mut App) {
        let animate = app.models().read(&state.animate, |v| *v).unwrap_or(false);
        if !animate {
            return;
        }

        // Force a path rebuild periodically while keeping per-frame cost reasonable.
        if state.frame % 120 != 0 {
            return;
        }

        let _ = app.models_mut().update(&state.plot, |m| {
            let span = (m.data_bounds.x_max - m.data_bounds.x_min).max(1e-6);
            let shift = span * 0.05;
            let phase = ((state.frame / 120) % 2) == 1;
            let dir = if phase { 1.0 } else { -1.0 };
            m.data_bounds.x_min = (m.data_bounds.x_min + shift * dir).max(0.0);
            m.data_bounds.x_max =
                (m.data_bounds.x_max + shift * dir).max(m.data_bounds.x_min + 1e-6);
        });
    }
}

impl WinitAppDriver for PlotStressDriver {
    type WindowState = PlotStressWindowState;

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
        let plot = app
            .models_mut()
            .insert(Self::build_plot_model(self.points, self.series));
        let animate = app.models_mut().insert(true);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        PlotStressWindowState {
            ui,
            root: None,
            plot,
            animate,
            max_frames: self.max_frames,
            frame: 0,
            last_report: None,
            last_renderer_report: None,
            render_time_accum: Duration::ZERO,
            render_frames_accum: 0,
        }
    }

    fn hot_reload_window(
        &mut self,
        app: &mut App,
        _services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        state: &mut Self::WindowState,
    ) {
        crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
        state.root = None;
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

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
            ..
        } = context;

        match event {
            Event::WindowCloseRequested
            | Event::KeyDown {
                key: fret_core::KeyCode::Escape,
                ..
            } => {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
                return;
            }
            Event::KeyDown { key, repeat, .. } if !*repeat => match *key {
                fret_core::KeyCode::Space => {
                    let _ = app.models_mut().update(&state.animate, |v| *v = !*v);
                    app.request_redraw(window);
                    return;
                }
                fret_core::KeyCode::KeyH => Self::print_help(),
                _ => {}
            },
            _ => {}
        }

        state.ui.dispatch_event(app, services, event);
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let render_start = Instant::now();
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
        Self::maybe_animate_bounds(state, app);

        if state.last_report.is_none() {
            state.last_report = Some(Instant::now());
        }

        let root = state.root.get_or_insert_with(|| {
            let style = LinePlotStyle::default();
            let canvas = LinePlotCanvas::new(state.plot.clone()).style(style);
            let node = LinePlotCanvas::create_node(&mut state.ui, canvas);
            state.ui.set_root(node);
            node
        });

        state.ui.set_root(*root);
        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);

        scene.clear();
        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();
        frame.paint_all(scene);

        let elapsed = render_start.elapsed();
        state.render_time_accum += elapsed;
        state.render_frames_accum = state.render_frames_accum.saturating_add(1);

        if let Some(last) = state.last_report
            && last.elapsed() >= Duration::from_secs(1)
        {
            let avg_us = if state.render_frames_accum == 0 {
                0.0
            } else {
                state.render_time_accum.as_secs_f64() * 1_000_000.0
                    / state.render_frames_accum as f64
            };

            let animate = app.models().read(&state.animate, |v| *v).unwrap_or(false);

            try_println!(
                "frames={} points={} series={} animate={} avg_driver_render={:.1}us",
                state.frame,
                self.points,
                self.series,
                animate,
                avg_us
            );

            state.last_report = Some(Instant::now());
            state.render_time_accum = Duration::ZERO;
            state.render_frames_accum = 0;
        }

        if let Some(max) = state.max_frames
            && state.frame >= max
        {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        app.request_redraw(window);
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

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "fret-demo plot_stress_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    PlotStressDriver {
        points: DEFAULT_POINTS,
        series: DEFAULT_SERIES,
        max_frames: None,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    let mut points = DEFAULT_POINTS;
    let mut series = DEFAULT_SERIES;
    let mut max_frames: Option<u64> = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--points" => {
                let Some(value) = args.next() else {
                    anyhow::bail!("--points requires a value");
                };
                points = value.parse()?;
            }
            "--series" => {
                let Some(value) = args.next() else {
                    anyhow::bail!("--series requires a value");
                };
                series = value.parse()?;
            }
            "--frames" => {
                let Some(value) = args.next() else {
                    anyhow::bail!("--frames requires a value");
                };
                max_frames = Some(value.parse()?);
            }
            "--help" | "-h" => {
                try_println!(
                    "Usage: plot_stress_demo [--points N] [--series N] [--frames N]\n\nThis is a minimal stress harness aligned with ADR 0094 conventions (deterministic scene generation, periodic perf prints)."
                );
                return Ok(());
            }
            other => anyhow::bail!("unknown arg: {other}"),
        }
    }

    let app = build_app();
    let config = build_runner_config();
    let driver = PlotStressDriver {
        points,
        series,
        max_frames,
    };

    crate::run_native_demo(config, app, driver).context("run plot_stress_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
