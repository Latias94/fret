#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event};
#[cfg(not(target_arch = "wasm32"))]
use fret_launch::run_app;
use fret_launch::{
    FnDriver, WindowCreateSpec, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_runtime::{Model, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui::declarative;
use fret_ui::element::{
    ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow, PositionStyle,
    StackProps,
};

use delinea::data::DataTable;
use delinea::engine::ChartEngine;
use delinea::ids::GridId;
use delinea::spec::ChartSpec;
use fret_chart::{ChartCanvasPanelProps, chart_canvas_panel};

pub struct EchartsMultiGridDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    engine: Model<ChartEngine>,
    spec: ChartSpec,
    grids: Vec<GridId>,
}

#[derive(Default)]
pub struct EchartsMultiGridDemoDriver;

impl EchartsMultiGridDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> EchartsMultiGridDemoWindowState {
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let (engine, spec, grids) = Self::build_chart();
        let engine = app.models_mut().insert(engine);

        EchartsMultiGridDemoWindowState {
            ui,
            root: None,
            engine,
            spec,
            grids,
        }
    }

    fn build_chart() -> (ChartEngine, ChartSpec, Vec<GridId>) {
        // Intentionally small v1 subset used to validate multi-grid bindings:
        // a single engine instance with per-grid plot viewports, per-grid canvas views,
        // and global controllers (legend + tooltip/axisPointer overlay).
        let option_json = r#"
{
  "grid": [{}, {}],
  "dataset": {
    "source": [
      ["x","a","b"],
      [0,  1,  2],
      [1,  2,  3],
      [2,  4,  5],
      [3,  8,  13],
      [4,  16, 21],
      [5,  32, 34]
    ]
  },
  "xAxis": [
    { "type": "value", "name": "X (grid 0)", "gridIndex": 0 },
    { "type": "value", "name": "X (grid 1)", "gridIndex": 1 }
  ],
  "yAxis": [
    { "type": "value", "name": "A", "gridIndex": 0 },
    { "type": "value", "name": "B", "gridIndex": 1 }
  ],
  "series": [
    {
      "type": "scatter",
      "name": "A scatter",
      "datasetIndex": 0,
      "xAxisIndex": 0,
      "yAxisIndex": 0,
      "encode": { "x": "x", "y": "a" },
      "large": true,
      "progressive": 64
    },
    {
      "type": "line",
      "name": "B line",
      "datasetIndex": 0,
      "xAxisIndex": 1,
      "yAxisIndex": 1,
      "encode": { "x": "x", "y": "b" }
    }
  ]
}
"#;

        let translated = fret_chart::echarts::translate_json_str(option_json)
            .expect("valid v1 ECharts option JSON");
        let fret_chart::echarts::TranslatedChart {
            spec,
            datasets,
            actions,
        } = translated;

        let mut engine =
            ChartEngine::new(spec.clone()).expect("translated chart spec should be valid");
        for (dataset_id, table) in datasets {
            engine.datasets_mut().insert(dataset_id, table);
        }
        for action in actions {
            engine.apply_action(action);
        }

        let grids = collect_grids(&spec);
        (engine, spec, grids)
    }

    fn chart_panel(
        spec: ChartSpec,
        engine: Model<ChartEngine>,
        grid: GridId,
    ) -> ChartCanvasPanelProps {
        let mut props = ChartCanvasPanelProps::new(spec).grid_view(grid);
        props.engine = Some(engine);
        props.pointer_region.layout.flex.grow = 1.0;
        props.pointer_region.layout.flex.shrink = 1.0;
        props.pointer_region.layout.flex.basis = Length::Px(fret_core::Px(0.0));
        props.pointer_region.layout.overflow = Overflow::Clip;
        props
    }
}

fn collect_grids(spec: &ChartSpec) -> Vec<GridId> {
    if !spec.grids.is_empty() {
        return spec.grids.iter().map(|grid| grid.id).collect();
    }

    let mut grids: Vec<GridId> = spec.axes.iter().map(|axis| axis.grid).collect();
    grids.sort();
    grids.dedup();
    if grids.is_empty() {
        grids.push(GridId::new(1));
    }
    grids
}

