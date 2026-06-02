//! Declarative line-plot line, area, and stems series paint owner.

use fret_core::{DrawOrder, PathStyle, Point, Px, StrokeStyle};
use fret_ui::canvas::CanvasPainter;

use crate::cartesian::{PlotTransform, polyline_commands};
use crate::models::StepMode;
use crate::series::SeriesId;
use crate::style::LinePlotStyle;

use super::super::commands::{
    area_fill_commands_from_polyline, line_plot_area_fill_path_key, line_plot_series_path_key,
    stems_commands_from_points, step_commands_from_polyline,
};
use super::super::model::PlotPanelSeries;
use super::super::style_helpers::series_color;

pub(super) fn paint_line_plot_line_area_series(
    painter: &mut CanvasPainter<'_>,
    series: &PlotPanelSeries,
    series_transform: PlotTransform,
    step_mode: Option<StepMode>,
    style: LinePlotStyle,
    series_index: usize,
    series_count: usize,
    emphasized_series: Option<SeriesId>,
    raster_scale_factor: f32,
) {
    let Some(points) = series.data.as_slice() else {
        return;
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
    if let Some(fill) = series.fill
        && let Some(baseline_y) = series_transform.data_y_to_px(f64::from(fill.baseline))
    {
        let fill_commands = area_fill_commands_from_polyline(&commands, baseline_y);
        if !fill_commands.is_empty() {
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
