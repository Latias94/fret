//! Immediate-mode (`UiWriter`) adapters for `fret-plot`.
//!
//! This module is intentionally tiny: `fret-plot` remains a declarative plot crate, while this
//! feature only adds ergonomic glue for authoring frontends that implement `UiWriter`.

use fret_authoring::UiWriter;
use fret_ui::UiHost;

use crate::declarative::{
    AreaPlotPanelProps, BarsPlotPanelProps, CandlestickPlotPanelProps, ErrorBarsPlotPanelProps,
    HeatmapPlotPanelProps, Histogram2DPlotPanelProps, HistogramPlotPanelProps, LinePlotPanelProps,
    ShadedPlotPanelProps, StemsPlotPanelProps,
};

/// Adds a line plot panel to an `imui` output list.
#[track_caller]
pub fn line_plot_panel<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, props: LinePlotPanelProps) {
    let element = ui.with_cx_mut(|cx| crate::declarative::line_plot_panel(cx, props));
    ui.add(element);
}

/// Adds an error-bars plot panel to an `imui` output list.
#[track_caller]
pub fn error_bars_plot_panel<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    props: ErrorBarsPlotPanelProps,
) {
    let element = ui.with_cx_mut(|cx| crate::declarative::error_bars_plot_panel(cx, props));
    ui.add(element);
}

/// Adds a histogram plot panel to an `imui` output list.
#[track_caller]
pub fn histogram_plot_panel<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    props: HistogramPlotPanelProps,
) {
    let element = ui.with_cx_mut(|cx| crate::declarative::histogram_plot_panel(cx, props));
    ui.add(element);
}

/// Adds a bars plot panel to an `imui` output list.
#[track_caller]
pub fn bars_plot_panel<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, props: BarsPlotPanelProps) {
    let element = ui.with_cx_mut(|cx| crate::declarative::bars_plot_panel(cx, props));
    ui.add(element);
}

/// Adds a candlestick plot panel to an `imui` output list.
#[track_caller]
pub fn candlestick_plot_panel<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    props: CandlestickPlotPanelProps,
) {
    let element = ui.with_cx_mut(|cx| crate::declarative::candlestick_plot_panel(cx, props));
    ui.add(element);
}

/// Adds a heatmap plot panel to an `imui` output list.
#[track_caller]
pub fn heatmap_plot_panel<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    props: HeatmapPlotPanelProps,
) {
    let element = ui.with_cx_mut(|cx| crate::declarative::heatmap_plot_panel(cx, props));
    ui.add(element);
}

/// Adds a 2D histogram plot panel to an `imui` output list.
#[track_caller]
pub fn histogram2d_plot_panel<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    props: Histogram2DPlotPanelProps,
) {
    let element = ui.with_cx_mut(|cx| crate::declarative::histogram2d_plot_panel(cx, props));
    ui.add(element);
}

/// Adds an area plot panel to an `imui` output list.
#[track_caller]
pub fn area_plot_panel<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, props: AreaPlotPanelProps) {
    let element = ui.with_cx_mut(|cx| crate::declarative::area_plot_panel(cx, props));
    ui.add(element);
}

/// Adds a shaded plot panel to an `imui` output list.
#[track_caller]
pub fn shaded_plot_panel<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    props: ShadedPlotPanelProps,
) {
    let element = ui.with_cx_mut(|cx| crate::declarative::shaded_plot_panel(cx, props));
    ui.add(element);
}

/// Adds a stems plot panel to an `imui` output list.
#[track_caller]
pub fn stems_plot_panel<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    props: StemsPlotPanelProps,
) {
    let element = ui.with_cx_mut(|cx| crate::declarative::stems_plot_panel(cx, props));
    ui.add(element);
}
