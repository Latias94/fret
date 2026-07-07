fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

fn assert_default_view_demo_driver(
    source: &str,
    demo_title: &str,
    default_size: &str,
    view_type: &str,
    root_name: &str,
) {
    let source = compact(source);
    let demo_title = compact(demo_title);
    let default_size = compact(default_size);
    let needles = [
        "useanyhow::Contextas_;".to_string(),
        "usefret::app::prelude::*;".to_string(),
        "crate::build_default_view_demo_app()".to_string(),
        format!("crate::build_default_view_demo_runner_config(\"{demo_title}\",{default_size})"),
        format!("crate::build_default_view_demo_fn_driver::<{view_type}>(\"{root_name}\")"),
        "->implfret_launch::WinitAppDriver".to_string(),
    ];
    for needle in needles {
        assert!(
            source.contains(&needle),
            "default view demo driver should keep launch wiring in the internal harness; missing `{needle}`"
        );
    }
}

#[test]
fn plot_declarative_demo_uses_default_declarative_line_plot_panel() {
    let source = compact(include_str!("../src/plot_declarative_demo.rs"));

    for needle in [
        "usefret::app::prelude::*;",
        "usefret_plot::LinePlotPanelBinding;",
        "usefret_plot::declarative::line_plot_panel_in;",
        "plot:LinePlotPanelBinding",
        "fninit(app:&mutApp,_window:WindowId)->Self",
        "LinePlotModel::from_series(",
        "LineSeries::new(",
        "Series::from_points_sorted(",
        "LinePlotPanelBinding::new(app,model)",
        "self.plot.panel_props()",
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
        "usefret_runtime::Model;",
        "model:Model<LinePlotModel>",
        "LinePlotPanelProps::new(self.model.clone())",
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
        "usefret_plot::LinePlotPanelBinding;",
        "usefret_plot::declarative::line_plot_panel_in;",
        "TagsDemoView",
        "plot:LinePlotPanelBinding",
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
        "LinePlotPanelBinding::new_with_state(app,model,state)",
        "moddriver;",
        "pubusedriver::{build_app,build_fn_driver,build_runner_config,run};",
        "self.plot.panel_props()",
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
        "usefret_runtime::Model;",
        "model:Model<LinePlotModel>",
        "plot_state:Model<PlotState>",
        "plot_output:Model<PlotOutput>",
        "app.models_mut().insert(PlotOutput::default())",
        "LinePlotPanelProps::new(self.model.clone())",
        ".state(self.plot_state.clone())",
        ".output(self.plot_output.clone())",
        "usefret_bootstrap::ui_app_driver;",
        "ui_app_driver::UiAppDriver::new(",
        "usefret_runtime::PlatformCapabilities;",
        "fret::advanced::view::view_init_window",
        "fret::advanced::view::view_view",
        "fret::advanced::view::ViewWindowState",
        "fret_launch::",
        "FnDriver",
        "build_default_view_demo_app()",
        "build_default_view_demo_runner_config(",
        "build_default_view_demo_fn_driver::<",
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
        "usefret_plot::LinePlotPanelBinding;",
        "usefret_plot::declarative::line_plot_panel_in;",
        "PlotImageDemoView",
        "plot:LinePlotPanelBinding",
        "image:Option<ui_assets::ImageId>",
        "fninit(app:&mutApp,_window:WindowId)->Self",
        "LinePlotModel::from_series(",
        "LineSeries::new(",
        "Series::from_points_sorted(",
        "ui_assets::rgba8_image_state(cx,self.image_size.0,self.image_size.1,self.image_bytes.as_slice(),ui_assets::ImageColorSpace::Srgb,)",
        "LinePlotPanelBinding::new(app,model)",
        "self.plot.update_state(cx.app_mut(),|state|{",
        "PlotImage::new(",
        "PlotImageLayer::BelowGrid",
        "AxisLabelFormatter::number(AxisNumberFormat::Fixed(2))",
        "moddriver;",
        "pubusedriver::{build_app,build_fn_driver,build_runner_config,run};",
        "self.plot.panel_props()",
        ".y_axis_labels(AxisLabelFormatter::number(AxisNumberFormat::Fixed(2)))",
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
        "usefret_runtime::Model;",
        "model:Model<LinePlotModel>",
        "plot_state:Model<PlotState>",
        "plot_output:Model<PlotOutput>",
        "app.models_mut().insert(PlotState",
        "app.models_mut().insert(PlotOutput::default())",
        "cx.app_mut().models_mut().update(&self.plot_state",
        "LinePlotPanelProps::new(self.model.clone())",
        ".state(self.plot_state.clone())",
        ".output(self.plot_output.clone())",
        "usefret_bootstrap::ui_app_driver;",
        "ui_app_driver::UiAppDriver::new(",
        "usefret_runtime::PlatformCapabilities;",
        "fret::advanced::view::view_init_window",
        "fret::advanced::view::view_view",
        "fret::advanced::view::ViewWindowState",
        "fret_launch::",
        "FnDriver",
        "build_default_view_demo_app()",
        "build_default_view_demo_runner_config(",
        "build_default_view_demo_fn_driver::<",
    ] {
        assert!(
            !source.contains(legacy),
            "plot_image_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn tags_demo_driver_owns_default_view_launch_wiring() {
    assert_default_view_demo_driver(
        include_str!("../src/tags_demo/driver.rs"),
        "fret-demo tags_demo",
        "(960.0,640.0)",
        "TagsDemoView",
        "tags-demo",
    );
}

#[test]
fn plot_image_demo_driver_owns_default_view_launch_wiring() {
    assert_default_view_demo_driver(
        include_str!("../src/plot_image_demo/driver.rs"),
        "fret-demo plot_image_demo",
        "(960.0,640.0)",
        "PlotImageDemoView",
        "plot-image-demo",
    );
}

#[test]
fn drag_demo_uses_manual_harness_declarative_line_plot_panel() {
    let source = compact(include_str!("../src/drag_demo.rs"));

    for needle in [
        "usefret_plot::LinePlotPanelBinding;",
        "usefret_plot::declarative::line_plot_panel_in;",
        "plot:LinePlotPanelBinding",
        "LinePlotPanelBinding::new_with_state(app,model,state)",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds)",
        "state.plot.output_untracked(app)",
        "state.plot.update_state(app,|s|{",
        "plot.panel_props().style(style)",
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
        "fret_runtime::Model<",
        "PlotOutput",
        "LinePlotPanelProps::new(plot.clone())",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "state.plot_state.update(app",
        "app.models_mut().insert(PlotOutput::default())",
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
        "usefret_plot::LinePlotPanelBinding;",
        "usefret_plot::declarative::line_plot_panel_in;",
        "usefret_plot::models::{LinePlotModel,LineSeries};",
        "structPlotStressModelOwner{",
        "plot:LinePlotPanelBinding",
        "fnplot_binding(&self)->LinePlotPanelBinding",
        "fnanimate_enabled(&self,app:&App)->bool",
        "fntoggle_animate(&self,app:&mutApp)",
        "fnshift_plot_bounds_for_animation(&self,app:&mutApp,frame:u64)",
        "self.plot.update_model(app,|model,_cx|",
        "state.models.plot_binding()",
        "LinePlotModel::from_series_with_bounds(",
        "LineSeries::new(label,data)",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds)",
        "plot.panel_props().style(style)",
        ".style(style)",
        "vec![line_plot_panel_in(cx,props)]",
    ] {
        assert!(
            source.contains(needle),
            "plot_stress_demo manual harness should use declarative plot authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_plot::declarative::{LinePlotPanelProps,line_plot_panel_in};",
        "plot:Model<LinePlotModel>",
        "fnplot_model(&self)->Model<LinePlotModel>",
        "app.models_mut().insert(PlotStressDriver::build_plot_model(points,series))",
        "LinePlotPanelProps::new(plot.clone())",
        "usefret_plot::retained",
        "fret_plot::retained::",
        "LinePlotCanvas",
        "PlotCanvas",
        "create_node_retained(",
        "app.models().read(&state.animate",
        "app.models_mut().update(&state.animate",
        "app.models_mut().update(&state.plot",
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
        "usefret_plot::LinePlotPanelBinding;",
        "usefret_plot::declarative::line_plot_panel_in;",
        "usefret_plot::models::{LinePlotModel,LineSeries,YAxis};",
        "usefret_plot::state::{InfLineX,InfLineY,PlotOverlays,PlotState};",
        "usefret_plot::style::{LinePlotStyle,SeriesTooltipMode};",
        "plot:LinePlotPanelBinding",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds)",
        "LinePlotModel::from_series(vec![",
        "LinePlotPanelBinding::new_with_state(app,model,state)",
        "state.plot.output_untracked(app)",
        "plot.panel_props()",
        ".style(style)",
        ".y_axis_labels(",
        ".y2_axis_labels(",
        ".y3_axis_labels(",
        ".y4_axis_labels(",
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
        "fret_runtime::Model<",
        "PlotOutput",
        "LinePlotPanelProps::new(plot.clone())",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "app.models_mut().insert(PlotOutput::default())",
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
        "usefret_plot::LinePlotPanelBinding;",
        "usefret_plot::declarative::line_plot_panel_in;",
        "usefret_plot::models::{LinePlotModel,LineSeries,YAxis};",
        "usefret_ui::{UiTree,declarative};",
        "plot:LinePlotPanelBinding",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds)",
        "LinePlotPanelBinding::new(app,LinePlotModel::from_series(vec![",
        "LinePlotStyle::default()",
        "state.plot.output_untracked(app)",
        "plot.panel_props()",
        ".x_scale(AxisScale::Log10)",
        ".y_axis_labels(",
        ".y2_axis_labels(",
        ".y3_axis_labels(",
        ".y4_axis_labels(",
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
        "fret_runtime::Model<",
        "app.models_mut().insert(PlotState::default())",
        "app.models_mut().insert(PlotOutput::default())",
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
        "usefret_plot::declarative::{area_plot_panel_in,line_plot_panel_in};",
        "usefret_plot::{AreaPlotPanelBinding,LinePlotPanelBinding};",
        "top_plot:LinePlotPanelBinding",
        "bottom_plot:AreaPlotPanelBinding",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"linked-cursor-demo-top\"",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"linked-cursor-demo-bottom\"",
        "LinePlotPanelBinding::new(app,LinePlotModel::from_series(vec![",
        "AreaPlotPanelBinding::new(app,AreaPlotModel::from_series(vec![",
        "linked.push_binding(&top_plot).push_binding(&bottom_plot);",
        "top_plot.panel_props().style(top_style)",
        "bottom_plot.panel_props().style(bottom_style)",
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
        "LinkedPlotMember",
        "fret_runtime::Model<",
        "PlotState",
        "PlotOutput",
        "LinePlotPanelProps::new(top_plot)",
        "AreaPlotPanelProps::new(bottom_plot.clone())",
        ".state(top_state)",
        ".output(top_output)",
        ".state(bottom_state.clone())",
        ".output(bottom_output.clone())",
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
        "usefret_plot::AreaPlotPanelBinding;",
        "usefret_plot::declarative::area_plot_panel_in;",
        "usefret_plot::models::{AreaPlotModel,AreaSeries};",
        "usefret_ui::{UiTree,declarative};",
        "plot:AreaPlotPanelBinding",
        "AreaPlotPanelBinding::new(app,AreaPlotModel::from_series(vec![",
        "AreaPlotModel::from_series(vec![",
        "AreaSeries::new(",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"area-demo\"",
        "plot.panel_props()",
        "state.plot.output_untracked(app)",
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
        "fret_runtime::Model<",
        "AreaPlotPanelProps::new(",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "app.models_mut().insert(PlotState::default())",
        "app.models_mut().insert(PlotOutput::default())",
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
        "usefret_plot::StemsPlotPanelBinding;",
        "usefret_plot::declarative::stems_plot_panel_in;",
        "usefret_plot::models::{StemsPlotModel,StemsSeries};",
        "usefret_ui::{UiTree,declarative};",
        "plot:StemsPlotPanelBinding",
        "StemsPlotModel::from_series(",
        "StemsSeries::new(",
        "StemsPlotPanelBinding::new(app,StemsPlotModel::from_series(series))",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"stems-demo\"",
        "plot.panel_props()",
        "state.plot.output_untracked(app)",
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
        "fret_runtime::Model<",
        "app.models_mut().insert(PlotState::default())",
        "app.models_mut().insert(PlotOutput::default())",
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
        "usefret_plot::LinePlotPanelBinding;",
        "usefret_plot::declarative::line_plot_panel_in;",
        "usefret_plot::models::{LinePlotModel,LineSeries,StepMode};",
        "usefret_ui::{UiTree,declarative};",
        "plot:LinePlotPanelBinding",
        "LinePlotPanelBinding::new(app,LinePlotModel::from_series(vec![",
        "LinePlotModel::from_series(vec![",
        "LineSeries::new(",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"stairs-demo\"",
        "plot.panel_props()",
        ".step_mode(StepMode::Post)",
        "state.plot.output_untracked(app)",
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
        "fret_runtime::Model<",
        "LinePlotPanelProps::new(",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "app.models_mut().insert(PlotState::default())",
        "app.models_mut().insert(PlotOutput::default())",
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
        "usefret_plot::ShadedPlotPanelBinding;",
        "usefret_plot::declarative::shaded_plot_panel_in;",
        "usefret_plot::models::{ShadedPlotModel,ShadedSeries};",
        "usefret_ui::{UiTree,declarative};",
        "plot:ShadedPlotPanelBinding",
        "ShadedPlotPanelBinding::new(app,ShadedPlotModel::from_series(vec![",
        "ShadedPlotModel::from_series(vec![",
        "ShadedSeries::new(",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"shaded-demo\"",
        "plot.panel_props()",
        ".x_axis_labels(AxisLabelFormatter::time_seconds(TimeAxisFormat{",
        "state.plot.output_untracked(app)",
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
        "fret_runtime::Model<",
        "ShadedPlotPanelProps::new(",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "app.models_mut().insert(PlotState::default())",
        "app.models_mut().insert(PlotOutput::default())",
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
        "usefret_plot::ErrorBarsPlotPanelBinding;",
        "usefret_plot::declarative::error_bars_plot_panel_in;",
        "usefret_plot::models::{ErrorBar,ErrorBarsPlotModel,ErrorBarsSeries,YAxis};",
        "usefret_ui::{UiTree,declarative};",
        "plot:ErrorBarsPlotPanelBinding",
        "ErrorBarsPlotPanelBinding::new(app,ErrorBarsPlotModel::from_series(vec![",
        "ErrorBarsPlotModel::from_series(vec![",
        "ErrorBarsSeries::new(",
        ".y_errors(Arc::from(left_y_errors))",
        ".x_errors(Arc::from(left_x_errors))",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"error-bars-demo\"",
        "plot.panel_props()",
        "state.plot.output_untracked(app)",
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
        "fret_runtime::Model<",
        "ErrorBarsPlotPanelProps::new(",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "app.models_mut().insert(PlotState::default())",
        "app.models_mut().insert(PlotOutput::default())",
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
        "usefret_plot::HistogramPlotPanelBinding;",
        "usefret_plot::declarative::histogram_plot_panel_in;",
        "usefret_plot::models::{HistogramPlotModel,HistogramSeries};",
        "usefret_ui::{UiTree,declarative};",
        "plot:HistogramPlotPanelBinding",
        "HistogramPlotModel::from_series(series)",
        "HistogramSeries::new(",
        ".bins(80)",
        ".bar_gap_fraction(0.12)",
        "HistogramPlotPanelBinding::new(app,HistogramPlotModel::from_series(series))",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"histogram-demo\"",
        "plot.panel_props()",
        "state.plot.output_untracked(app)",
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
        "fret_runtime::Model<",
        "app.models_mut().insert(PlotState::default())",
        "app.models_mut().insert(PlotOutput::default())",
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
        "usefret_plot::BarsPlotPanelBinding;",
        "usefret_plot::declarative::bars_plot_panel_in;",
        "usefret_plot::models::{BarsPlotModel,CategoryBarSeries};",
        "usefret_ui::{UiTree,declarative};",
        "plot:BarsPlotPanelBinding",
        "CategoryBarSeries::new(",
        "BarsPlotPanelBinding::new(app,BarsPlotModel::grouped_categories(categories,series,0.75,0.18,0.0),)",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"grouped-bars-demo\"",
        "plot.panel_props()",
        "state.plot.output_untracked(app)",
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
        "fret_runtime::Model<",
        "BarsPlotPanelProps::new(",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "app.models_mut().insert(PlotState::default())",
        "app.models_mut().insert(PlotOutput::default())",
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
        "usefret_plot::BarsPlotPanelBinding;",
        "usefret_plot::declarative::bars_plot_panel_in;",
        "usefret_plot::models::{BarsPlotModel,CategoryBarSeries};",
        "usefret_ui::{UiTree,declarative};",
        "plot:BarsPlotPanelBinding",
        "CategoryBarSeries::new(",
        "BarsPlotPanelBinding::new(app,BarsPlotModel::stacked_categories(categories,series,0.8),)",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"stacked-bars-demo\"",
        "plot.panel_props()",
        "state.plot.output_untracked(app)",
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
        "fret_runtime::Model<",
        "BarsPlotPanelProps::new(",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "app.models_mut().insert(PlotState::default())",
        "app.models_mut().insert(PlotOutput::default())",
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
        "usefret_plot::CandlestickPlotPanelBinding;",
        "usefret_plot::declarative::candlestick_plot_panel_in;",
        "usefret_plot::models::{CandlestickPlotModel,CandlestickSeries,OhlcPoint};",
        "usefret_ui::{UiTree,declarative};",
        "plot:CandlestickPlotPanelBinding",
        "CandlestickPlotPanelBinding::new(app,CandlestickPlotModel::from_series(vec![",
        "CandlestickPlotModel::from_series(vec![",
        "CandlestickSeries::new_sorted(",
        ".width(0.9)",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"candlestick-demo\"",
        "plot.panel_props()",
        "state.plot.output_untracked(app)",
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
        "fret_runtime::Model<",
        "CandlestickPlotPanelProps::new(",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "app.models_mut().insert(PlotState::default())",
        "app.models_mut().insert(PlotOutput::default())",
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
        "usefret_plot::HeatmapPlotPanelBinding;",
        "usefret_plot::declarative::heatmap_plot_panel_in;",
        "usefret_plot::models::HeatmapPlotModel;",
        "usefret_ui::{UiTree,declarative};",
        "plot:HeatmapPlotPanelBinding",
        "HeatmapPlotPanelBinding::new(app,HeatmapPlotModel::new(data_bounds,cols,rows,values),)",
        "HeatmapPlotModel::new(data_bounds,cols,rows,values)",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"heatmap-demo\"",
        "plot.panel_props()",
        ".style(style)",
        "state.plot.output_untracked(app)",
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
        "fret_runtime::Model<",
        "HeatmapPlotPanelProps::new(",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "app.models_mut().insert(PlotState::default())",
        "app.models_mut().insert(PlotOutput::default())",
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
        "usefret_plot::Histogram2DPlotPanelBinding;",
        "usefret_plot::declarative::histogram2d_plot_panel_in;",
        "usefret_plot::models::Histogram2DPlotModel;",
        "usefret_ui::{UiTree,declarative};",
        "plot:Histogram2DPlotPanelBinding",
        "histogram2d_counts(Histogram2DConfig::new(bounds,256,192),points)",
        "Histogram2DPlotModel::new(grid.data_bounds,grid.cols,grid.rows,grid.values)",
        "Histogram2DPlotPanelBinding::new(app,model)",
        "declarative::RenderRootContext::new(&mutstate.ui,app,services,window,bounds).render_root(\"histogram2d-demo\"",
        "plot.panel_props()",
        ".x_axis_labels(AxisLabelFormatter::number(AxisNumberFormat::Fixed(2)))",
        ".y_axis_labels(AxisLabelFormatter::number(AxisNumberFormat::Fixed(2)))",
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
        "fret_runtime::Model<",
        "Histogram2DPlotPanelProps::new(",
        ".state(plot_state.clone())",
        ".output(plot_output.clone())",
        "app.models_mut().insert(PlotState::default())",
        "app.models_mut().insert(PlotOutput::default())",
    ] {
        assert!(
            !source.contains(legacy),
            "histogram2d_demo should not teach retained plot authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn bars_demo_uses_manual_harness_declarative_chart_canvas_panel_binding() {
    let source = compact(include_str!("../src/bars_demo.rs"));

    for needle in [
        "usefret_chart::{ChartCanvasPanelBinding,chart_canvas_panel};",
        "chart:ChartCanvasPanelBinding",
        "fnbuild_chart()->(ChartEngine,ChartSpec)",
        "letchart=ChartCanvasPanelBinding::new(app,spec,engine);",
        "state.chart.output_untracked(app)",
        "declarative::render_root(",
        "\"bars-demo-root\"",
        "chart.observe_engine_paint(cx);",
        "letprops=chart.panel_props();",
        "vec![chart_canvas_panel(cx,props)]",
    ] {
        assert!(
            source.contains(needle),
            "bars_demo manual harness should use declarative chart canvas binding authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_chart::{ChartCanvasPanelProps,chart_canvas_panel};",
        "usefret_chart::retained",
        "fret_chart::retained::",
        "fret_runtime::Model<",
        "engine:Model<ChartEngine>",
        "output:Model<ChartCanvasOutput>",
        "app.models_mut().insert(engine)",
        "app.models_mut().insert(ChartEngine::",
        "ChartCanvasPanelProps::new(",
        "props.engine=Some(engine);",
        ".output_model(",
        "cx.observe_model(&engine",
        "ChartCanvas::new(",
        "ChartCanvas::create_node(",
        "create_node_retained(",
    ] {
        assert!(
            !source.contains(legacy),
            "bars_demo should not teach retained/manual chart canvas authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn category_line_demo_uses_manual_harness_declarative_chart_canvas_panel_binding() {
    let source = compact(include_str!("../src/category_line_demo.rs"));

    for needle in [
        "usefret_chart::{ChartCanvasPanelBinding,chart_canvas_panel};",
        "chart:ChartCanvasPanelBinding",
        "fnbuild_chart()->(ChartEngine,ChartSpec)",
        "AxisScale::Category(delinea::CategoryAxisScale{categories})",
        "data_zoom_x:vec![DataZoomXSpec{",
        "engine.apply_action(Action::SetDataWindowX{",
        "window:Some(DataWindow{min:16.0,max:64.0,})",
        "letchart=ChartCanvasPanelBinding::new(app,spec,engine);",
        "fret_ui::declarative::render_root(",
        "\"category-line-demo-root\"",
        "chart.observe_engine_paint(cx);",
        "letprops=chart.panel_props();",
        "vec![chart_canvas_panel(cx,props)]",
    ] {
        assert!(
            source.contains(needle),
            "category_line_demo manual harness should use declarative chart canvas binding authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_chart::{ChartCanvasPanelProps,chart_canvas_panel};",
        "usefret_chart::retained",
        "fret_chart::retained::",
        "fret_runtime::Model<",
        "engine:Model<ChartEngine>",
        "output:Model<ChartCanvasOutput>",
        "app.models_mut().insert(engine)",
        "app.models_mut().insert(ChartEngine::",
        "ChartCanvasPanelProps::new(",
        "props.engine=Some(engine);",
        ".output_model(",
        "cx.observe_model(&engine",
        "ChartCanvas::new(",
        "ChartCanvas::create_node(",
        "create_node_retained(",
    ] {
        assert!(
            !source.contains(legacy),
            "category_line_demo should not teach retained/manual chart canvas authoring; unexpected `{legacy}`"
        );
    }
}

#[test]
fn horizontal_bars_demo_uses_manual_harness_declarative_chart_canvas_panel_binding() {
    let source = compact(include_str!("../src/horizontal_bars_demo.rs"));

    for needle in [
        "usefret_chart::{ChartCanvasPanelBinding,chart_canvas_panel};",
        "chart:ChartCanvasPanelBinding",
        "fnbuild_chart()->(ChartEngine,ChartSpec)",
        "AxisScale::Category(delinea::CategoryAxisScale{categories})",
        "visual_maps:vec![VisualMapSpec{",
        "mode:VisualMapMode::Continuous",
        "opacity_mul_range:Some((0.2,1.0))",
        "stack:Some(stack_id)",
        "letchart=ChartCanvasPanelBinding::new(app,spec,engine);",
        "fret_ui::declarative::render_root(",
        "\"horizontal-bars-demo-root\"",
        "chart.observe_engine_paint(cx);",
        "letprops=chart.panel_props();",
        "vec![chart_canvas_panel(cx,props)]",
    ] {
        assert!(
            source.contains(needle),
            "horizontal_bars_demo manual harness should use declarative chart canvas binding authoring; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_chart::{ChartCanvasPanelProps,chart_canvas_panel};",
        "usefret_chart::retained",
        "fret_chart::retained::",
        "fret_runtime::Model<",
        "engine:Model<ChartEngine>",
        "output:Model<ChartCanvasOutput>",
        "app.models_mut().insert(engine)",
        "app.models_mut().insert(ChartEngine::",
        "ChartCanvasPanelProps::new(",
        "props.engine=Some(engine);",
        ".output_model(",
        "cx.observe_model(&engine",
        "ChartCanvas::new(",
        "ChartCanvas::create_node(",
        "create_node_retained(",
    ] {
        assert!(
            !source.contains(legacy),
            "horizontal_bars_demo should not teach retained/manual chart canvas authoring; unexpected `{legacy}`"
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

#[test]
fn fret_examples_wasm_target_enables_fret_ui_assets_for_asset_demos() {
    let manifest = compact(include_str!("../Cargo.toml"));

    assert!(
        manifest.contains("fret={path=\"../../ecosystem/fret\",default-features=false,features=[\"app\",\"state\",\"ui-assets\"]}"),
        "fret-examples wasm target should enable fret/ui-assets because wasm-compiled demos import fret::app::ui_assets"
    );
}
