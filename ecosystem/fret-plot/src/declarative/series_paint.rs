//! Declarative line-plot series paint router.

use fret_ui::canvas::CanvasPainter;

use crate::cartesian::PlotTransform;
use crate::models::{StepMode, YAxis};
use crate::series::SeriesId;
use crate::style::LinePlotStyle;

use super::geometry::line_plot_view_bounds_for_y_axis;
use super::model::PlotPanelModel;

mod bar_histogram;
mod candlestick;
mod error_bars;
mod line_area;
mod shaded;

use bar_histogram::{paint_line_plot_bars_series, paint_line_plot_histogram_series};
use candlestick::paint_line_plot_candlestick_series;
use error_bars::paint_line_plot_error_bars_series;
use line_area::paint_line_plot_line_area_series;
use shaded::paint_line_plot_shaded_series;

pub(super) fn paint_line_plot_series(
    painter: &mut CanvasPainter<'_>,
    model: &PlotPanelModel,
    transform: PlotTransform,
    hidden_series: &[SeriesId],
    step_mode: Option<StepMode>,
    style: LinePlotStyle,
    pinned_series: Option<SeriesId>,
    legend_hover: Option<SeriesId>,
) {
    let series_count = model.series.len();
    let raster_scale_factor = painter.scale_factor();
    let emphasized_series = if style.emphasize_hovered_series {
        pinned_series.or(legend_hover)
    } else {
        None
    };
    let right_transform = model.data_bounds_y2.map(|axis_bounds| PlotTransform {
        viewport: transform.viewport,
        data: line_plot_view_bounds_for_y_axis(transform.data, axis_bounds),
        x_scale: transform.x_scale,
        y_scale: transform.y_scale,
    });
    let right2_transform = model.data_bounds_y3.map(|axis_bounds| PlotTransform {
        viewport: transform.viewport,
        data: line_plot_view_bounds_for_y_axis(transform.data, axis_bounds),
        x_scale: transform.x_scale,
        y_scale: transform.y_scale,
    });
    let right3_transform = model.data_bounds_y4.map(|axis_bounds| PlotTransform {
        viewport: transform.viewport,
        data: line_plot_view_bounds_for_y_axis(transform.data, axis_bounds),
        x_scale: transform.x_scale,
        y_scale: transform.y_scale,
    });

    for (index, series) in model.series.iter().enumerate() {
        if hidden_series.contains(&series.id) {
            continue;
        }
        let series_transform = match series.y_axis {
            YAxis::Left => transform,
            YAxis::Right => right_transform.unwrap_or(transform),
            YAxis::Right2 => right2_transform.unwrap_or(transform),
            YAxis::Right3 => right3_transform.unwrap_or(transform),
        };
        if let Some(candlestick) = &series.candlestick {
            paint_line_plot_candlestick_series(
                painter,
                series,
                candlestick,
                series_transform,
                style,
                index,
                series_count,
                emphasized_series,
                raster_scale_factor,
            );
            continue;
        }
        if let Some(bars) = &series.bars {
            paint_line_plot_bars_series(
                painter,
                series,
                bars,
                series_transform,
                style,
                index,
                series_count,
                emphasized_series,
                raster_scale_factor,
            );
            continue;
        }
        if let Some(histogram) = &series.histogram {
            paint_line_plot_histogram_series(
                painter,
                series,
                histogram,
                series_transform,
                style,
                index,
                series_count,
                emphasized_series,
                raster_scale_factor,
            );
            continue;
        }
        if let Some(error_bars) = &series.error_bars {
            paint_line_plot_error_bars_series(
                painter,
                series,
                error_bars,
                series_transform,
                style,
                index,
                series_count,
                emphasized_series,
                raster_scale_factor,
            );
            continue;
        }
        if let Some(lower_data) = &series.lower_data {
            paint_line_plot_shaded_series(
                painter,
                series,
                lower_data,
                series_transform,
                style,
                index,
                series_count,
                emphasized_series,
                raster_scale_factor,
            );
            continue;
        }
        paint_line_plot_line_area_series(
            painter,
            series,
            series_transform,
            step_mode,
            style,
            index,
            series_count,
            emphasized_series,
            raster_scale_factor,
        );
    }
}
