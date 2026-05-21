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

#[test]
fn chart_stress_demo_uses_declarative_canvas_panel() {
    let source = compact(include_str!("../src/chart_stress_demo.rs"));

    for needle in [
        "usefret_chart::{ChartCanvasPanelProps,chart_canvas_panel};",
        "engine:Model<ChartEngine>",
        "spec:ChartSpec",
        "fnbuild_chart(",
        "points:usize,",
        "scatter_lod:Option<SeriesLodSpecV1>,",
        ")->(ChartEngine,ChartSpec)",
        "fret_ui::declarative::render_root(",
        "ChartCanvasPanelProps::new(spec)",
        "props.engine=Some(engine);",
        "vec![chart_canvas_panel(cx,props)]",
        "chart_stress_demo:points={}avg_declarative_render={:.1}usstage_runs(data/layout/visual/marks)={}/{}/{}/{}emitted(points/marks)={}/{}",
    ] {
        assert!(
            source.contains(needle),
            "chart_stress_demo should mount stress charts through the declarative chart panel and keep stats reporting; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_chart::retained::ChartCanvas;",
        "structChartStressCanvas",
        "impl<H:fret_ui::UiHost>Widget<H>forChartStressCanvas",
        "usefret_ui::retained_bridge::",
        "ChartCanvas::new(",
        "ChartCanvas::create_node(",
        "create_node_retained(",
        "avg_canvas_paint",
    ] {
        assert!(
            !source.contains(legacy),
            "chart_stress_demo should not retain the chart stress widget wrapper; unexpected `{legacy}`"
        );
    }
}

#[test]
fn chart_multi_axis_demo_uses_declarative_canvas_panel_with_linked_inputs() {
    let source = compact(include_str!("../src/chart_multi_axis_demo.rs"));

    for needle in [
        "usefret_chart::{AxisPointerLinkAnchor,BrushSelectionLink2D,ChartCanvasOutput,ChartCanvasPanelProps,ChartLinkPolicy,ChartLinkRouter,LinkAxisKey,LinkedChartGroup,LinkedChartMember,chart_canvas_panel,};",
        "top_engine:Model<ChartEngine>",
        "bottom_engine:Model<ChartEngine>",
        "top_spec:ChartSpec",
        "bottom_spec:ChartSpec",
        "linked:LinkedChartGroup",
        "fnbuild_chart(chart_id:delinea::ids::ChartId)->(ChartEngine,ChartSpec,ChartLinkRouter)",
        "let(top_engine,top_spec,top_router)=ChartMultiAxisDemoDriver::build_chart",
        "let(bottom_engine,bottom_spec,bottom_router)=ChartMultiAxisDemoDriver::build_chart",
        "ChartCanvasPanelProps::new(spec)",
        ".output_model(output)",
        ".linked_brush(shared_brush)",
        ".linked_axis_pointer(shared_axis_pointer)",
        ".linked_domain_windows(shared_domain_windows)",
        "props.engine=Some(engine);",
        "chart_canvas_panel(cx,props)",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"chart-multi-axis-demo\"",
    ] {
        assert!(
            source.contains(needle),
            "chart_multi_axis_demo should mount linked charts through the declarative chart panel; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_chart::retained::ChartCanvas",
        "ChartCanvas::new(",
        "ChartCanvas::new_shared(",
        "ChartCanvas::create_node(",
        "FixedSplit::create_node_with_children(",
        "Rc<RefCell<ChartEngine>>",
        "std::rc::Rc<std::cell::RefCell<ChartEngine>>",
        "create_node_retained(",
    ] {
        assert!(
            !source.contains(legacy),
            "chart_multi_axis_demo should not retain chart widget or retained split authoring; unexpected `{legacy}`"
        );
    }
}
