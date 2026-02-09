#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event, Px};
#[cfg(not(target_arch = "wasm32"))]
use fret_launch::run_app;
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;

use fret_chart::retained::{UniformGrid, create_multi_grid_chart_canvas_nodes};

struct EchartsMultiGridDemoWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
}

#[derive(Default)]
struct EchartsMultiGridDemoDriver;

impl EchartsMultiGridDemoDriver {
    fn build_ui(window: AppWindowId) -> EchartsMultiGridDemoWindowState {
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        EchartsMultiGridDemoWindowState { ui, root: None }
    }
}

impl WinitAppDriver for EchartsMultiGridDemoDriver {
    type WindowState = EchartsMultiGridDemoWindowState;

    fn create_window_state(&mut self, _app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(window)
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
            }
            _ => {
                state.ui.dispatch_event(app, services, event);
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

        if state.root.is_none() {
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
                actions: _actions,
            } = translated;

            let layout = UniformGrid::new(1).with_gap(Px(8.0));
            let nodes =
                create_multi_grid_chart_canvas_nodes(&mut state.ui, spec, &datasets, layout)
                    .expect("translated chart spec should be valid");

            state.ui.set_root(nodes.root);
            state
                .ui
                .set_focus(nodes.canvases.first().map(|(_, node)| *node));
            state.root = Some(nodes.root);
        }

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
        main_window_title: "fret-demo echarts_multi_grid_demo (ECharts -> delinea -> ChartCanvas)"
            .to_string(),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    EchartsMultiGridDemoDriver::default()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    let app = build_app();
    let config = build_runner_config();
    let driver = build_driver();

    run_app(config, app, driver)
        .context("run echarts_multi_grid_demo app")
        .map_err(anyhow::Error::from)
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
