use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event, KeyCode};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
    WinitWindowContext,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;

use delinea::data::{Column, DataTable};
use delinea::ids::{AxisId, DatasetId, FieldId, GridId, SeriesId};
use delinea::{
    AxisKind, AxisPointerTrigger, AxisPointerType, AxisRange, AxisScale, ChartSpec, DatasetSpec,
    FieldSpec,
};
use delinea::{SeriesEncode, SeriesKind, SeriesSpec};
use fret_chart::retained::ChartCanvas;
use fret_ui::retained_bridge::{CommandCx, EventCx, LayoutCx, PaintCx, SemanticsCx, Widget};
use std::time::{Duration, Instant};

fn parse_env_usize(key: &str) -> Option<usize> {
    std::env::var_os(key).and_then(|v| v.to_string_lossy().parse::<usize>().ok())
}

fn parse_env_u64(key: &str) -> Option<u64> {
    std::env::var_os(key).and_then(|v| v.to_string_lossy().parse::<u64>().ok())
}

fn parse_env_bool(key: &str) -> bool {
    std::env::var_os(key).is_some_and(|v| !v.is_empty() && v != "0")
}

struct ChartStressCanvas {
    points: usize,
    canvas: ChartCanvas,
    last_report: Option<Instant>,
    paint_time_accum: Duration,
    paint_frames_accum: u64,
}

impl ChartStressCanvas {
    fn new(points: usize, canvas: ChartCanvas) -> Self {
        Self {
            points,
            canvas,
            last_report: None,
            paint_time_accum: Duration::ZERO,
            paint_frames_accum: 0,
        }
    }

    fn create_node<H: fret_ui::UiHost>(
        ui: &mut fret_ui::UiTree<H>,
        widget: Self,
    ) -> fret_core::NodeId {
        use fret_ui::retained_bridge::UiTreeRetainedExt as _;
        ui.create_node_retained(widget)
    }
}

impl<H: fret_ui::UiHost> Widget<H> for ChartStressCanvas {
    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        <ChartCanvas as Widget<H>>::event(&mut self.canvas, cx, event);
    }

    fn command(&mut self, cx: &mut CommandCx<'_, H>, command: &fret_runtime::CommandId) -> bool {
        <ChartCanvas as Widget<H>>::command(&mut self.canvas, cx, command)
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        <ChartCanvas as Widget<H>>::cleanup_resources(&mut self.canvas, services);
    }

    fn render_transform(&self, bounds: fret_core::Rect) -> Option<fret_core::Transform2D> {
        <ChartCanvas as Widget<H>>::render_transform(&self.canvas, bounds)
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> fret_core::Size {
        <ChartCanvas as Widget<H>>::layout(&mut self.canvas, cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        if self.last_report.is_none() {
            self.last_report = Some(Instant::now());
        }

        let start = Instant::now();
        <ChartCanvas as Widget<H>>::paint(&mut self.canvas, cx);
        let elapsed = start.elapsed();

        self.paint_time_accum += elapsed;
        self.paint_frames_accum = self.paint_frames_accum.saturating_add(1);

        let stats = self.canvas.engine().stats().clone();
        if let Some(last) = self.last_report
            && last.elapsed() >= Duration::from_secs(1)
        {
            let avg_us = if self.paint_frames_accum == 0 {
                0.0
            } else {
                self.paint_time_accum.as_secs_f64() * 1_000_000.0 / self.paint_frames_accum as f64
            };

            println!(
                "chart_stress_demo: points={} avg_canvas_paint={:.1}us stage_runs(data/layout/visual/marks)={}/{}/{}/{} emitted(points/marks)={}/{}",
                self.points,
                avg_us,
                stats.stage_data_runs,
                stats.stage_layout_runs,
                stats.stage_visual_runs,
                stats.stage_marks_runs,
                stats.points_emitted,
                stats.marks_emitted
            );

            self.last_report = Some(Instant::now());
            self.paint_time_accum = Duration::ZERO;
            self.paint_frames_accum = 0;
        }

        // Keep the widget painting even when paint caching would otherwise replay.
        cx.request_animation_frame();
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        <ChartCanvas as Widget<H>>::semantics(&mut self.canvas, cx);
    }
}

struct ChartStressWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    frame: u64,
    max_frames: Option<u64>,
    last_driver_report: Option<Instant>,
    driver_time_accum: Duration,
    driver_frames_accum: u64,
}

#[derive(Default)]
struct ChartStressDriver {
    points: usize,
    max_frames: Option<u64>,
}

impl ChartStressDriver {
    fn print_help() {
        println!("chart_stress_demo env:");
        println!("  FRET_CHART_STRESS_POINTS=<usize> (default 1_000_000, clamp 1..=10_000_000)");
        println!("  FRET_CHART_STRESS_EXIT_AFTER_FRAMES=<u64> (optional)");
        println!("  FRET_CHART_STRESS_HELP=1 (print this help on start)");
        println!();
        println!("controls:");
        println!("  H: print help");
        println!("  Esc: close");
    }

