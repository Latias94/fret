fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn echarts_demo_chart_titles_use_section_chrome_role() {
    let source = include_str!("../src/echarts_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret::advanced::text;",
        "text::section_chrome_label(",
        "std::sync::Arc::clone(&chart.title)",
    ] {
        assert!(
            source.contains(needle),
            "echarts demo chart titles should stay on the shared section chrome text role; missing `{needle}`"
        );
    }

    assert!(
        !source.contains("cx.text(std::sync::Arc::clone(&chart.title))"),
        "echarts demo chart titles should not use bare wrapping text"
    );
    assert!(
        !source.contains("usefret_ui_kit::declarative::textasdecl_text;"),
        "echarts demo chart titles should not import raw kit text helpers"
    );
    assert!(
        !source.contains("decl_text::"),
        "echarts demo chart titles should not call raw kit text helpers"
    );
}

#[test]
fn echarts_demo_uses_chart_binding_for_adapter_smoke() {
    let source = include_str!("../src/echarts_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret_chart::{ChartCanvasPanelBinding,chart_canvas_panel};",
        "structEchartsDemoChart{title:std::sync::Arc<str>,chart:ChartCanvasPanelBinding,}",
        "ChartCanvasPanelBinding::new(app,spec_basic,engine_basic)",
        "ChartCanvasPanelBinding::new(app,spec_percent,engine_percent)",
        "chart.chart.observe_engine_paint(cx);",
        "chart.chart.panel_props()",
        "out.push(chart_canvas_panel(cx,props));",
    ] {
        assert!(
            source.contains(needle),
            "echarts adapter smoke should use the chart binding instead of raw chart model wiring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_runtime::Model;",
        "usefret_ui::{ElementContext,Invalidation};",
        "engine:Model<ChartEngine>",
        "spec:ChartSpec",
        "app.models_mut().insert(engine_basic)",
        "app.models_mut().insert(engine_percent)",
        "cx.observe_model(&chart.engine",
        "ChartCanvasPanelProps::new(chart.spec.clone())",
        "props.engine=Some(chart.engine.clone());",
    ] {
        assert!(
            !source.contains(legacy),
            "echarts adapter smoke should not expose raw chart model/props wiring; unexpected `{legacy}`"
        );
    }
}
