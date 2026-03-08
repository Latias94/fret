#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event};
use fret_launch::{
    FnDriver, WindowCreateSpec, WinitEventContext, WinitHotReloadContext, WinitRenderContext,
    WinitRunnerConfig,
};
use fret_plot::cartesian::DataRect;
use fret_plot::retained::{
    DragLineX, DragLineY, DragPoint, DragRect, LinePlotCanvas, LinePlotModel, LinePlotStyle,
    LineSeries, PlotDragOutput, PlotOutput, PlotOverlays, PlotState, SeriesTooltipMode, YAxis,
};
use fret_plot::series::Series;
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;

pub struct DragDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    plot: fret_runtime::Model<LinePlotModel>,
    plot_state: fret_runtime::Model<PlotState>,
    plot_output: fret_runtime::Model<PlotOutput>,
    last_applied_output_revision: u64,
}

#[derive(Default)]
pub struct DragDemoDriver;

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

fn create_window_state(
    _driver: &mut DragDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> DragDemoWindowState {
    DragDemoDriver::build_ui(app, window)
}

fn hot_reload_window(
    _driver: &mut DragDemoDriver,
    context: WinitHotReloadContext<'_, DragDemoWindowState>,
) {
    let WinitHotReloadContext {
        app, window, state, ..
    } = context;

    crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
    state.root = None;
}

fn handle_event(
    _driver: &mut DragDemoDriver,
    context: WinitEventContext<'_, DragDemoWindowState>,
    event: &Event,
) {
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
                            DragDemoDriver::apply_drag(s, drag);
                        });
                    }
                }
            }
        }
    }
}

fn render(_driver: &mut DragDemoDriver, context: WinitRenderContext<'_, DragDemoWindowState>) {
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
    _driver: &mut DragDemoDriver,
    _app: &mut App,
    _request: &fret_app::CreateWindowRequest,
) -> Option<WindowCreateSpec> {
    None
}

fn window_created(
    _driver: &mut DragDemoDriver,
    _app: &mut App,
    _request: &fret_app::CreateWindowRequest,
    _new_window: AppWindowId,
) {
}

fn configure_fn_driver_hooks(
    hooks: &mut fret_launch::FnDriverHooks<DragDemoDriver, DragDemoWindowState>,
) {
    hooks.hot_reload_window = Some(hot_reload_window);
    hooks.window_create_spec = Some(window_create_spec);
    hooks.window_created = Some(window_created);
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

pub fn build_fn_driver() -> FnDriver<DragDemoDriver, DragDemoWindowState> {
    FnDriver::new(
        DragDemoDriver::default(),
        create_window_state,
        handle_event,
        render,
    )
    .with_hooks(configure_fn_driver_hooks)
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

    crate::run_native_with_fn_driver_with_hooks(
        config,
        app,
        DragDemoDriver::default(),
        create_window_state,
        handle_event,
        render,
        configure_fn_driver_hooks,
    )
    .context("run drag_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
