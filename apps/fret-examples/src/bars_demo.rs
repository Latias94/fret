#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use delinea::data::{Column, DataTable};
use delinea::ids::{AxisId, DatasetId, FieldId, GridId, SeriesId};
use delinea::{
    AxisKind, AxisPointerTrigger, AxisPointerType, AxisScale, ChartEngine, ChartSpec, DatasetSpec,
    FieldSpec, GridSpec, SeriesEncode, SeriesKind, SeriesSpec,
};
use fret_app::{App, Effect, WindowRequest};
use fret_chart::{ChartCanvasOutput, ChartCanvasPanelProps, chart_canvas_panel};
use fret_core::{AppWindowId, Event};
#[cfg(not(target_arch = "wasm32"))]
use fret_launch::run_app;
use fret_launch::{
    FnDriver, FnDriverHooks, WinitEventContext, WinitHotReloadContext, WinitRenderContext,
    WinitRunnerConfig,
};
use fret_runtime::{Model, PlatformCapabilities};
use fret_ui::{UiTree, declarative};

pub struct BarsDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    engine: Model<ChartEngine>,
    spec: ChartSpec,
    output: Model<ChartCanvasOutput>,
    last_logged_output_revision: u64,
}

#[derive(Default)]
pub struct BarsDemoDriver;

impl BarsDemoDriver {
    fn build_chart() -> (ChartEngine, ChartSpec) {
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
        let x_field = FieldId::new(1);
        let y_field = FieldId::new(2);
        let series_id = SeriesId::new(1);

        let categories: Vec<String> = (0..12).map(|i| format!("Category {i}")).collect();

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
                        id: y_field,
                        column: 1,
                    },
                ],
                ..Default::default()
            }],
            grids: vec![GridSpec { id: grid_id }],
            axes: vec![
                delinea::AxisSpec {
                    id: x_axis,
                    name: Some("Category".to_string()),
                    kind: AxisKind::X,
                    grid: grid_id,
                    position: None,
                    scale: AxisScale::Category(delinea::CategoryAxisScale { categories }),
                    range: None,
                },
                delinea::AxisSpec {
                    id: y_axis,
                    name: Some("Value".to_string()),
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: None,
                    scale: AxisScale::Value(Default::default()),
                    range: Some(delinea::AxisRange::Auto),
                },
            ],
            data_zoom_x: vec![],
            data_zoom_y: vec![],
            tooltip: None,
            axis_pointer: Some(delinea::AxisPointerSpec {
                enabled: true,
                trigger: AxisPointerTrigger::Axis,
                pointer_type: AxisPointerType::Shadow,
                label: Default::default(),
                snap: false,
                trigger_distance_px: 12.0,
                throttle_px: 0.75,
            }),
            visual_maps: vec![],
            series: vec![SeriesSpec {
                id: series_id,
                name: Some("Bars".to_string()),
                kind: SeriesKind::Bar,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_field,
                    y2: None,
                },
                x_axis,
                y_axis,
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            }],
        };

        let mut engine = ChartEngine::new(spec.clone()).expect("chart spec should be valid");

        let n = 12usize;
        let mut x: Vec<f64> = Vec::with_capacity(n);
        let mut y: Vec<f64> = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f64 / (n - 1).max(1) as f64;
            x.push(i as f64);
            y.push((t * 7.0).sin() * 40.0 + (t * 3.0).cos() * 15.0 + 60.0);
        }

        let mut table = DataTable::default();
        table.push_column(Column::F64(x));
        table.push_column(Column::F64(y));
        engine.datasets_mut().insert(dataset_id, table);

        (engine, spec)
    }

    fn build_ui(app: &mut App, window: AppWindowId) -> BarsDemoWindowState {
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let (engine, spec) = Self::build_chart();
        let engine = app.models_mut().insert(engine);
        let output = app.models_mut().insert(ChartCanvasOutput::default());

        BarsDemoWindowState {
            ui,
            root: None,
            engine,
            spec,
            output,
            last_logged_output_revision: 0,
        }
    }
}

fn create_window_state(
    _driver: &mut BarsDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> BarsDemoWindowState {
    BarsDemoDriver::build_ui(app, window)
}

fn hot_reload_window(
    _driver: &mut BarsDemoDriver,
    context: WinitHotReloadContext<'_, BarsDemoWindowState>,
) {
    let WinitHotReloadContext {
        app, window, state, ..
    } = context;

    crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
    state.root = None;
}

fn configure_fn_driver_hooks(hooks: &mut FnDriverHooks<BarsDemoDriver, BarsDemoWindowState>) {
    hooks.hot_reload_window = Some(hot_reload_window);
}

fn handle_event(
    _driver: &mut BarsDemoDriver,
    context: WinitEventContext<'_, BarsDemoWindowState>,
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
                Event::Pointer(fret_core::PointerEvent::Up { .. }) | Event::KeyDown { .. }
            ) {
                let output = state
                    .output
                    .read(app, |_app, o| o.clone())
                    .unwrap_or_default();
                if output.revision != state.last_logged_output_revision {
                    state.last_logged_output_revision = output.revision;
                    if !output.snapshot.tooltip_lines.is_empty() {
                        let tooltip = output
                            .snapshot
                            .tooltip_lines
                            .iter()
                            .map(|line| line.text.as_str())
                            .collect::<Vec<_>>()
                            .join(" | ");
                        tracing::info!("tooltip: {tooltip}");
                    }
                }
            }
        }
    }
}

fn render(_driver: &mut BarsDemoDriver, context: WinitRenderContext<'_, BarsDemoWindowState>) {
    let WinitRenderContext {
        app,
        services,
        window,
        state,
        bounds,
        scale_factor,
        scene,
    } = context;

    let engine = state.engine.clone();
    let spec = state.spec.clone();
    let output = state.output.clone();
    let root = declarative::render_root(
        &mut state.ui,
        app,
        services,
        window,
        bounds,
        "bars-demo-root",
        move |cx| {
            cx.observe_model(&engine, fret_ui::Invalidation::Paint);
            let mut props = ChartCanvasPanelProps::new(spec).output_model(output);
            props.engine = Some(engine);
            vec![chart_canvas_panel(cx, props)]
        },
    );
    state.root = Some(root);

    state.ui.set_root(root);
    state.ui.request_semantics_snapshot();
    state.ui.ingest_paint_cache_source(scene);

    scene.clear();
    let mut frame =
        fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
    frame.layout_all();
    frame.paint_all(scene);
}

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title:
            "fret-demo bars_demo (RMB drag zoom, Alt+LMB drag query, LMB double-click fit)"
                .to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(960.0, 640.0),
        ..Default::default()
    }
}

pub fn build_fn_driver() -> FnDriver<BarsDemoDriver, BarsDemoWindowState> {
    FnDriver::new(
        BarsDemoDriver::default(),
        create_window_state,
        handle_event,
        render,
    )
    .with_hooks(|hooks| {
        hooks.hot_reload_window = Some(hot_reload_window);
    })
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
        BarsDemoDriver::default(),
        create_window_state,
        handle_event,
        render,
        configure_fn_driver_hooks,
    )
    .context("run bars_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
