//! Declarative line-plot bar and histogram series paint owner.

use fret_core::{DrawOrder, PathStyle, Point, Px};
use fret_ui::canvas::CanvasPainter;

use crate::cartesian::PlotTransform;
use crate::series::SeriesId;
use crate::style::LinePlotStyle;

use super::super::commands::{
    bars_commands_from_series, histogram_commands_from_series, line_plot_area_fill_path_key,
};
use super::super::model::{PlotPanelBars, PlotPanelHistogram, PlotPanelSeries};
use super::super::style_helpers::series_color;

pub(super) fn paint_line_plot_bars_series(
    painter: &mut CanvasPainter<'_>,
    series: &PlotPanelSeries,
    bars: &PlotPanelBars,
    series_transform: PlotTransform,
    style: LinePlotStyle,
    series_index: usize,
    series_count: usize,
    emphasized_series: Option<SeriesId>,
    raster_scale_factor: f32,
) {
    let commands = bars_commands_from_series(series_transform, &*series.data, bars);
    if commands.is_empty() {
        return;
    }
    paint_filled_series_path(
        painter,
        series,
        &commands,
        style,
        series_index,
        series_count,
        emphasized_series,
        raster_scale_factor,
    );
}

pub(super) fn paint_line_plot_histogram_series(
    painter: &mut CanvasPainter<'_>,
    series: &PlotPanelSeries,
    histogram: &PlotPanelHistogram,
    series_transform: PlotTransform,
    style: LinePlotStyle,
    series_index: usize,
    series_count: usize,
    emphasized_series: Option<SeriesId>,
    raster_scale_factor: f32,
) {
    let commands = histogram_commands_from_series(series_transform, &*series.data, histogram);
    if commands.is_empty() {
        return;
    }
    paint_filled_series_path(
        painter,
        series,
        &commands,
        style,
        series_index,
        series_count,
        emphasized_series,
        raster_scale_factor,
    );
}

fn paint_filled_series_path(
    painter: &mut CanvasPainter<'_>,
    series: &PlotPanelSeries,
    commands: &[fret_core::PathCommand],
    style: LinePlotStyle,
    series_index: usize,
    series_count: usize,
    emphasized_series: Option<SeriesId>,
    raster_scale_factor: f32,
) {
    let mut fill_color = series
        .stroke_color
        .unwrap_or_else(|| series_color(style, series_index, series_count));
    if let Some(emphasized) = emphasized_series
        && series.id != emphasized
    {
        fill_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
    }
    painter.path(
        line_plot_area_fill_path_key(series.id.0),
        DrawOrder(19),
        Point::new(Px(0.0), Px(0.0)),
        commands,
        PathStyle::Fill(fret_core::FillStyle::default()),
        fill_color,
        raster_scale_factor,
    );
}
