//! Declarative line-plot plot-text overlay paint owner.

use fret_core::{
    Corners, DrawOrder, Edges, FontWeight, Paint, Point, Px, Rect, SceneOp, Size, TextOverflow,
    TextStyle, TextWrap,
};
use fret_ui::canvas::{CanvasPainter, CanvasTextConstraints};

use crate::cartesian::{AxisScale, DataRect, PlotTransform};
use crate::state::PlotOverlays;
use crate::style::LinePlotStyle;

use super::super::geometry::line_plot_view_bounds_for_y_axis;
use super::{line_plot_annotation_tokens, line_plot_clamp_plot_left, line_plot_clamp_plot_top};

pub(in crate::declarative) fn paint_line_plot_text_overlays(
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
    if overlays.text.is_empty() {
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
    let scope = painter.key_scope(&"fret-plot.declarative.plot-text-overlays");

    for (index, text) in overlays.text.iter().enumerate() {
        if !text.x.is_finite() || !text.y.is_finite() {
            continue;
        }
        let (transform, right) = match text.axis {
            crate::models::YAxis::Left => (Some(transform), false),
            crate::models::YAxis::Right => (Some(transform_y2.unwrap_or(transform)), true),
            crate::models::YAxis::Right2 => (Some(transform_y3.unwrap_or(transform)), true),
            crate::models::YAxis::Right3 => (Some(transform_y4.unwrap_or(transform)), true),
        };
        let Some(transform) = transform else {
            continue;
        };
        let Some(px_x) = transform.data_x_to_px(text.x) else {
            continue;
        };
        let Some(px_y) = transform.data_y_to_px(text.y) else {
            continue;
        };
        let origin = Point::new(
            Px((px_x.0 + text.offset.x.0).round()),
            Px((px_y.0 + text.offset.y.0).round()),
        );
        let padding = if text.background.is_some() && text.padding.0 <= 0.0 {
            tokens.padding
        } else {
            text.padding
        };
        let corner_radius = if text.background.is_some() && text.corner_radius.0 <= 0.0 {
            tokens.radius
        } else {
            text.corner_radius
        };

        let key: u64 = painter
            .child_key(
                scope,
                &(
                    "plot-text",
                    index,
                    text.x.to_bits(),
                    text.y.to_bits(),
                    text.offset.x.0.to_bits(),
                    text.offset.y.0.to_bits(),
                    text.axis,
                    text.text.as_str(),
                ),
            )
            .into();
        let (_blob, metrics) = painter.prepare_text_with_blob(
            key,
            text.text.clone(),
            text_style.clone(),
            constraints,
            raster_scale_factor,
        );

        let width = Px(metrics.size.width.0 + padding.0 * 2.0);
        let height = Px(metrics.size.height.0 + padding.0 * 2.0);
        if width.0 < 0.0 || height.0 < 0.0 {
            continue;
        }
        let left = if right {
            line_plot_clamp_plot_left(
                plot,
                (plot.origin.x.0 + plot.size.width.0 - width.0 - tokens.padding.0)
                    .max(plot.origin.x.0),
                width,
            )
        } else {
            line_plot_clamp_plot_left(plot, origin.x.0, width)
        };
        let top = line_plot_clamp_plot_top(plot, origin.y.0, height);
        let rect = Rect::new(Point::new(Px(left), Px(top)), Size::new(width, height));

        if let Some(background) = text.background {
            painter.scene().push(SceneOp::Quad {
                order: DrawOrder(3),
                rect,
                background: Paint::Solid(background).into(),
                border: Edges::all(Px(1.0)),
                border_paint: Paint::Solid(text.border.unwrap_or(tokens.border)).into(),
                corner_radii: Corners::all(corner_radius),
            });
        }

        let _ = painter.text(
            key,
            DrawOrder(3),
            Point::new(
                Px(rect.origin.x.0 + padding.0),
                Px(rect.origin.y.0 + padding.0 + metrics.baseline.0),
            ),
            text.text.clone(),
            text_style.clone(),
            text.color.unwrap_or(tokens.text),
            constraints,
            raster_scale_factor,
        );
    }
}