    fn build_canvas(points: usize) -> ChartCanvas {
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
        let x_field = FieldId::new(1);
        let y_line_field = FieldId::new(2);
        let y_scatter_field = FieldId::new(3);

        let spec = ChartSpec {
            id: delinea::ids::ChartId::new(1),
            viewport: None,
            datasets: vec![DatasetSpec {
                id: dataset_id,
                fields: vec![
                    FieldSpec {
                        id: x_field,
                        column: 0,
                    },
                    FieldSpec {
                        id: y_line_field,
                        column: 1,
                    },
                    FieldSpec {
                        id: y_scatter_field,
                        column: 2,
                    },
                ],
            }],
            grids: vec![delinea::GridSpec { id: grid_id }],
            axes: vec![
                delinea::AxisSpec {
                    id: x_axis,
                    name: Some("X".to_string()),
                    kind: AxisKind::X,
                    grid: grid_id,
                    position: None,
                    scale: AxisScale::Value(Default::default()),
                    range: Some(AxisRange::Auto),
                },
                delinea::AxisSpec {
                    id: y_axis,
                    name: Some("Y".to_string()),
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: None,
                    scale: AxisScale::Value(Default::default()),
                    range: Some(AxisRange::Auto),
                },
            ],
            data_zoom_x: vec![],
            data_zoom_y: vec![],
            tooltip: None,
            axis_pointer: Some(delinea::AxisPointerSpec {
                enabled: true,
                trigger: AxisPointerTrigger::Axis,
                pointer_type: AxisPointerType::Line,
                label: Default::default(),
                snap: false,
                trigger_distance_px: 12.0,
                throttle_px: 0.75,
            }),
            visual_maps: vec![],
            series: vec![
                SeriesSpec {
                    id: SeriesId::new(1),
                    name: Some("Line".to_string()),
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_line_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                },
                SeriesSpec {
                    id: SeriesId::new(2),
                    name: Some("Scatter".to_string()),
                    kind: SeriesKind::Scatter,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_scatter_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                },
            ],
        };

        let mut canvas = ChartCanvas::new(spec).expect("chart spec should be valid");

        let n = points.max(1);
        let mut x: Vec<f64> = Vec::with_capacity(n);
        let mut y_line: Vec<f64> = Vec::with_capacity(n);
        let mut y_scatter: Vec<f64> = Vec::with_capacity(n);

        for i in 0..n {
            let t = i as f64 / (n as f64).max(1.0);
            let xi = i as f64;

            // Deterministic discontinuities to validate missing/segment policy (ADR 1141).
            let yi_line = if i != 0 && i % 50_000 == 0 {
                f64::NAN
            } else {
                (t * std::f64::consts::TAU * 64.0).sin() + (t * 7.0).cos() * 0.05
            };

            // Scatter cloud: stable pseudo-noise without RNG.
            let yi_scatter = (t * std::f64::consts::TAU * 91.0).sin() * 0.7
                + ((i as u64 * 6364136223846793005u64 + 1) % 10_000) as f64 / 10_000.0 * 0.15;

            x.push(xi);
            y_line.push(yi_line);
            y_scatter.push(yi_scatter);
        }

        let mut table = DataTable::default();
        table.push_column(Column::F64(x));
        table.push_column(Column::F64(y_line));
        table.push_column(Column::F64(y_scatter));
        canvas.engine_mut().datasets_mut().insert(dataset_id, table);

        canvas
    }
}

impl WinitAppDriver for ChartStressDriver {
    type WindowState = ChartStressWindowState;

    fn create_window_state(&mut self, _app: &mut App, window: AppWindowId) -> Self::WindowState {
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        ChartStressWindowState {
            ui,
            root: None,
            frame: 0,
            max_frames: self.max_frames,
            last_driver_report: None,
            driver_time_accum: Duration::ZERO,
            driver_frames_accum: 0,
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
                key: KeyCode::Escape,
                ..
            } => {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
                return;
            }
            Event::KeyDown {
                key: KeyCode::KeyH,
                repeat,
                ..
            } if !*repeat => {
                Self::print_help();
            }
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
        if state.last_driver_report.is_none() {
            state.last_driver_report = Some(Instant::now());
        }

        let root = state.root.get_or_insert_with(|| {
            let canvas = Self::build_canvas(self.points);
            let widget = ChartStressCanvas::new(self.points, canvas);
            let node = ChartStressCanvas::create_node(&mut state.ui, widget);
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
        state.driver_time_accum += elapsed;
        state.driver_frames_accum = state.driver_frames_accum.saturating_add(1);

        if let Some(last) = state.last_driver_report
            && last.elapsed() >= Duration::from_secs(1)
        {
            let avg_us = if state.driver_frames_accum == 0 {
                0.0
            } else {
                state.driver_time_accum.as_secs_f64() * 1_000_000.0
                    / state.driver_frames_accum as f64
            };

            println!(
                "chart_stress_demo: frames={} avg_driver_render={:.1}us",
                state.frame, avg_us
            );

            state.last_driver_report = Some(Instant::now());
            state.driver_time_accum = Duration::ZERO;
            state.driver_frames_accum = 0;
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
        main_window_title: "fret-demo chart_stress_demo (delinea + fret-chart)".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(1280.0, 720.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    let points = parse_env_usize("FRET_CHART_STRESS_POINTS").unwrap_or(1_000_000);
    let points = points.clamp(1, 10_000_000);
    let max_frames = parse_env_u64("FRET_CHART_STRESS_EXIT_AFTER_FRAMES");

    if parse_env_bool("FRET_CHART_STRESS_HELP") {
        ChartStressDriver::print_help();
    }

    ChartStressDriver { points, max_frames }
}

pub fn run() -> anyhow::Result<()> {
    let config = build_runner_config();
    let app = build_app();
    let driver = build_driver();

    crate::run_native_demo(config, app, driver).context("run chart_stress_demo app")
}
