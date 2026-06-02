//! Declarative line-plot reference-line overlay paint owner.

use fret_core::{Color, DrawOrder, Point, Px, Rect, Size};
use fret_ui::canvas::CanvasPainter;

use crate::cartesian::{AxisScale, DataRect, PlotTransform};
use crate::models::YAxis;
use crate::state::PlotOverlays;
use crate::style::LinePlotStyle;

use super::super::geometry::line_plot_view_bounds_for_y_axis;
use super::super::paint_primitives::push_filled_rect;

pub(in crate::declarative) fn paint_line_plot_reference_lines(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    view_bounds: DataRect,
    view_bounds_y2: Option<DataRect>,
    view_bounds_y3: Option<DataRect>,
    view_bounds_y4: Option<DataRect>,
    overlays: &PlotOverlays,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    if overlays.inf_lines_x.is_empty()
        && overlays.inf_lines_y.is_empty()
        && overlays.drag_lines_x.is_empty()
        && overlays.drag_lines_y.is_empty()
    {
        return;
    }
    let Some(transform) = (PlotTransform {
        viewport: plot,
        data: view_bounds,
        x_scale,
        y_scale,
    })
    .prepare() else {
        return;
    };

    let transform_y2 = view_bounds_y2.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });
    let transform_y3 = view_bounds_y3.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });
    let transform_y4 = view_bounds_y4.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });

    let theme = painter.theme().snapshot();
    let base_color = style
        .crosshair_color
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    let default_color = Color {
        a: (base_color.a * 0.45).clamp(0.05, 1.0),
        ..base_color
    };

    let x_lines = overlays
        .inf_lines_x
        .iter()
        .map(|line| (line.x, line.width, line.color.unwrap_or(default_color)))
        .chain(
            overlays
                .drag_lines_x
                .iter()
                .map(|line| (line.x, line.width, line.color.unwrap_or(default_color))),
        );
    for (x_value, line_width, line_color) in x_lines {
        let Some(x) = transform.data_x_to_px(x_value) else {
            continue;
        };
        let width = line_width.0.max(1.0).min(plot.size.width.0.max(1.0));
        let left =
            (x.0 - width * 0.5).clamp(plot.origin.x.0, plot.origin.x.0 + plot.size.width.0 - width);
        push_filled_rect(
            painter,
            Rect::new(
                Point::new(Px(left.round()), plot.origin.y),
                Size::new(Px(width), plot.size.height),
            ),
            DrawOrder(3),
            line_color,
        );
    }

    let y_lines = overlays
        .inf_lines_y
        .iter()
        .map(|line| {
            (
                line.y,
                line.axis,
                line.width,
                line.color.unwrap_or(default_color),
            )
        })
        .chain(overlays.drag_lines_y.iter().map(|line| {
            (
                line.y,
                line.axis,
                line.width,
                line.color.unwrap_or(default_color),
            )
        }));
    for (y_value, axis, line_width, line_color) in y_lines {
        let transform = match axis {
            YAxis::Left => transform,
            YAxis::Right => transform_y2.unwrap_or(transform),
            YAxis::Right2 => transform_y3.unwrap_or(transform),
            YAxis::Right3 => transform_y4.unwrap_or(transform),
        };
        let Some(y) = transform.data_y_to_px(y_value) else {
            continue;
        };
        let height = line_width.0.max(1.0).min(plot.size.height.0.max(1.0));
        let top = (y.0 - height * 0.5).clamp(
            plot.origin.y.0,
            plot.origin.y.0 + plot.size.height.0 - height,
        );
        push_filled_rect(
            painter,
            Rect::new(
                Point::new(plot.origin.x, Px(top.round())),
                Size::new(plot.size.width, Px(height)),
            ),
            DrawOrder(3),
            line_color,
        );
    }
}
