//! Declarative line-plot grid and baseline axis paint owner.

use fret_core::{DrawOrder, Px};
use fret_ui::canvas::CanvasPainter;

use crate::cartesian::PlotTransform;
use crate::plot::axis::{AxisLabelFormatter, AxisTicks, axis_ticks_scaled};
use crate::style::LinePlotStyle;

use super::axis_labels::paint_line_plot_axis_tick_labels;
use super::paint_primitives::{push_horizontal_line, push_vertical_line};

pub(super) fn paint_line_plot_grid_and_axes(
    painter: &mut CanvasPainter<'_>,
    transform: PlotTransform,
    style: LinePlotStyle,
    x_axis_labels: &AxisLabelFormatter,
    y_axis_labels: &AxisLabelFormatter,
) {
    let plot = transform.viewport;
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return;
    }

    let theme = painter.theme().snapshot();
    let mut grid_color = style
        .grid_color
        .unwrap_or_else(|| theme.color_required("border"));
    grid_color.a *= 0.45;
    let axis_color = style
        .axis_color
        .unwrap_or_else(|| theme.color_required("border"));
    let tick_count = style.tick_count.max(2);

    let x_ticks = axis_ticks_scaled(
        transform.data.x_min,
        transform.data.x_max,
        tick_count,
        AxisTicks::Nice,
        transform.x_scale,
    );
    let y_ticks = axis_ticks_scaled(
        transform.data.y_min,
        transform.data.y_max,
        tick_count,
        AxisTicks::Nice,
        transform.y_scale,
    );

    for x in x_ticks.iter().copied() {
        let Some(px) = transform.data_x_to_px(x) else {
            continue;
        };
        push_vertical_line(
            painter,
            px,
            plot.origin.y,
            plot.size.height,
            DrawOrder(2),
            grid_color,
        );
    }

    for y in y_ticks.iter().copied() {
        let Some(py) = transform.data_y_to_px(y) else {
            continue;
        };
        push_horizontal_line(
            painter,
            plot.origin.x,
            py,
            plot.size.width,
            DrawOrder(2),
            grid_color,
        );
    }

    let baseline_y = transform
        .data_y_to_px(0.0)
        .filter(|y| y.0 >= plot.origin.y.0 && y.0 <= plot.origin.y.0 + plot.size.height.0)
        .unwrap_or_else(|| Px(plot.origin.y.0 + plot.size.height.0 - 1.0));
    let baseline_x = transform
        .data_x_to_px(0.0)
        .filter(|x| x.0 >= plot.origin.x.0 && x.0 <= plot.origin.x.0 + plot.size.width.0)
        .unwrap_or(plot.origin.x);

    push_horizontal_line(
        painter,
        plot.origin.x,
        baseline_y,
        plot.size.width,
        DrawOrder(10),
        axis_color,
    );
    push_vertical_line(
        painter,
        baseline_x,
        plot.origin.y,
        plot.size.height,
        DrawOrder(10),
        axis_color,
    );

    paint_line_plot_axis_tick_labels(
        painter,
        transform,
        style,
        &x_ticks,
        &y_ticks,
        x_axis_labels,
        y_axis_labels,
    );
}
