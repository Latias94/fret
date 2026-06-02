//! Declarative line-plot axis tick label paint owner.

use fret_core::{DrawOrder, FontWeight, Point, Px, Rect, TextOverflow, TextStyle, TextWrap};
use fret_ui::canvas::{CanvasPainter, CanvasTextConstraints};

use crate::cartesian::{AxisScale, DataRect, PlotTransform};
use crate::plot::axis::{AxisLabelFormatter, AxisTicks, axis_ticks_scaled};
use crate::style::LinePlotStyle;

use super::axis_tick_label_text;
use super::geometry::line_plot_view_bounds_for_y_axis;

pub(super) fn paint_line_plot_axis_tick_labels(
    painter: &mut CanvasPainter<'_>,
    transform: PlotTransform,
    style: LinePlotStyle,
    x_ticks: &[f64],
    y_ticks: &[f64],
    x_formatter: &AxisLabelFormatter,
    y_formatter: &AxisLabelFormatter,
) {
    if x_ticks.is_empty() && y_ticks.is_empty() {
        return;
    }

    let plot = transform.viewport;
    let theme = painter.theme().snapshot();
    let text_color = style
        .label_color
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    let text_style = TextStyle {
        size: Px(12.0),
        weight: FontWeight::NORMAL,
        ..TextStyle::default()
    };
    let constraints = CanvasTextConstraints {
        max_width: Some(Px(72.0)),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let x_span = (transform.data.x_max - transform.data.x_min).abs();
    let y_span = (transform.data.y_max - transform.data.y_min).abs();
    let scope = painter.key_scope(&"fret-plot.declarative.axis-labels");
    let raster_scale_factor = painter.scale_factor();

    let x_label_y = Px(plot.origin.y.0 + plot.size.height.0 + 2.0);
    for (index, value) in x_ticks.iter().copied().enumerate() {
        let Some(x) = transform.data_x_to_px(value) else {
            continue;
        };
        let text = axis_tick_label_text(transform.x_scale, x_formatter, value, x_span);
        if text.is_empty() {
            continue;
        }
        let key: u64 = painter
            .child_key(scope, &("x", index, value.to_bits()))
            .into();
        let _ = painter.text(
            key,
            DrawOrder(11),
            Point::new(Px(x.0 - 12.0), x_label_y),
            text,
            text_style.clone(),
            text_color,
            constraints,
            raster_scale_factor,
        );
    }

    let y_label_x = Px((plot.origin.x.0 - style.axis_gap.0 + 4.0).max(0.0));
    for (index, value) in y_ticks.iter().copied().enumerate() {
        let Some(y) = transform.data_y_to_px(value) else {
            continue;
        };
        let text = axis_tick_label_text(transform.y_scale, y_formatter, value, y_span);
        if text.is_empty() {
            continue;
        }
        let key: u64 = painter
            .child_key(scope, &("y", index, value.to_bits()))
            .into();
        let _ = painter.text(
            key,
            DrawOrder(11),
            Point::new(y_label_x, y),
            text,
            text_style.clone(),
            text_color,
            constraints,
            raster_scale_factor,
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_line_plot_right_axis_tick_labels(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    primary_view_bounds: DataRect,
    view_bounds_y2: Option<DataRect>,
    view_bounds_y3: Option<DataRect>,
    view_bounds_y4: Option<DataRect>,
    style: LinePlotStyle,
    y2_formatter: &AxisLabelFormatter,
    y3_formatter: &AxisLabelFormatter,
    y4_formatter: &AxisLabelFormatter,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return;
    }

    for (axis_index, axis_key, axis_bounds, formatter) in [
        (0usize, "y2", view_bounds_y2, y2_formatter),
        (1usize, "y3", view_bounds_y3, y3_formatter),
        (2usize, "y4", view_bounds_y4, y4_formatter),
    ] {
        let Some(axis_bounds) = axis_bounds else {
            continue;
        };
        let transform = PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(primary_view_bounds, axis_bounds),
            x_scale,
            y_scale,
        };
        let y_ticks = axis_ticks_scaled(
            transform.data.y_min,
            transform.data.y_max,
            style.tick_count.max(2),
            AxisTicks::Nice,
            transform.y_scale,
        );
        paint_line_plot_right_axis_tick_labels_for_axis(
            painter, transform, style, &y_ticks, formatter, axis_index, axis_key,
        );
    }
}

fn paint_line_plot_right_axis_tick_labels_for_axis(
    painter: &mut CanvasPainter<'_>,
    transform: PlotTransform,
    style: LinePlotStyle,
    y_ticks: &[f64],
    formatter: &AxisLabelFormatter,
    axis_index: usize,
    axis_key: &'static str,
) {
    if y_ticks.is_empty() {
        return;
    }

    let plot = transform.viewport;
    let theme = painter.theme().snapshot();
    let text_color = style
        .label_color
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    let text_style = TextStyle {
        size: Px(12.0),
        weight: FontWeight::NORMAL,
        ..TextStyle::default()
    };
    let constraints = CanvasTextConstraints {
        max_width: Some(Px(72.0)),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let span = (transform.data.y_max - transform.data.y_min).abs();
    let scope = painter.key_scope(&"fret-plot.declarative.right-axis-labels");
    let raster_scale_factor = painter.scale_factor();
    let lane_gap = style.axis_gap.0.max(18.0);
    let label_x = Px(plot.origin.x.0 + plot.size.width.0 + 4.0 + axis_index as f32 * lane_gap);

    for (index, value) in y_ticks.iter().copied().enumerate() {
        let Some(y) = transform.data_y_to_px(value) else {
            continue;
        };
        let text = axis_tick_label_text(transform.y_scale, formatter, value, span);
        if text.is_empty() {
            continue;
        }
        let key: u64 = painter
            .child_key(scope, &(axis_key, index, value.to_bits()))
            .into();
        let _ = painter.text(
            key,
            DrawOrder(11),
            Point::new(label_x, y),
            text,
            text_style.clone(),
            text_color,
            constraints,
            raster_scale_factor,
        );
    }
}