fn create_window_state(
    _driver: &mut EchartsMultiGridDemoDriver,
    app: &mut App,
    window: AppWindowId,
) -> EchartsMultiGridDemoWindowState {
    EchartsMultiGridDemoDriver::build_ui(app, window)
}

fn handle_event(
    _driver: &mut EchartsMultiGridDemoDriver,
    context: WinitEventContext<'_, EchartsMultiGridDemoWindowState>,
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
        }
        _ => {
            state.ui.dispatch_event(app, services, event);
        }
    }
}

fn render(
    _driver: &mut EchartsMultiGridDemoDriver,
    context: WinitRenderContext<'_, EchartsMultiGridDemoWindowState>,
) {
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
    let grids = state.grids.clone();
    let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
        .render_root("echarts-multi-grid-demo", move |cx| {
            cx.observe_model(&engine, fret_ui::Invalidation::Paint);

            let mut stack = StackProps::default();
            stack.layout.size.width = Length::Fill;
            stack.layout.size.height = Length::Fill;

            let mut grid_column_layout = LayoutStyle::default();
            grid_column_layout.size.width = Length::Fill;
            grid_column_layout.size.height = Length::Fill;

            let mut grid_column_container = ContainerProps::default();
            grid_column_container.layout = grid_column_layout;

            let mut column = FlexProps::default();
            column.layout.size.width = Length::Fill;
            column.layout.size.height = Length::Fill;
            column.direction = fret_core::Axis::Vertical;
            column.gap = fret_core::Px(8.0).into();
            column.justify = MainAlign::Start;
            column.align = CrossAlign::Stretch;

            let mut overlay_props = ChartCanvasPanelProps::new(spec.clone()).overlay_only();
            overlay_props.engine = Some(engine.clone());
            overlay_props.pointer_region.layout.position = PositionStyle::Absolute;
            overlay_props.pointer_region.layout.size.width = Length::Fill;
            overlay_props.pointer_region.layout.size.height = Length::Fill;

            vec![cx.stack_props(stack, |cx| {
                let spec_for_grids = spec.clone();
                let engine_for_grids = engine.clone();
                let grid_views = cx.container(grid_column_container, move |cx| {
                    vec![cx.flex(column, move |cx| {
                        grids
                            .iter()
                            .copied()
                            .map(|grid| {
                                let props = EchartsMultiGridDemoDriver::chart_panel(
                                    spec_for_grids.clone(),
                                    engine_for_grids.clone(),
                                    grid,
                                );
                                chart_canvas_panel(cx, props)
                            })
                            .collect::<Vec<_>>()
                    })]
                });

                vec![grid_views, chart_canvas_panel(cx, overlay_props)]
            })]
        });
    state.ui.set_root(root);
    state.ui.publish_window_runtime_snapshots(app);
    state.root = Some(root);

    state.ui.request_semantics_snapshot();
    state.ui.ingest_paint_cache_source(scene);
    scene.clear();

    let mut frame =
        fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
    frame.layout_all();
    frame.paint_all(scene);
}

fn window_create_spec(
    _driver: &mut EchartsMultiGridDemoDriver,
    _app: &mut App,
    _request: &fret_app::CreateWindowRequest,
) -> Option<WindowCreateSpec> {
    None
}

fn window_created(
    _driver: &mut EchartsMultiGridDemoDriver,
    _app: &mut App,
    _request: &fret_app::CreateWindowRequest,
    _new_window: AppWindowId,
) {
}

fn configure_fn_driver_hooks(
    hooks: &mut fret_launch::FnDriverHooks<
        EchartsMultiGridDemoDriver,
        EchartsMultiGridDemoWindowState,
    >,
) {
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
        main_window_title: "fret-demo echarts_multi_grid_demo (ECharts -> delinea -> ChartCanvas)"
            .to_string(),
        ..Default::default()
    }
}

pub fn build_fn_driver() -> FnDriver<EchartsMultiGridDemoDriver, EchartsMultiGridDemoWindowState> {
    FnDriver::new(
        EchartsMultiGridDemoDriver::default(),
        create_window_state,
        handle_event,
        render,
    )
    .with_hooks(configure_fn_driver_hooks)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    let app = build_app();
    let config = build_runner_config();
    let driver = build_fn_driver();

    run_app(config, app, driver)
        .context("run echarts_multi_grid_demo app")
        .map_err(anyhow::Error::from)
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
