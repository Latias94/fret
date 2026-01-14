use fret_app::App;
use fret_bootstrap::ui_app_with_hooks;
use fret_core::AppWindowId;
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Invalidation};

use delinea::{ChartEngine, ChartSpec};
use fret_chart::{ChartCanvasPanelProps, chart_canvas_panel};

struct EchartsDemoState {
    engine: Model<ChartEngine>,
    spec: ChartSpec,
}

pub fn run() -> anyhow::Result<()> {
    ui_app_with_hooks("echarts-demo", init_window, view, |d| d)
        .with_default_diagnostics()
        .with_main_window("echarts_demo", (960.0, 720.0))
        .with_default_config_files()?
        .run()
        .map_err(anyhow::Error::from)
}

fn init_window(app: &mut App, _window: AppWindowId) -> EchartsDemoState {
    // Intentionally small v1 subset used as a smoke test for the `fret-chart::echarts` adapter.
    let option_json = r#"
{
  "xAxis": { "type": "category", "name": "Day", "data": ["Mon","Tue","Wed","Thu","Fri","Sat","Sun"] },
  "yAxis": { "type": "value", "name": "Value" },
  "tooltip": { "trigger": "axis", "axisPointer": { "type": "line" } },
  "series": [
    { "type": "line", "name": "Line", "data": [120, 200, 150, 80, 70, 110, 130] },
    { "type": "bar", "name": "Bar", "data": [42, 66, 21, 100, 140, 90, 30] }
  ]
}
"#;

    let translated =
        fret_chart::echarts::translate_json_str(option_json).expect("valid v1 ECharts option JSON");

    let mut engine =
        ChartEngine::new(translated.spec.clone()).expect("translated chart spec should be valid");
    for (dataset_id, table) in translated.datasets {
        engine.datasets_mut().insert(dataset_id, table);
    }

    let engine = app.models_mut().insert(engine);
    EchartsDemoState {
        engine,
        spec: translated.spec,
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut EchartsDemoState) -> Vec<AnyElement> {
    cx.observe_model(&st.engine, Invalidation::Paint);

    let mut props = ChartCanvasPanelProps::new(st.spec.clone());
    props.engine = Some(st.engine.clone());
    vec![chart_canvas_panel(cx, props)]
}
