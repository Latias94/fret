//! Public declarative plot panel entrypoints.

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, ElementContextAccess, UiHost};

use super::{
    AreaPlotPanelProps, BarsPlotPanelProps, CandlestickPlotPanelProps, ErrorBarsPlotPanelProps,
    HeatmapPlotPanelProps, Histogram2DPlotPanelProps, HistogramPlotPanelProps, LinePlotPanelProps,
    PlotPanelModel, PlotPanelProps, ShadedPlotPanelProps, StemsPlotPanelProps, plot_panel,
};
#[track_caller]
pub fn error_bars_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: ErrorBarsPlotPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);
    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("error bars plot model should exist");
    plot_panel(
        cx,
        PlotPanelProps {
            canvas: props.canvas,
            model: PlotPanelModel::from(&model),
            state: props.state,
            output: props.output,
            style: props.style,
            x_axis_labels: props.x_axis_labels,
            y_axis_labels: props.y_axis_labels,
            y2_axis_labels: props.y2_axis_labels,
            y3_axis_labels: props.y3_axis_labels,
            y4_axis_labels: props.y4_axis_labels,
            x_scale: props.x_scale,
            y_scale: props.y_scale,
            step_mode: props.step_mode,
        },
    )
}

#[track_caller]
pub fn histogram_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: HistogramPlotPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);
    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("histogram plot model should exist");
    plot_panel(
        cx,
        PlotPanelProps {
            canvas: props.canvas,
            model: PlotPanelModel::from(&model),
            state: props.state,
            output: props.output,
            style: props.style,
            x_axis_labels: props.x_axis_labels,
            y_axis_labels: props.y_axis_labels,
            y2_axis_labels: props.y2_axis_labels,
            y3_axis_labels: props.y3_axis_labels,
            y4_axis_labels: props.y4_axis_labels,
            x_scale: props.x_scale,
            y_scale: props.y_scale,
            step_mode: props.step_mode,
        },
    )
}

#[track_caller]
pub fn bars_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: BarsPlotPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);
    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("bars plot model should exist");
    plot_panel(
        cx,
        PlotPanelProps {
            canvas: props.canvas,
            model: PlotPanelModel::from(&model),
            state: props.state,
            output: props.output,
            style: props.style,
            x_axis_labels: props.x_axis_labels,
            y_axis_labels: props.y_axis_labels,
            y2_axis_labels: props.y2_axis_labels,
            y3_axis_labels: props.y3_axis_labels,
            y4_axis_labels: props.y4_axis_labels,
            x_scale: props.x_scale,
            y_scale: props.y_scale,
            step_mode: props.step_mode,
        },
    )
}

#[track_caller]
pub fn candlestick_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: CandlestickPlotPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);
    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("candlestick plot model should exist");
    plot_panel(
        cx,
        PlotPanelProps {
            canvas: props.canvas,
            model: PlotPanelModel::from(&model),
            state: props.state,
            output: props.output,
            style: props.style,
            x_axis_labels: props.x_axis_labels,
            y_axis_labels: props.y_axis_labels,
            y2_axis_labels: props.y2_axis_labels,
            y3_axis_labels: props.y3_axis_labels,
            y4_axis_labels: props.y4_axis_labels,
            x_scale: props.x_scale,
            y_scale: props.y_scale,
            step_mode: props.step_mode,
        },
    )
}

#[track_caller]
pub fn heatmap_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: HeatmapPlotPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);
    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("heatmap plot model should exist");
    plot_panel(
        cx,
        PlotPanelProps {
            canvas: props.canvas,
            model: PlotPanelModel::from(&model),
            state: props.state,
            output: props.output,
            style: props.style,
            x_axis_labels: props.x_axis_labels,
            y_axis_labels: props.y_axis_labels,
            y2_axis_labels: props.y2_axis_labels,
            y3_axis_labels: props.y3_axis_labels,
            y4_axis_labels: props.y4_axis_labels,
            x_scale: props.x_scale,
            y_scale: props.y_scale,
            step_mode: props.step_mode,
        },
    )
}

#[track_caller]
pub fn histogram2d_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: Histogram2DPlotPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);
    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("histogram2d plot model should exist");
    plot_panel(
        cx,
        PlotPanelProps {
            canvas: props.canvas,
            model: PlotPanelModel::from(&model),
            state: props.state,
            output: props.output,
            style: props.style,
            x_axis_labels: props.x_axis_labels,
            y_axis_labels: props.y_axis_labels,
            y2_axis_labels: props.y2_axis_labels,
            y3_axis_labels: props.y3_axis_labels,
            y4_axis_labels: props.y4_axis_labels,
            x_scale: props.x_scale,
            y_scale: props.y_scale,
            step_mode: props.step_mode,
        },
    )
}

