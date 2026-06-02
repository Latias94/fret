//! Declarative line-plot error-bars series paint owner.

use fret_core::{DrawOrder, PathStyle, Point, Px, StrokeStyle};
use fret_ui::canvas::CanvasPainter;

use crate::cartesian::PlotTransform;
use crate::series::SeriesId;
use crate::style::LinePlotStyle;

use super::super::commands::{error_bars_commands_from_series, line_plot_series_path_key};
use super::super::model::{PlotPanelErrorBars, PlotPanelSeries};
use super::super::style_helpers::series_color;

pub(super) fn paint_line_plot_error_bars_series(
    painter: &mut CanvasPainter<'_>,
    series: &PlotPanelSeries,
    error_bars: &PlotPanelErrorBars,
    series_transform: PlotTransform,
    style: LinePlotStyle,
    series_index: usize,
    series_count: usize,
    emphasized_series: Option<SeriesId>,
    raster_scale_factor: f32,
) {
    let commands = error_bars_commands_from_series(series_transform, &*series.data, error_bars);
    if commands.len() < 2 {
        return;
    }

    let mut stroke_color = series
        .stroke_color
        .unwrap_or_else(|| series_color(style, series_index, series_count));
    if let Some(emphasized) = emphasized_series
        && series.id != emphasized
    {
        stroke_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
    }
    let stroke_width = series.stroke_width.unwrap_or(style.stroke_width);
    painter.path(
        line_plot_series_path_key(series.id.0),
        DrawOrder(20),
        Point::new(Px(0.0), Px(0.0)),
        &commands,
        PathStyle::Stroke(StrokeStyle {
            width: stroke_width,
        }),
        stroke_color,
        raster_scale_factor,
    );
}
