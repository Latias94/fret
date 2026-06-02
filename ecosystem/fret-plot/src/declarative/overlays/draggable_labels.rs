//! Declarative line-plot draggable overlay label paint owner.

use fret_core::{FontWeight, Point, Px, Rect, TextOverflow, TextStyle, TextWrap};
use fret_ui::canvas::{CanvasPainter, CanvasTextConstraints};

use crate::cartesian::{AxisScale, DataRect, PlotTransform};
use crate::plot::axis::AxisLabelFormatter;
use crate::state::PlotOverlays;
use crate::style::LinePlotStyle;

use super::super::geometry::line_plot_view_bounds_for_y_axis;
use super::super::style_helpers::axis_tick_label_text;
use super::{
    line_plot_annotation_label, line_plot_annotation_tokens, paint_line_plot_annotation_text_box,
    paint_line_plot_tag_x_overlay, paint_line_plot_tag_y_overlay,
};

pub(in crate::declarative) fn paint_line_plot_draggable_overlay_labels(
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
    if overlays.drag_lines_x.is_empty()
        && overlays.drag_lines_y.is_empty()
        && overlays.drag_points.is_empty()
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

    let tokens = line_plot_annotation_tokens(painter, style);
    let text_style = TextStyle {
        size: Px(12.0),
        weight: FontWeight::NORMAL,
        ..TextStyle::default()
    };
    let constraints = CanvasTextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let raster_scale_factor = painter.scale_factor();
    let scope = painter.key_scope(&"fret-plot.declarative.draggable-overlay-labels");
    let formatter = AxisLabelFormatter::default();
    let x_span = (view_bounds.x_max - view_bounds.x_min).abs();
    let y_span = (view_bounds.y_max - view_bounds.y_min).abs();

    for (index, line) in overlays.drag_lines_x.iter().enumerate() {
        if !line.x.is_finite() {
            continue;
        }
        let Some(x_px) = transform.data_x_to_px(line.x) else {
            continue;
        };
        let value = line
            .show_value
            .then(|| axis_tick_label_text(x_scale, &formatter, line.x, x_span));
        let text = line_plot_annotation_label(line.label.as_deref(), value);
        if text.is_empty() {
            continue;
        }
        let key: u64 = painter
            .child_key(
                scope,
                &(
                    "drag-line-x",
                    index,
                    line.id,
                    line.x.to_bits(),
                    line.label.as_deref(),
                    line.show_value,
                    text.as_str(),
                ),
            )
            .into();
        paint_line_plot_tag_x_overlay(
            painter,
            plot,
            Px(x_px.0.round()),
            line.color.unwrap_or(tokens.stroke),
            key,
            text,
            &text_style,
            constraints,
            raster_scale_factor,
            tokens,
        );
    }

    for (index, line) in overlays.drag_lines_y.iter().enumerate() {
        if !line.y.is_finite() {
            continue;
        }
        let (transform, right, span) = match line.axis {
            crate::models::YAxis::Left => (Some(transform), false, y_span),
            crate::models::YAxis::Right => (
                Some(transform_y2.unwrap_or(transform)),
                true,
                view_bounds_y2
                    .map(|b| (b.y_max - b.y_min).abs())
                    .unwrap_or(y_span),
            ),
            crate::models::YAxis::Right2 => (
                Some(transform_y3.unwrap_or(transform)),
                true,
                view_bounds_y3
                    .map(|b| (b.y_max - b.y_min).abs())
                    .unwrap_or(y_span),
            ),
            crate::models::YAxis::Right3 => (
                Some(transform_y4.unwrap_or(transform)),
                true,
                view_bounds_y4
                    .map(|b| (b.y_max - b.y_min).abs())
                    .unwrap_or(y_span),
            ),
        };
        let Some(transform) = transform else {
            continue;
        };
        let Some(y_px) = transform.data_y_to_px(line.y) else {
            continue;
        };
        let value = line
            .show_value
            .then(|| axis_tick_label_text(y_scale, &formatter, line.y, span));
        let text = line_plot_annotation_label(line.label.as_deref(), value);
        if text.is_empty() {
            continue;
        }
        let key: u64 = painter
            .child_key(
                scope,
                &(
                    "drag-line-y",
                    index,
                    line.id,
                    line.y.to_bits(),
                    line.label.as_deref(),
                    line.show_value,
                    line.axis,
                    text.as_str(),
                ),
            )
            .into();
        paint_line_plot_tag_y_overlay(
            painter,
            plot,
            Px(y_px.0.round()),
            right,
            line.color.unwrap_or(tokens.stroke),
            key,
            text,
            &text_style,
            constraints,
            raster_scale_factor,
            tokens,
        );
    }

    for (index, point) in overlays.drag_points.iter().enumerate() {
        let current = point.point;
        if !current.x.is_finite() || !current.y.is_finite() {
            continue;
        }
        let (transform, span) = match point.axis {
            crate::models::YAxis::Left => (Some(transform), y_span),
            crate::models::YAxis::Right => (
                Some(transform_y2.unwrap_or(transform)),
                view_bounds_y2
                    .map(|b| (b.y_max - b.y_min).abs())
                    .unwrap_or(y_span),
            ),
            crate::models::YAxis::Right2 => (
                Some(transform_y3.unwrap_or(transform)),
                view_bounds_y3
                    .map(|b| (b.y_max - b.y_min).abs())
                    .unwrap_or(y_span),
            ),
            crate::models::YAxis::Right3 => (
                Some(transform_y4.unwrap_or(transform)),
                view_bounds_y4
                    .map(|b| (b.y_max - b.y_min).abs())
                    .unwrap_or(y_span),
            ),
        };
        let Some(transform) = transform else {
            continue;
        };
        let p_px = transform.data_to_px(current);
        if !p_px.x.0.is_finite() || !p_px.y.0.is_finite() {
            continue;
        }
        let value = point.show_value.then(|| {
            let x = axis_tick_label_text(x_scale, &formatter, current.x, x_span);
            let y = axis_tick_label_text(y_scale, &formatter, current.y, span);
            format!("({x}, {y})")
        });
        let text = line_plot_annotation_label(point.label.as_deref(), value);
        if text.is_empty() {
            continue;
        }
        let margin = Px(8.0);
        let origin = Point::new(
            Px((p_px.x.0 + margin.0).round()),
            Px((p_px.y.0 - margin.0).round()),
        );
        let key: u64 = painter
            .child_key(
                scope,
                &(
                    "drag-point",
                    index,
                    point.id,
                    current.x.to_bits(),
                    current.y.to_bits(),
                    point.label.as_deref(),
                    point.show_value,
                    point.axis,
                    text.as_str(),
                ),
            )
            .into();
        paint_line_plot_annotation_text_box(
            painter,
            plot,
            origin,
            key,
            text,
            &text_style,
            constraints,
            raster_scale_factor,
            tokens.text,
            Some(tokens.background),
            Some(tokens.border),
            tokens.padding,
            tokens.radius,
        );
    }
}
