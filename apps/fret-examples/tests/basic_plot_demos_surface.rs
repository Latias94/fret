fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn plot_declarative_demo_uses_default_declarative_line_plot_panel() {
    let source = compact(include_str!("../src/plot_declarative_demo.rs"));

    for needle in [
        "usefret::app::prelude::*;",
        "usefret_runtime::Model;",
        "usefret_plot::declarative::{LinePlotPanelProps,line_plot_panel_in};",
        "model:Model<LinePlotModel>",
        "fninit(app:&mutApp,_window:WindowId)->Self",
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
        "usefret::advanced::raw::Model;",
        "usefret_plot::retained",
        "fret_plot::retained::",
        "LinePlotCanvas",
        "PlotCanvas",
        "create_node_retained(",
        "LineChart::into_canvas(",
        "advanced::prelude::*",
        "component::prelude::*",
        "KernelApp",
        "AppWindowId",
    ] {
        assert!(
            !source.contains(legacy),
            "plot_declarative_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn tags_demo_uses_default_declarative_line_plot_panel() {
    let source = compact(include_str!("../src/tags_demo.rs"));

    for needle in [
        "usefret::app::prelude::*;",
        "usefret_runtime::Model;",
        "usefret_plot::declarative::{LinePlotPanelProps,line_plot_panel_in};",
        "TagsDemoView",
        "fninit(app:&mutApp,_window:WindowId)->Self",
        "LinePlotModel::from_series(",
        "LineSeries::new(",
        "Series::from_points_sorted(",
        "PlotOverlays{",
        "tags_x:vec![",
        "fret_plot::state::TagX::new(25.0).label(\"T1\")",
        "fret_plot::state::TagX::new(75.0).label(\"T2\")",
        "tags_y:vec![",
        "fret_plot::state::TagY::new(0.5,fret_plot::models::YAxis::Left).label(\"limit\")",
        "text:vec![",
        "fret_plot::state::PlotText::new(50.0,-0.75,fret_plot::models::YAxis::Left,\"PlotTextat(50,-0.75)\",)",
        "LinePlotPanelProps::new(self.model.clone())",
        ".state(self.plot_state.clone())",
        ".output(self.plot_output.clone())",
        "line_plot_panel_in(cx,props).into()",
    ] {
        assert!(
            source.contains(needle),
            "tags_demo should teach default declarative plot authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret::advanced::raw::Model;",
        "usefret_plot::retained",
        "fret_plot::retained::",
        "LinePlotCanvas",
        "PlotCanvas",
        "create_node_retained(",
        "advanced::prelude::*",
        "component::prelude::*",
        "KernelApp",
        "AppWindowId",
    ] {
        assert!(
            !source.contains(legacy),
            "tags_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn plot_image_demo_uses_default_declarative_line_plot_panel() {
    let source = compact(include_str!("../src/plot_image_demo.rs"));

    for needle in [
        "usefret::app::prelude::*;",
        "usefret::app::{RenderContextAccessas_,ui_assets};",
        "usefret_runtime::Model;",
        "usefret_plot::declarative::{LinePlotPanelProps,line_plot_panel_in};",
        "PlotImageDemoView",
        "image:Option<ui_assets::ImageId>",
        "fninit(app:&mutApp,_window:WindowId)->Self",
        "LinePlotModel::from_series(",
        "LineSeries::new(",
        "Series::from_points_sorted(",
        "ui_assets::rgba8_image_state(cx,self.image_size.0,self.image_size.1,self.image_bytes.as_slice(),ui_assets::ImageColorSpace::Srgb,)",
        "PlotImage::new(",
        "PlotImageLayer::BelowGrid",
        "AxisLabelFormatter::number(AxisNumberFormat::Fixed(2))",
        "LinePlotPanelProps::new(self.model.clone())",
        ".y_axis_labels(AxisLabelFormatter::number(AxisNumberFormat::Fixed(2)))",
        ".state(self.plot_state.clone())",
        ".output(self.plot_output.clone())",
        "line_plot_panel_in(cx,props).into()",
    ] {
        assert!(
            source.contains(needle),
            "plot_image_demo should teach default declarative plot authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret::advanced::raw::Model;",
        "usefret_plot::retained",
        "fret_plot::retained::",
        "LinePlotCanvas",
        "PlotCanvas",
        "create_node_retained(",
        "advanced::prelude::*",
        "component::prelude::*",
        "KernelApp",
        "AppWindowId",
        "ImageAssetCacheHostExt",
        "ImageAssetKey",
        "with_image_asset_cache",
        "use_image_asset(",
        "fret_core::ImageColorSpace",
    ] {
        assert!(
            !source.contains(legacy),
            "plot_image_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn drag_demo_uses_manual_harness_declarative_line_plot_panel() {
    let source = compact(include_str!("../src/drag_demo.rs"));

    for needle in [
        "usefret_plot::declarative::{LinePlotPanelProps,line_plot_panel_in};",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds)",
        "LinePlotPanelProps::new(plot.clone())",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "vec![line_plot_panel_in(cx,props)]",
        "PlotDragOutput::LineX",
        "PlotDragOutput::LineY",
        "PlotDragOutput::Point",
        "PlotDragOutput::Rect",
    ] {
        assert!(
            source.contains(needle),
            "drag_demo manual harness should use declarative plot authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_plot::retained",
        "fret_plot::retained::",
        "LinePlotCanvas",
        "PlotCanvas",
        "create_node_retained(",
    ] {
        assert!(
            !source.contains(legacy),
            "drag_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn plot_stress_demo_uses_manual_harness_declarative_line_plot_panel() {
    let source = compact(include_str!("../src/plot_stress_demo.rs"));

    for needle in [
        "usefret_plot::declarative::{LinePlotPanelProps,line_plot_panel_in};",
        "usefret_plot::models::{LinePlotModel,LineSeries};",
        "LinePlotModel::from_series_with_bounds(",
        "LineSeries::new(label,data)",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds)",
        "LinePlotPanelProps::new(plot.clone())",
        ".style(style)",
        "vec![line_plot_panel_in(cx,props)]",
    ] {
        assert!(
            source.contains(needle),
            "plot_stress_demo manual harness should use declarative plot authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_plot::retained",
        "fret_plot::retained::",
        "LinePlotCanvas",
        "PlotCanvas",
        "create_node_retained(",
    ] {
        assert!(
            !source.contains(legacy),
            "plot_stress_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn inf_lines_demo_uses_manual_harness_declarative_line_plot_panel() {
    let source = compact(include_str!("../src/inf_lines_demo.rs"));

    for needle in [
        "usefret_plot::declarative::{LinePlotPanelProps,line_plot_panel_in};",
        "usefret_plot::models::{LinePlotModel,LineSeries,YAxis};",
        "usefret_plot::state::{InfLineX,InfLineY,PlotOutput,PlotOverlays,PlotState};",
        "usefret_plot::style::{LinePlotStyle,SeriesTooltipMode};",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds)",
        "LinePlotModel::from_series(vec![",
        "LinePlotPanelProps::new(plot.clone())",
        ".style(style)",
        ".y_axis_labels(",
        ".y2_axis_labels(",
        ".y3_axis_labels(",
        ".y4_axis_labels(",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "vec![line_plot_panel_in(cx,props)]",
    ] {
        assert!(
            source.contains(needle),
            "inf_lines_demo manual harness should use declarative plot authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_plot::retained",
        "fret_plot::retained::",
        "LinePlotCanvas",
        "PlotCanvas",
        "create_node_retained(",
    ] {
        assert!(
            !source.contains(legacy),
            "inf_lines_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn plot_demo_uses_manual_harness_declarative_line_plot_panel() {
    let source = compact(include_str!("../src/plot_demo.rs"));

    for needle in [
        "usefret_plot::declarative::{LinePlotPanelProps,line_plot_panel_in};",
        "usefret_plot::models::{LinePlotModel,LineSeries,YAxis};",
        "usefret_ui::{UiTree,declarative};",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds)",
        "LinePlotModel::from_series(vec![",
        "LinePlotStyle::default()",
        "LinePlotPanelProps::new(plot.clone())",
        ".x_scale(AxisScale::Log10)",
        ".y_axis_labels(",
        ".y2_axis_labels(",
        ".y3_axis_labels(",
        ".y4_axis_labels(",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "vec![line_plot_panel_in(cx,props)]",
    ] {
        assert!(
            source.contains(needle),
            "plot_demo manual harness should use declarative plot authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_plot::retained",
        "fret_plot::retained::",
        "LinePlotCanvas",
        "PlotCanvas",
        "create_node_retained(",
    ] {
        assert!(
            !source.contains(legacy),
            "plot_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn linked_cursor_demo_uses_manual_harness_declarative_top_line_plot_panel() {
    let source = compact(include_str!("../src/linked_cursor_demo.rs"));

    for needle in [
        "usefret_plot::declarative::{AreaPlotPanelProps,LinePlotPanelProps,area_plot_panel_in,line_plot_panel_in,};",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"linked-cursor-demo-top\"",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"linked-cursor-demo-bottom\"",
        "LinePlotModel::from_series(vec![",
        "AreaPlotModel::from_series(vec![",
        "LinePlotPanelProps::new(top_plot)",
        "AreaPlotPanelProps::new(bottom_plot.clone())",
        ".state(top_state)",
        ".output(top_output)",
        ".state(bottom_state.clone())",
        ".output(bottom_output.clone())",
        "vec![line_plot_panel_in(cx,props)]",
        "vec![area_plot_panel_in(cx,props)]",
        "state.ui.set_focus(Some(top_node));",
    ] {
        assert!(
            source.contains(needle),
            "linked_cursor_demo manual harness should use declarative line/area plot authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "LinePlotCanvas::new(",
        "LinePlotCanvas::create_node(&mutstate.ui,top_canvas)",
        "AreaPlotCanvas::new(",
        "AreaPlotCanvas::create_node(&mutstate.ui,bottom_canvas)",
    ] {
        assert!(
            !source.contains(legacy),
            "linked_cursor_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn area_demo_uses_manual_harness_declarative_area_plot_panel() {
    let source = compact(include_str!("../src/area_demo.rs"));

    for needle in [
        "usefret_plot::declarative::{AreaPlotPanelProps,area_plot_panel_in};",
        "usefret_plot::models::{AreaPlotModel,AreaSeries};",
        "usefret_ui::{UiTree,declarative};",
        "AreaPlotModel::from_series(vec![",
        "AreaSeries::new(",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"area-demo\"",
        "AreaPlotPanelProps::new(plot.clone())",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "vec![area_plot_panel_in(cx,props)]",
    ] {
        assert!(
            source.contains(needle),
            "area_demo manual harness should use declarative area plot authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_plot::retained",
        "fret_plot::retained::",
        "AreaPlotCanvas",
        "PlotCanvas",
        "create_node_retained(",
    ] {
        assert!(
            !source.contains(legacy),
            "area_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn stems_demo_uses_manual_harness_declarative_stems_plot_panel() {
    let source = compact(include_str!("../src/stems_demo.rs"));

    for needle in [
        "usefret_plot::declarative::{StemsPlotPanelProps,stems_plot_panel_in};",
        "usefret_plot::models::{StemsPlotModel,StemsSeries};",
        "usefret_ui::{UiTree,declarative};",
        "StemsPlotModel::from_series(",
        "StemsSeries::new(",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"stems-demo\"",
        "StemsPlotPanelProps::new(plot.clone())",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "vec![stems_plot_panel_in(cx,props)]",
    ] {
        assert!(
            source.contains(needle),
            "stems_demo manual harness should use declarative stems plot authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_plot::retained",
        "fret_plot::retained::",
        "StemsPlotCanvas",
        "PlotCanvas",
        "create_node_retained(",
    ] {
        assert!(
            !source.contains(legacy),
            "stems_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn stairs_demo_uses_manual_harness_declarative_line_plot_panel_with_step_mode() {
    let source = compact(include_str!("../src/stairs_demo.rs"));

    for needle in [
        "usefret_plot::declarative::{LinePlotPanelProps,line_plot_panel_in};",
        "usefret_plot::models::{LinePlotModel,LineSeries,StepMode};",
        "usefret_ui::{UiTree,declarative};",
        "LinePlotModel::from_series(vec![",
        "LineSeries::new(",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"stairs-demo\"",
        "LinePlotPanelProps::new(plot.clone())",
        ".step_mode(StepMode::Post)",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "vec![line_plot_panel_in(cx,props)]",
    ] {
        assert!(
            source.contains(needle),
            "stairs_demo manual harness should use declarative step-mode plot authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_plot::retained",
        "fret_plot::retained::",
        "StairsPlotCanvas",
        "PlotCanvas",
        "create_node_retained(",
    ] {
        assert!(
            !source.contains(legacy),
            "stairs_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn shaded_demo_uses_manual_harness_declarative_shaded_plot_panel() {
    let source = compact(include_str!("../src/shaded_demo.rs"));

    for needle in [
        "usefret_plot::declarative::{ShadedPlotPanelProps,shaded_plot_panel_in};",
        "usefret_plot::models::{ShadedPlotModel,ShadedSeries};",
        "usefret_ui::{UiTree,declarative};",
        "ShadedPlotModel::from_series(vec![",
        "ShadedSeries::new(",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"shaded-demo\"",
        "ShadedPlotPanelProps::new(plot.clone())",
        ".x_axis_labels(AxisLabelFormatter::time_seconds(TimeAxisFormat{",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "vec![shaded_plot_panel_in(cx,props)]",
    ] {
        assert!(
            source.contains(needle),
            "shaded_demo manual harness should use declarative shaded plot authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_plot::retained",
        "fret_plot::retained::",
        "ShadedPlotCanvas",
        "PlotCanvas",
        "create_node_retained(",
    ] {
        assert!(
            !source.contains(legacy),
            "shaded_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn error_bars_demo_uses_manual_harness_declarative_error_bars_plot_panel() {
    let source = compact(include_str!("../src/error_bars_demo.rs"));

    for needle in [
        "usefret_plot::declarative::{ErrorBarsPlotPanelProps,error_bars_plot_panel_in};",
        "usefret_plot::models::{ErrorBar,ErrorBarsPlotModel,ErrorBarsSeries,YAxis};",
        "usefret_ui::{UiTree,declarative};",
        "ErrorBarsPlotModel::from_series(vec![",
        "ErrorBarsSeries::new(",
        ".y_errors(Arc::from(left_y_errors))",
        ".x_errors(Arc::from(left_x_errors))",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"error-bars-demo\"",
        "ErrorBarsPlotPanelProps::new(plot.clone())",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "vec![error_bars_plot_panel_in(cx,props)]",
    ] {
        assert!(
            source.contains(needle),
            "error_bars_demo manual harness should use declarative error-bars plot authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_plot::retained",
        "fret_plot::retained::",
        "ErrorBarsPlotCanvas",
        "PlotCanvas",
        "create_node_retained(",
    ] {
        assert!(
            !source.contains(legacy),
            "error_bars_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn histogram_demo_uses_manual_harness_declarative_histogram_plot_panel() {
    let source = compact(include_str!("../src/histogram_demo.rs"));

    for needle in [
        "usefret_plot::declarative::{HistogramPlotPanelProps,histogram_plot_panel_in};",
        "usefret_plot::models::{HistogramPlotModel,HistogramSeries};",
        "usefret_ui::{UiTree,declarative};",
        "HistogramPlotModel::from_series(series)",
        "HistogramSeries::new(",
        ".bins(80)",
        ".bar_gap_fraction(0.12)",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"histogram-demo\"",
        "HistogramPlotPanelProps::new(plot.clone())",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "vec![histogram_plot_panel_in(cx,props)]",
    ] {
        assert!(
            source.contains(needle),
            "histogram_demo manual harness should use declarative histogram plot authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_plot::retained",
        "fret_plot::retained::",
        "HistogramPlotCanvas",
        "PlotCanvas",
        "create_node_retained(",
    ] {
        assert!(
            !source.contains(legacy),
            "histogram_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn grouped_bars_demo_uses_manual_harness_declarative_bars_plot_panel() {
    let source = compact(include_str!("../src/grouped_bars_demo.rs"));

    for needle in [
        "usefret_plot::declarative::{BarsPlotPanelProps,bars_plot_panel_in};",
        "usefret_plot::models::{BarsPlotModel,CategoryBarSeries};",
        "usefret_ui::{UiTree,declarative};",
        "CategoryBarSeries::new(",
        "BarsPlotModel::grouped_categories(categories,series,0.75,0.18,0.0)",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"grouped-bars-demo\"",
        "BarsPlotPanelProps::new(plot.clone())",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "vec![bars_plot_panel_in(cx,props)]",
    ] {
        assert!(
            source.contains(needle),
            "grouped_bars_demo manual harness should use declarative grouped bars authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_plot::retained",
        "fret_plot::retained::",
        "BarsPlotCanvas",
        "PlotCanvas",
        "create_node_retained(",
    ] {
        assert!(
            !source.contains(legacy),
            "grouped_bars_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn stacked_bars_demo_uses_manual_harness_declarative_bars_plot_panel() {
    let source = compact(include_str!("../src/stacked_bars_demo.rs"));

    for needle in [
        "usefret_plot::declarative::{BarsPlotPanelProps,bars_plot_panel_in};",
        "usefret_plot::models::{BarsPlotModel,CategoryBarSeries};",
        "usefret_ui::{UiTree,declarative};",
        "CategoryBarSeries::new(",
        "BarsPlotModel::stacked_categories(categories,series,0.8)",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"stacked-bars-demo\"",
        "BarsPlotPanelProps::new(plot.clone())",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "vec![bars_plot_panel_in(cx,props)]",
    ] {
        assert!(
            source.contains(needle),
            "stacked_bars_demo manual harness should use declarative stacked bars authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_plot::retained",
        "fret_plot::retained::",
        "BarsPlotCanvas",
        "PlotCanvas",
        "create_node_retained(",
    ] {
        assert!(
            !source.contains(legacy),
            "stacked_bars_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}
#[test]
fn candlestick_demo_uses_manual_harness_declarative_candlestick_plot_panel() {
    let source = compact(include_str!("../src/candlestick_demo.rs"));

    for needle in [
        "usefret_plot::declarative::{CandlestickPlotPanelProps,candlestick_plot_panel_in};",
        "usefret_plot::models::{CandlestickPlotModel,CandlestickSeries,OhlcPoint};",
        "usefret_ui::{UiTree,declarative};",
        "CandlestickPlotModel::from_series(vec![",
        "CandlestickSeries::new_sorted(",
        ".width(0.9)",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"candlestick-demo\"",
        "CandlestickPlotPanelProps::new(plot.clone())",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "vec![candlestick_plot_panel_in(cx,props)]",
    ] {
        assert!(
            source.contains(needle),
            "candlestick_demo manual harness should use declarative candlestick plot authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_plot::retained",
        "fret_plot::retained::",
        "CandlestickPlotCanvas",
        "PlotCanvas",
        "create_node_retained(",
    ] {
        assert!(
            !source.contains(legacy),
            "candlestick_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn heatmap_demo_uses_manual_harness_declarative_heatmap_plot_panel() {
    let source = compact(include_str!("../src/heatmap_demo.rs"));

    for needle in [
        "usefret_plot::declarative::{HeatmapPlotPanelProps,heatmap_plot_panel_in};",
        "usefret_plot::models::HeatmapPlotModel;",
        "usefret_ui::{UiTree,declarative};",
        "HeatmapPlotModel::new(data_bounds,cols,rows,values)",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"heatmap-demo\"",
        "HeatmapPlotPanelProps::new(plot.clone())",
        ".style(style)",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "vec![heatmap_plot_panel_in(cx,props)]",
    ] {
        assert!(
            source.contains(needle),
            "heatmap_demo manual harness should use declarative heatmap plot authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_plot::retained",
        "fret_plot::retained::",
        "HeatmapPlotCanvas",
        "PlotCanvas",
        "create_node_retained(",
    ] {
        assert!(
            !source.contains(legacy),
            "heatmap_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn histogram2d_demo_uses_manual_harness_declarative_histogram2d_plot_panel() {
    let source = compact(include_str!("../src/histogram2d_demo.rs"));

    for needle in [
        "usefret_plot::declarative::{Histogram2DPlotPanelProps,histogram2d_plot_panel_in};",
        "usefret_plot::models::Histogram2DPlotModel;",
        "usefret_ui::{UiTree,declarative};",
        "histogram2d_counts(Histogram2DConfig::new(bounds,256,192),points)",
        "Histogram2DPlotModel::new(grid.data_bounds,grid.cols,grid.rows,grid.values)",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"histogram2d-demo\"",
        "Histogram2DPlotPanelProps::new(plot.clone())",
        ".x_axis_labels(AxisLabelFormatter::number(AxisNumberFormat::Fixed(2)))",
        ".y_axis_labels(AxisLabelFormatter::number(AxisNumberFormat::Fixed(2)))",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "vec![histogram2d_plot_panel_in(cx,props)]",
    ] {
        assert!(
            source.contains(needle),
            "histogram2d_demo manual harness should use declarative histogram2d plot authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_plot::retained",
        "fret_plot::retained::",
        "Histogram2DPlotCanvas",
        "PlotCanvas",
        "create_node_retained(",
    ] {
        assert!(
            !source.contains(legacy),
            "histogram2d_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn docs_index_separates_default_app_plot_demo_from_manual_harnesses() {
    let docs = include_str!("../../../docs/README.md");
    let default_app =
        "Default app plot demo (FretApp + View): [apps/fret-examples/src/plot_declarative_demo.rs]";
    let manual_harness = "Manual plot harnesses (driver-level declarative panels): [apps/fret-examples/src/plot_demo.rs]";

    assert!(
        docs.contains(default_app),
        "docs/README.md should expose the FretApp + View plot demo before lower-level harnesses"
    );
    assert!(
        docs.contains(manual_harness),
        "docs/README.md should classify plot_demo.rs as a manual harness"
    );
    assert!(
        docs.find(default_app) < docs.find(manual_harness),
        "docs/README.md should list the default app plot demo before manual harnesses"
    );
}

#[test]
fn fret_examples_does_not_enable_fret_plot_retained_compat_feature() {
    let manifest = compact(include_str!("../Cargo.toml"));

    assert!(
        manifest.contains("fret-plot={path=\"../../ecosystem/fret-plot\"}"),
        "fret-examples should depend on default declarative fret-plot without feature flags"
    );
    assert!(
        !manifest.contains(
            "fret-plot={path=\"../../ecosystem/fret-plot\",features=[\"compat-retained-canvas\"]}"
        ),
        "fret-examples should not enable fret-plot/compat-retained-canvas after plot demos migrated"
    );
}
