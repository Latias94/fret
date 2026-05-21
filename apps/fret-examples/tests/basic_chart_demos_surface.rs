fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn basic_chart_demos_use_declarative_canvas_panel() {
    for (name, source) in [
        ("chart_demo", include_str!("../src/chart_demo.rs")),
        (
            "category_line_demo",
            include_str!("../src/category_line_demo.rs"),
        ),
        (
            "horizontal_bars_demo",
            include_str!("../src/horizontal_bars_demo.rs"),
        ),
    ] {
        let source = compact(source);

        for needle in [
            "usefret_chart::{ChartCanvasPanelProps,chart_canvas_panel};",
            "engine:Model<ChartEngine>",
            "spec:ChartSpec",
            "fret_ui::declarative::render_root(",
            "ChartCanvasPanelProps::new(spec)",
            "props.engine=Some(engine);",
            "vec![chart_canvas_panel(cx,props)]",
        ] {
            assert!(
                source.contains(needle),
                "{name} should render charts through the declarative chart panel; missing `{needle}`"
            );
        }

        for legacy in [
            "usefret_chart::retained::ChartCanvas;",
            "ChartCanvas::new(",
            "ChartCanvas::new_shared(",
            "ChartCanvas::create_node(",
            "create_node_retained(",
        ] {
            assert!(
                !source.contains(legacy),
                "{name} should not teach retained chart widget authoring; unexpected `{legacy}`"
            );
        }
    }
}