#[track_caller]
pub fn line_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: LinePlotPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);
    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("line plot model should exist");
    plot_panel(
        cx,
        PlotPanelProps {
            canvas: props.canvas,
            model: PlotPanelModel::from(&model),
            state: props.state,
            output: props.output,
            style: props.style,
            x_axis_labels: props.x_axis_labels,
            y_axis_labels: props.y_axis_labels,
            y2_axis_labels: props.y2_axis_labels,
            y3_axis_labels: props.y3_axis_labels,
            y4_axis_labels: props.y4_axis_labels,
            x_scale: props.x_scale,
            y_scale: props.y_scale,
            step_mode: props.step_mode,
        },
    )
}

#[track_caller]
pub fn area_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: AreaPlotPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);
    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("area plot model should exist");
    plot_panel(
        cx,
        PlotPanelProps {
            canvas: props.canvas,
            model: PlotPanelModel::from(&model),
            state: props.state,
            output: props.output,
            style: props.style,
            x_axis_labels: props.x_axis_labels,
            y_axis_labels: props.y_axis_labels,
            y2_axis_labels: props.y2_axis_labels,
            y3_axis_labels: props.y3_axis_labels,
            y4_axis_labels: props.y4_axis_labels,
            x_scale: props.x_scale,
            y_scale: props.y_scale,
            step_mode: props.step_mode,
        },
    )
}

#[track_caller]
pub fn shaded_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: ShadedPlotPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);
    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("shaded plot model should exist");
    plot_panel(
        cx,
        PlotPanelProps {
            canvas: props.canvas,
            model: PlotPanelModel::from(&model),
            state: props.state,
            output: props.output,
            style: props.style,
            x_axis_labels: props.x_axis_labels,
            y_axis_labels: props.y_axis_labels,
            y2_axis_labels: props.y2_axis_labels,
            y3_axis_labels: props.y3_axis_labels,
            y4_axis_labels: props.y4_axis_labels,
            x_scale: props.x_scale,
            y_scale: props.y_scale,
            step_mode: props.step_mode,
        },
    )
}

#[track_caller]
pub fn stems_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: StemsPlotPanelProps,
) -> AnyElement {
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);
    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("stems plot model should exist");
    plot_panel(
        cx,
        PlotPanelProps {
            canvas: props.canvas,
            model: PlotPanelModel::from(&model),
            state: props.state,
            output: props.output,
            style: props.style,
            x_axis_labels: props.x_axis_labels,
            y_axis_labels: props.y_axis_labels,
            y2_axis_labels: props.y2_axis_labels,
            y3_axis_labels: props.y3_axis_labels,
            y4_axis_labels: props.y4_axis_labels,
            x_scale: props.x_scale,
            y_scale: props.y_scale,
            step_mode: props.step_mode,
        },
    )
}

/// Capability-first adapter for [`error_bars_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn error_bars_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: ErrorBarsPlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    error_bars_plot_panel(cx.elements(), props)
}

/// Capability-first adapter for [`histogram_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn histogram_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: HistogramPlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    histogram_plot_panel(cx.elements(), props)
}

/// Capability-first adapter for [`bars_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn bars_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: BarsPlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    bars_plot_panel(cx.elements(), props)
}

/// Capability-first adapter for [`candlestick_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn candlestick_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: CandlestickPlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    candlestick_plot_panel(cx.elements(), props)
}

/// Capability-first adapter for [`heatmap_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn heatmap_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: HeatmapPlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    heatmap_plot_panel(cx.elements(), props)
}

/// Capability-first adapter for [`histogram2d_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn histogram2d_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: Histogram2DPlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    histogram2d_plot_panel(cx.elements(), props)
}

/// Capability-first adapter for [`line_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn line_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: LinePlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    line_plot_panel(cx.elements(), props)
}

/// Capability-first adapter for [`area_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn area_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: AreaPlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    area_plot_panel(cx.elements(), props)
}

/// Capability-first adapter for [`shaded_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn shaded_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: ShadedPlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    shaded_plot_panel(cx.elements(), props)
}

/// Capability-first adapter for [`stems_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn stems_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: StemsPlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    stems_plot_panel(cx.elements(), props)
}
