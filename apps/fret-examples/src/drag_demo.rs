#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_plot::cartesian::DataRect;
use fret_plot::retained::{
    DragLineX, DragLineY, DragPoint, DragRect, LinePlotCanvas, LinePlotModel, LinePlotStyle,
    LineSeries, PlotDragOutput, PlotOutput, PlotOverlays, PlotState, SeriesTooltipMode, YAxis,
};
use fret_plot::series::Series;
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;

struct DragDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: fret_runtime::Model<LinePlotModel>,
    plot_state: fret_runtime::Model<PlotState>,
    plot_output: fret_runtime::Model<PlotOutput>,
    last_applied_output_revision: u64,
}

#[derive(Default)]
struct DragDemoDriver;

impl DragDemoDriver {
    fn apply_drag(state: &mut PlotState, drag: PlotDragOutput) {
        match drag {
            PlotDragOutput::LineX { id, x, .. } => {
                if let Some(line) = state.overlays.drag_lines_x.iter_mut().find(|l| l.id == id) {
                    line.x = x;
                }
            }
            PlotDragOutput::LineY { id, y, .. } => {
                if let Some(line) = state.overlays.drag_lines_y.iter_mut().find(|l| l.id == id) {
                    line.y = y;
                }
            }
            PlotDragOutput::Point { id, point, .. } => {
                if let Some(p) = state.overlays.drag_points.iter_mut().find(|p| p.id == id) {
                    p.point = point;
                }
            }
            PlotDragOutput::Rect { id, rect, .. } => {
                if let Some(r) = state.overlays.drag_rects.iter_mut().find(|r| r.id == id) {
                    r.rect = rect;
                }
            }
        }
    }

    fn build_ui(app: &mut App, window: AppWindowId) -> DragDemoWindowState {
        let n = 2048usize;
        let mut series0 = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f64 / (n - 1) as f64;
            let x = t * 100.0;
            let y = (t * std::f64::consts::TAU * 3.0).sin();
            if !x.is_finite() || !y.is_finite() {
                continue;
            }
            series0.push(fret_plot::cartesian::DataPoint { x, y });
        }

        let plot = app
            .models_mut()
            .insert(LinePlotModel::from_series(vec![LineSeries::new(
                "signal",
                Series::from_points_sorted(series0, true),
            )]));

        let mut state = PlotState::default();
        state.overlays = PlotOverlays {
            drag_lines_x: vec![
                DragLineX::new(0x5841u64, 25.0).label("X A"),
                DragLineX::new(0x5842u64, 75.0).label("X B"),
            ],
            drag_lines_y: vec![
                DragLineY::new(0x5941u64, 0.5, YAxis::Left).label("Y hi"),
                DragLineY::new(0x5942u64, -0.5, YAxis::Left).label("Y lo"),
            ],
            drag_points: vec![
                DragPoint::new(
                    0x5041u64,
                    fret_plot::cartesian::DataPoint { x: 15.0, y: 0.0 },
                    YAxis::Left,
                )
                .label("P A")
                .show_value(true),
                DragPoint::new(
                    0x5042u64,
                    fret_plot::cartesian::DataPoint { x: 85.0, y: 0.0 },
                    YAxis::Left,
                )
                .label("P B")
                .show_value(true),
            ],
            drag_rects: vec![
                DragRect::new(
                    0x5241u64,
                    DataRect {
                        x_min: 35.0,
                        x_max: 65.0,
                        y_min: -0.25,
                        y_max: 0.25,
                    },
                    YAxis::Left,
                )
                .label("window"),
            ],
            ..Default::default()
        };

        let plot_state = app.models_mut().insert(state);
        let plot_output = app.models_mut().insert(PlotOutput::default());

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        DragDemoWindowState {
            ui,
            root: None,
            plot,
            plot_state,
            plot_output,
            last_applied_output_revision: 0,
        }
    }
}

impl WinitAppDriver for DragDemoDriver {
    type WindowState = DragDemoWindowState;

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
        state.root = None;
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
            _ => {
                state.ui.dispatch_event(app, services, event);

                if matches!(
                    event,
                    Event::Pointer(fret_core::PointerEvent::Move { .. })
                        | Event::Pointer(fret_core::PointerEvent::Up { .. })
                ) {
                    let output = state
                        .plot_output
                        .read(app, |_app, o| *o)
                        .unwrap_or_default();

                    if output.revision != state.last_applied_output_revision {
                        state.last_applied_output_revision = output.revision;
                        if let Some(drag) = output.snapshot.drag {
                            let _ = state.plot_state.update(app, |s, _cx| {
                                Self::apply_drag(s, drag);
                            });
                        }
                    }
                }
            }
        }
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

        let root = state.root.get_or_insert_with(|| {
            let style = LinePlotStyle {
                series_tooltip: SeriesTooltipMode::NearestAtCursor,
                ..Default::default()
            };
            let canvas = LinePlotCanvas::new(state.plot.clone())
                .style(style)
                .state(state.plot_state.clone())
                .output(state.plot_output.clone());
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
        main_window_title:
            "fret-demo drag_demo (DragLineX/DragLineY/DragPoint/DragRect; Shift constrain, Alt snap)"
            .to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    DragDemoDriver::default()
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

    let app = build_app();
    let config = build_runner_config();
    let driver = build_driver();

    crate::run_native_demo(config, app, driver).context("run drag_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
