//! Declarative line-plot draggable shape overlay paint owner.

use fret_core::{Color, Corners, DrawOrder, Edges, Paint, Point, Px, Rect, SceneOp, Size};
use fret_ui::canvas::CanvasPainter;

use crate::cartesian::{AxisScale, DataPoint, DataRect, PlotTransform};
use crate::models::YAxis;
use crate::state::PlotOverlays;
use crate::style::LinePlotStyle;

use super::super::geometry::line_plot_view_bounds_for_y_axis;

pub(in crate::declarative) fn paint_line_plot_draggable_shapes(
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
    if overlays.drag_points.is_empty() && overlays.drag_rects.is_empty() {
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
    let stroke = Color {
        a: (base_color.a * 0.45).clamp(0.05, 1.0),
        ..base_color
    };
    let border = style
        .tooltip_border
        .unwrap_or_else(|| theme.color_required("border"));

    for point in overlays.drag_points.iter() {
        let p = point.point;
        if !p.x.is_finite() || !p.y.is_finite() {
            continue;
        }
        let transform = match point.axis {
            YAxis::Left => transform,
            YAxis::Right => transform_y2.unwrap_or(transform),
            YAxis::Right2 => transform_y3.unwrap_or(transform),
            YAxis::Right3 => transform_y4.unwrap_or(transform),
        };
        let p_px = transform.data_to_px(p);
        if !p_px.x.0.is_finite() || !p_px.y.0.is_finite() {
            continue;
        }

        let radius = point.radius.0.max(2.0);
        let diameter = (radius * 2.0).max(1.0);
        let max_left = (plot.size.width.0 - diameter).max(0.0);
        let max_top = (plot.size.height.0 - diameter).max(0.0);
        let left = (p_px.x.0 - plot.origin.x.0 - radius).clamp(0.0, max_left);
        let top = (p_px.y.0 - plot.origin.y.0 - radius).clamp(0.0, max_top);
        painter.scene().push(SceneOp::Quad {
            order: DrawOrder(3),
            rect: Rect::new(
                Point::new(
                    Px((plot.origin.x.0 + left).round()),
                    Px((plot.origin.y.0 + top).round()),
                ),
                Size::new(Px(diameter), Px(diameter)),
            ),
            background: Paint::Solid(point.color.unwrap_or(stroke)).into(),
            border: Edges::all(Px(1.0)),
            border_paint: Paint::Solid(border).into(),
            corner_radii: Corners::all(Px(radius)),
        });
    }

    for rect in overlays.drag_rects.iter() {
        let transform = match rect.axis {
            YAxis::Left => transform,
            YAxis::Right => transform_y2.unwrap_or(transform),
            YAxis::Right2 => transform_y3.unwrap_or(transform),
            YAxis::Right3 => transform_y4.unwrap_or(transform),
        };
        let current = rect.rect;
        if !current.x_min.is_finite()
            || !current.x_max.is_finite()
            || !current.y_min.is_finite()
            || !current.y_max.is_finite()
        {
            continue;
        }

        let a = transform.data_to_px(DataPoint {
            x: current.x_min,
            y: current.y_min,
        });
        let b = transform.data_to_px(DataPoint {
            x: current.x_max,
            y: current.y_max,
        });
        if !a.x.0.is_finite() || !a.y.0.is_finite() || !b.x.0.is_finite() || !b.y.0.is_finite() {
            continue;
        }

        let left =
            a.x.0
                .min(b.x.0)
                .clamp(plot.origin.x.0, plot.origin.x.0 + plot.size.width.0);
        let right =
            a.x.0
                .max(b.x.0)
                .clamp(plot.origin.x.0, plot.origin.x.0 + plot.size.width.0);
        let top =
            a.y.0
                .min(b.y.0)
                .clamp(plot.origin.y.0, plot.origin.y.0 + plot.size.height.0);
        let bottom =
            a.y.0
                .max(b.y.0)
                .clamp(plot.origin.y.0, plot.origin.y.0 + plot.size.height.0);
        let width = (right - left).max(0.0);
        let height = (bottom - top).max(0.0);
        if width <= 0.0 || height <= 0.0 {
            continue;
        }

        let color = rect.color.unwrap_or(stroke);
        let fill = rect.fill.unwrap_or(Color { a: 0.12, ..color });
        painter.scene().push(SceneOp::Quad {
            order: DrawOrder(3),
            rect: Rect::new(
                Point::new(Px(left.round()), Px(top.round())),
                Size::new(Px(width), Px(height)),
            ),
            background: Paint::Solid(fill).into(),
            border: Edges::all(Px(rect.border_width.0.max(1.0))),
            border_paint: Paint::Solid(color).into(),
            corner_radii: Corners::default(),
        });
    }
}
