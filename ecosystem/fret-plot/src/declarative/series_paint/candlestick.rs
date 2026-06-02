//! Declarative line-plot candlestick series paint owner.

use fret_core::{Color, DrawOrder, PathStyle, Point, Px, StrokeStyle};
use fret_ui::canvas::CanvasPainter;

use crate::cartesian::PlotTransform;
use crate::series::SeriesId;
use crate::style::LinePlotStyle;

use super::super::commands::{
    candlestick_commands_from_series, line_plot_area_fill_path_key,
    line_plot_candlestick_down_path_key, line_plot_series_path_key,
};
use super::super::model::{PlotPanelCandlestick, PlotPanelSeries};
use super::super::style_helpers::series_color;

pub(super) fn paint_line_plot_candlestick_series(
    painter: &mut CanvasPainter<'_>,
    series: &PlotPanelSeries,
    candlestick: &PlotPanelCandlestick,
    series_transform: PlotTransform,
    style: LinePlotStyle,
    series_index: usize,
    series_count: usize,
    emphasized_series: Option<SeriesId>,
    raster_scale_factor: f32,
) {
    let stroke_width = series.stroke_width.unwrap_or(style.stroke_width);
    let (wick_commands, up_body_commands, down_body_commands) = candlestick_commands_from_series(
        series_transform,
        candlestick,
        stroke_width,
        raster_scale_factor,
    );
    if wick_commands.is_empty() && up_body_commands.is_empty() && down_body_commands.is_empty() {
        return;
    }

    let mut wick_color = candlestick
        .wick_color
        .or(series.stroke_color)
        .unwrap_or_else(|| series_color(style, series_index, series_count));
    let mut up_fill = candlestick.up_fill.unwrap_or(Color {
        r: 0.25,
        g: 0.80,
        b: 0.45,
        a: 0.85,
    });
    let mut down_fill = candlestick.down_fill.unwrap_or(Color {
        r: 0.90,
        g: 0.35,
        b: 0.45,
        a: 0.85,
    });
    if let Some(emphasized) = emphasized_series
        && series.id != emphasized
    {
        let dim = style.dimmed_series_alpha.clamp(0.0, 1.0);
        wick_color.a *= dim;
        up_fill.a *= dim;
        down_fill.a *= dim;
    }

    if !up_body_commands.is_empty() {
        painter.path(
            line_plot_area_fill_path_key(series.id.0),
            DrawOrder(19),
            Point::new(Px(0.0), Px(0.0)),
            &up_body_commands,
            PathStyle::Fill(fret_core::FillStyle::default()),
            up_fill,
            raster_scale_factor,
        );
    }
    if !down_body_commands.is_empty() {
        painter.path(
            line_plot_candlestick_down_path_key(series.id.0),
            DrawOrder(19),
            Point::new(Px(0.0), Px(0.0)),
            &down_body_commands,
            PathStyle::Fill(fret_core::FillStyle::default()),
            down_fill,
            raster_scale_factor,
        );
    }
    if wick_commands.len() >= 2 {
        painter.path(
            line_plot_series_path_key(series.id.0),
            DrawOrder(20),
            Point::new(Px(0.0), Px(0.0)),
            &wick_commands,
            PathStyle::Stroke(StrokeStyle {
                width: stroke_width,
            }),
            wick_color,
            raster_scale_factor,
        );
    }
}
