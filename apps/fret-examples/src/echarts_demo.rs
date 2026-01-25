use fret_app::App;
use fret_bootstrap::ui_app_with_hooks;
use fret_core::AppWindowId;
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Invalidation};

use delinea::{ChartEngine, ChartSpec};
use fret_chart::{ChartCanvasPanelProps, chart_canvas_panel};

struct EchartsDemoChart {
    title: std::sync::Arc<str>,
    engine: Model<ChartEngine>,
    spec: ChartSpec,
}

struct EchartsDemoState {
    charts: Vec<EchartsDemoChart>,
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
    fn build_chart(option_json: &str) -> (ChartEngine, ChartSpec) {
        let translated = fret_chart::echarts::translate_json_str(option_json)
            .expect("valid v1 ECharts option JSON");

        let mut engine = ChartEngine::new(translated.spec.clone())
            .expect("translated chart spec should be valid");
        for (dataset_id, table) in translated.datasets {
            engine.datasets_mut().insert(dataset_id, table);
        }
        for action in translated.actions {
            engine.apply_action(action);
        }

        (engine, translated.spec)
    }

    // Intentionally small v1 subset used as a smoke test for the `fret-chart::echarts` adapter.
    let option_basic = r#"
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

    // Percent-window order sensitivity smoke: Y percent extent must be scoped by X filtering,
    // otherwise out-of-window outliers leak into the derived Y domain.
    let option_percent_order = {
        let points: Vec<[f64; 2]> = (0..=100)
            .map(|x| {
                let xf = x as f64;
                let y = if xf < 30.0 || xf > 70.0 { 1000.0 } else { xf };
                [xf, y]
            })
            .collect();
        let value = serde_json::json!({
          "xAxis": { "type": "value", "name": "X" },
          "yAxis": { "type": "value", "name": "Y" },
          "tooltip": { "trigger": "axis", "axisPointer": { "type": "line" } },
          "dataZoom": [
            { "type": "inside", "xAxisIndex": 0, "start": 30, "end": 70 },
            { "type": "inside", "yAxisIndex": 0, "start": 0, "end": 100 }
          ],
          "series": [
            { "type": "scatter", "name": "Scatter", "data": points }
          ]
        });
        serde_json::to_string_pretty(&value).expect("serialize ECharts option")
    };

    let (engine_basic, spec_basic) = build_chart(option_basic);
    let (engine_percent, spec_percent) = build_chart(&option_percent_order);

    let engine_basic = app.models_mut().insert(engine_basic);
    let engine_percent = app.models_mut().insert(engine_percent);
    EchartsDemoState {
        charts: vec![
            EchartsDemoChart {
                title: "ECharts adapter smoke (category line + bar)".into(),
                engine: engine_basic,
                spec: spec_basic,
            },
            EchartsDemoChart {
                title: "Percent order sensitivity smoke (X before Y)".into(),
                engine: engine_percent,
                spec: spec_percent,
            },
        ],
    }
}

fn view(
    cx: &mut ElementContext<'_, App>,
    st: &mut EchartsDemoState,
) -> fret_bootstrap::ui_app_driver::ViewElements {
    for chart in &st.charts {
        cx.observe_model(&chart.engine, Invalidation::Paint);
    }

    let mut out: Vec<AnyElement> = Vec::new();
    for chart in &st.charts {
        out.push(cx.text(std::sync::Arc::clone(&chart.title)));
        let mut props = ChartCanvasPanelProps::new(chart.spec.clone());
        props.engine = Some(chart.engine.clone());
        out.push(chart_canvas_panel(cx, props));
    }
    out.into()
}
