//! Declarative line-plot tag overlay paint owner.

use fret_core::{FontWeight, Px, Rect, TextOverflow, TextStyle, TextWrap};
use fret_ui::canvas::{CanvasPainter, CanvasTextConstraints};

use crate::cartesian::{AxisScale, DataRect, PlotTransform};
use crate::plot::axis::AxisLabelFormatter;
use crate::state::PlotOverlays;
use crate::style::LinePlotStyle;

use super::super::geometry::line_plot_view_bounds_for_y_axis;
use super::super::style_helpers::axis_tick_label_text;
use super::{
    line_plot_annotation_label, line_plot_annotation_tokens, paint_line_plot_tag_x_overlay,
    paint_line_plot_tag_y_overlay,
};

pub(in crate::declarative) fn paint_line_plot_tag_overlays(
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
    if overlays.tags_x.is_empty() && overlays.tags_y.is_empty() {
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
    let scope = painter.key_scope(&"fret-plot.declarative.tag-overlays");
    let formatter = AxisLabelFormatter::default();
    let x_span = (view_bounds.x_max - view_bounds.x_min).abs();
    let y_span = (view_bounds.y_max - view_bounds.y_min).abs();

    for (index, tag) in overlays.tags_x.iter().enumerate() {
        if !tag.x.is_finite() {
            continue;
        }
        let Some(x_px) = transform.data_x_to_px(tag.x) else {
            continue;
        };
        let value = tag
            .show_value
            .then(|| axis_tick_label_text(x_scale, &formatter, tag.x, x_span));
        let text = line_plot_annotation_label(tag.label.as_deref(), value);
        if text.is_empty() {
            continue;
        }
        let key: u64 = painter
            .child_key(
                scope,
                &(
                    "tag-x",
                    index,
                    tag.x.to_bits(),
                    tag.label.as_deref(),
                    tag.show_value,
                    text.as_str(),
                ),
            )
            .into();
        paint_line_plot_tag_x_overlay(
            painter,
            plot,
            Px(x_px.0.round()),
            tag.color.unwrap_or(tokens.stroke),
            key,
            text,
            &text_style,
            constraints,
            raster_scale_factor,
            tokens,
        );
    }

    for (index, tag) in overlays.tags_y.iter().enumerate() {
        if !tag.y.is_finite() {
            continue;
        }
        let (transform, right, span) = match tag.axis {
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
        let Some(y_px) = transform.data_y_to_px(tag.y) else {
            continue;
        };
        let value = tag
            .show_value
            .then(|| axis_tick_label_text(y_scale, &formatter, tag.y, span));
        let text = line_plot_annotation_label(tag.label.as_deref(), value);
        if text.is_empty() {
            continue;
        }
        let key: u64 = painter
            .child_key(
                scope,
                &(
                    "tag-y",
                    index,
                    tag.y.to_bits(),
                    tag.label.as_deref(),
                    tag.show_value,
                    tag.axis,
                    text.as_str(),
                ),
            )
            .into();
        paint_line_plot_tag_y_overlay(
            painter,
            plot,
            Px(y_px.0.round()),
            right,
            tag.color.unwrap_or(tokens.stroke),
            key,
            text,
            &text_style,
            constraints,
            raster_scale_factor,
            tokens,
        );
    }
}
