fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn plot_declarative_demo_uses_default_declarative_line_plot_panel() {
    let source = compact(include_str!("../src/plot_declarative_demo.rs"));

    for needle in [
        "usefret_plot::declarative::{LinePlotPanelProps,line_plot_panel_in};",
        "model:Model<LinePlotModel>",
        "LinePlotModel::from_series(",
        "LineSeries::new(",
        "Series::from_points_sorted(",
        "LinePlotPanelProps::new(self.model.clone())",
        ".x_scale(AxisScale::Linear)",
        "line_plot_panel_in(cx,props).into()",
    ] {
        assert!(
            source.contains(needle),
            "plot_declarative_demo should teach default declarative plot authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_plot::retained",
        "fret_plot::retained::",
        "LinePlotCanvas",
        "PlotCanvas",
        "create_node_retained(",
        "LineChart::into_canvas(",
    ] {
        assert!(
            !source.contains(legacy),
            "plot_declarative_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}
