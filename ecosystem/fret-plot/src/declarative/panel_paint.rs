//! Declarative line-plot panel paint orchestration owner.

use fret_core::{Color, Corners, DrawOrder, Edges, Paint, PathStyle, Point, Px, StrokeStyle};
use fret_ui::canvas::CanvasPainter;

use crate::cartesian::{AxisScale, DataRect, PlotTransform, polyline_commands};
use crate::models::{StepMode, YAxis};
use crate::plot::axis::AxisLabelFormatter;
use crate::series::SeriesId;
use crate::state::{PlotImageLayer, PlotOutputSnapshot, PlotOverlays};
use crate::style::LinePlotStyle;

use super::axis_labels::paint_line_plot_right_axis_tick_labels;
use super::commands::{
    area_fill_commands_from_polyline, bars_commands_from_series, candlestick_commands_from_series,
    error_bars_commands_from_series, histogram_commands_from_series, line_plot_area_fill_path_key,
    line_plot_candlestick_down_path_key, line_plot_series_path_key,
    line_plot_shaded_lower_path_key, shaded_band_commands_from_series, stems_commands_from_points,
    step_commands_from_polyline,
};
use super::geometry::{line_plot_inner_rect, line_plot_view_bounds_for_y_axis};
use super::grid_axes::paint_line_plot_grid_and_axes;
use super::heatmap::{paint_line_plot_heatmap, paint_line_plot_heatmap_colorbar};
use super::interaction::LinePlotSelectionOverlay;
use super::legend::paint_line_plot_legend;
use super::model::PlotPanelModel;
use super::overlays::{
    paint_line_plot_draggable_overlay_labels, paint_line_plot_draggable_shapes,
    paint_line_plot_images, paint_line_plot_reference_lines, paint_line_plot_tag_overlays,
    paint_line_plot_text_overlays,
};
use super::readout::{paint_line_plot_cursor_readout, paint_line_plot_linked_cursor_readout};
use super::selection::{
    paint_line_plot_active_selection, paint_line_plot_query_selection,
    paint_line_plot_selection_tooltip,
};
use super::style_helpers::series_color;

