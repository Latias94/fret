//! Declarative line-plot series paint owner.

use fret_core::{DrawOrder, PathStyle, Point, Px, StrokeStyle};
use fret_ui::canvas::CanvasPainter;

use crate::cartesian::{PlotTransform, polyline_commands};
use crate::models::{StepMode, YAxis};
use crate::series::SeriesId;
use crate::style::LinePlotStyle;

use super::commands::{
    area_fill_commands_from_polyline, error_bars_commands_from_series,
    line_plot_area_fill_path_key, line_plot_series_path_key, line_plot_shaded_lower_path_key,
    shaded_band_commands_from_series, stems_commands_from_points, step_commands_from_polyline,
};
use super::geometry::line_plot_view_bounds_for_y_axis;
use super::model::PlotPanelModel;
use super::style_helpers::series_color;

mod bar_histogram;
mod candlestick;

use bar_histogram::{paint_line_plot_bars_series, paint_line_plot_histogram_series};
use candlestick::paint_line_plot_candlestick_series;

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
            let commands =
                error_bars_commands_from_series(series_transform, &*series.data, error_bars);
            if commands.len() < 2 {
                continue;
            }

            let mut stroke_color = series
                .stroke_color
                .unwrap_or_else(|| series_color(style, index, series_count));
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
            continue;
        }
        if let Some(lower_data) = &series.lower_data {
            let (fill_commands, upper_commands, lower_commands) =
                shaded_band_commands_from_series(series_transform, &*series.data, &**lower_data);

            let mut stroke_color = series
                .stroke_color
                .unwrap_or_else(|| series_color(style, index, series_count));
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
                        .unwrap_or_else(|| series_color(style, index, series_count))
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
            continue;
        }
        let Some(points) = series.data.as_slice() else {
            continue;
        };
        let commands = if let Some(baseline) = series.stem_baseline {
            stems_commands_from_points(series_transform, points, baseline)
        } else {
            let commands = polyline_commands(series_transform, points);
            if let Some(step_mode) = step_mode {
                step_commands_from_polyline(&commands, step_mode)
            } else {
                commands
            }
        };
        if commands.len() < 2 {
            continue;
        }

        let mut stroke_color = series
            .stroke_color
            .unwrap_or_else(|| series_color(style, index, series_count));
        if let Some(emphasized) = emphasized_series
            && series.id != emphasized
        {
            stroke_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
        }
        if let Some(fill) = series.fill
            && let Some(baseline_y) = series_transform.data_y_to_px(f64::from(fill.baseline))
        {
            let fill_commands = area_fill_commands_from_polyline(&commands, baseline_y);
            if !fill_commands.is_empty() {
                let mut fill_color = fill.color.unwrap_or_else(|| {
                    series
                        .stroke_color
                        .unwrap_or_else(|| series_color(style, index, series_count))
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
}
