//! Declarative line-plot shaded series paint owner.

use fret_core::{DrawOrder, PathStyle, Point, Px, StrokeStyle};
use fret_ui::canvas::CanvasPainter;

use crate::cartesian::PlotTransform;
use crate::series::{Series, SeriesId};
use crate::style::LinePlotStyle;

use super::super::commands::{
    line_plot_area_fill_path_key, line_plot_series_path_key, line_plot_shaded_lower_path_key,
    shaded_band_commands_from_series,
};
use super::super::model::PlotPanelSeries;
use super::super::style_helpers::series_color;

pub(super) fn paint_line_plot_shaded_series(
    painter: &mut CanvasPainter<'_>,
    series: &PlotPanelSeries,
    lower_data: &Series,
    series_transform: PlotTransform,
    style: LinePlotStyle,
    series_index: usize,
    series_count: usize,
    emphasized_series: Option<SeriesId>,
    raster_scale_factor: f32,
) {
    let (fill_commands, upper_commands, lower_commands) =
        shaded_band_commands_from_series(series_transform, &*series.data, &**lower_data);

    let mut stroke_color = series
        .stroke_color
        .unwrap_or_else(|| series_color(style, series_index, series_count));
    if let Some(emphasized) = emphasized_series
        && series.id != emphasized
    {
        stroke_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
    }

    if let Some(fill) = series.fill
        && !fill_commands.is_empty()
    {
        let mut fill_color = fill.color.unwrap_or_else(|| {
            series
                .stroke_color
                .unwrap_or_else(|| series_color(style, series_index, series_count))
        });
        fill_color.a = (fill_color.a * fill.alpha.clamp(0.0, 1.0)).clamp(0.0, 1.0);
        if let Some(emphasized) = emphasized_series
            && series.id != emphasized
        {
            fill_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
        }
        painter.path(
            line_plot_area_fill_path_key(series.id.0),
            DrawOrder(19),
            Point::new(Px(0.0), Px(0.0)),
            &fill_commands,
            PathStyle::Fill(fret_core::FillStyle::default()),
            fill_color,
            raster_scale_factor,
        );
    }

    let stroke_width = series.stroke_width.unwrap_or(style.stroke_width);
    if upper_commands.len() >= 2 {
        painter.path(
            line_plot_series_path_key(series.id.0),
            DrawOrder(20),
            Point::new(Px(0.0), Px(0.0)),
            &upper_commands,
            PathStyle::Stroke(StrokeStyle {
                width: stroke_width,
            }),
            stroke_color,
            raster_scale_factor,
        );
    }
    if lower_commands.len() >= 2 {
        painter.path(
            line_plot_shaded_lower_path_key(series.id.0),
            DrawOrder(20),
            Point::new(Px(0.0), Px(0.0)),
            &lower_commands,
            PathStyle::Stroke(StrokeStyle {
                width: stroke_width,
            }),
            stroke_color,
            raster_scale_factor,
        );
    }
}