pub(super) fn paint_line_plot_panel(
    painter: &mut CanvasPainter<'_>,
    model: &PlotPanelModel,
    output: Option<PlotOutputSnapshot>,
    linked_cursor_x: Option<f64>,
    pinned_series: Option<SeriesId>,
    legend_hover: Option<SeriesId>,
    view_bounds: DataRect,
    query_selection: Option<DataRect>,
    active_selection: Option<LinePlotSelectionOverlay>,
    overlays: &PlotOverlays,
    hidden_series: &[SeriesId],
    step_mode: Option<StepMode>,
    style: LinePlotStyle,
    x_axis_labels: &AxisLabelFormatter,
    y_axis_labels: &AxisLabelFormatter,
    y2_axis_labels: &AxisLabelFormatter,
    y3_axis_labels: &AxisLabelFormatter,
    y4_axis_labels: &AxisLabelFormatter,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    let bounds = painter.bounds();
    let plot = line_plot_inner_rect(bounds, style);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return;
    }

    let background = style
        .background
        .unwrap_or_else(|| painter.theme().snapshot().color_required("surface"));
    painter.scene().push(fret_core::SceneOp::Quad {
        order: DrawOrder(0),
        rect: bounds,
        background: Paint::Solid(background).into(),
        border: if style.border.is_some() {
            Edges::all(style.border_width)
        } else {
            Edges::default()
        },
        border_paint: Paint::Solid(style.border.unwrap_or(Color::TRANSPARENT)).into(),
        corner_radii: Corners::default(),
    });

    let transform = PlotTransform {
        viewport: plot,
        data: view_bounds,
        x_scale,
        y_scale,
    };
    paint_line_plot_images(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        PlotImageLayer::BelowGrid,
        x_scale,
        y_scale,
    );
    paint_line_plot_grid_and_axes(painter, transform, style, &x_axis_labels, &y_axis_labels);
    if let Some(heatmap) = &model.heatmap {
        paint_line_plot_heatmap(painter, transform, heatmap, style);
        paint_line_plot_heatmap_colorbar(painter, plot, heatmap, style);
    }
    paint_line_plot_right_axis_tick_labels(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        style,
        y2_axis_labels,
        y3_axis_labels,
        y4_axis_labels,
        x_scale,
        y_scale,
    );
    paint_line_plot_images(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        PlotImageLayer::AboveGrid,
        x_scale,
        y_scale,
    );
    paint_line_plot_reference_lines(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        style,
        x_scale,
        y_scale,
    );
    paint_line_plot_draggable_shapes(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        style,
        x_scale,
        y_scale,
    );
    paint_line_plot_draggable_overlay_labels(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        style,
        x_scale,
        y_scale,
    );
    paint_line_plot_tag_overlays(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        style,
        x_scale,
        y_scale,
    );
    paint_line_plot_text_overlays(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        style,
        x_scale,
        y_scale,
    );

    let series_count = model.series.len();
    let raster_scale_factor = painter.scale_factor();
    let emphasized_series = if style.emphasize_hovered_series {
        pinned_series.or(legend_hover)
    } else {
        None
    };
    let right_transform = model.data_bounds_y2.map(|axis_bounds| PlotTransform {
        viewport: plot,
        data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
        x_scale,
        y_scale,
    });
    let right2_transform = model.data_bounds_y3.map(|axis_bounds| PlotTransform {
        viewport: plot,
        data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
        x_scale,
        y_scale,
    });
    let right3_transform = model.data_bounds_y4.map(|axis_bounds| PlotTransform {
        viewport: plot,
        data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
        x_scale,
        y_scale,
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
            let stroke_width = series.stroke_width.unwrap_or(style.stroke_width);
            let (wick_commands, up_body_commands, down_body_commands) =
                candlestick_commands_from_series(
                    series_transform,
                    candlestick,
                    stroke_width,
                    raster_scale_factor,
                );
            if wick_commands.is_empty()
                && up_body_commands.is_empty()
                && down_body_commands.is_empty()
            {
                continue;
            }

            let mut wick_color = candlestick
                .wick_color
                .or(series.stroke_color)
                .unwrap_or_else(|| series_color(style, index, series_count));
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
            continue;
        }
        if let Some(bars) = &series.bars {
            let commands = bars_commands_from_series(series_transform, &*series.data, bars);
            if commands.is_empty() {
                continue;
            }

            let mut fill_color = series
                .stroke_color
                .unwrap_or_else(|| series_color(style, index, series_count));
            if let Some(emphasized) = emphasized_series
                && series.id != emphasized
            {
                fill_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
            }
            painter.path(
                line_plot_area_fill_path_key(series.id.0),
                DrawOrder(19),
                Point::new(Px(0.0), Px(0.0)),
                &commands,
                PathStyle::Fill(fret_core::FillStyle::default()),
                fill_color,
                raster_scale_factor,
            );
            continue;
        }
        if let Some(histogram) = &series.histogram {
            let commands =
                histogram_commands_from_series(series_transform, &*series.data, histogram);
            if commands.is_empty() {
                continue;
            }

            let mut fill_color = series
                .stroke_color
                .unwrap_or_else(|| series_color(style, index, series_count));
            if let Some(emphasized) = emphasized_series
                && series.id != emphasized
            {
                fill_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
            }
            painter.path(
                line_plot_area_fill_path_key(series.id.0),
                DrawOrder(19),
                Point::new(Px(0.0), Px(0.0)),
                &commands,
                PathStyle::Fill(fret_core::FillStyle::default()),
                fill_color,
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

    paint_line_plot_legend(painter, model, plot, pinned_series, legend_hover, style);
    paint_line_plot_query_selection(
        painter,
        plot,
        view_bounds,
        query_selection,
        style,
        x_scale,
        y_scale,
    );
    paint_line_plot_active_selection(painter, plot, active_selection, style);
    if paint_line_plot_selection_tooltip(
        painter,
        bounds,
        plot,
        view_bounds,
        active_selection,
        style,
        x_scale,
        y_scale,
    ) {
        return;
    }
    paint_line_plot_cursor_readout(
        painter,
        model,
        plot,
        output,
        pinned_series,
        hidden_series,
        style,
        y_axis_labels,
        y2_axis_labels,
        y3_axis_labels,
        y4_axis_labels,
        x_scale,
        y_scale,
    );
    paint_line_plot_linked_cursor_readout(
        painter,
        model,
        plot,
        transform.data,
        output.and_then(|snapshot| snapshot.cursor),
        linked_cursor_x,
        pinned_series,
        hidden_series,
        style,
        y_axis_labels,
        y2_axis_labels,
        y3_axis_labels,
        y4_axis_labels,
        x_scale,
        y_scale,
    );
}
