//! Declarative line-plot overlay paint owner.

use fret_core::{
    Color, Corners, DrawOrder, Edges, FontWeight, Paint, Point, Px, Rect, Size, TextOverflow,
    TextStyle, TextWrap,
};
use fret_ui::canvas::{CanvasPainter, CanvasTextConstraints};

use crate::cartesian::{AxisScale, DataPoint, DataRect, PlotTransform};
use crate::state::PlotOverlays;
use crate::style::LinePlotStyle;

use super::geometry::line_plot_view_bounds_for_y_axis;
use super::paint_primitives::push_filled_rect;

mod draggable_labels;
mod images;
mod tags;

pub(super) use draggable_labels::paint_line_plot_draggable_overlay_labels;
pub(super) use images::paint_line_plot_images;
pub(super) use tags::paint_line_plot_tag_overlays;

pub(super) fn paint_line_plot_reference_lines(
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
            crate::models::YAxis::Left => transform,
            crate::models::YAxis::Right => transform_y2.unwrap_or(transform),
            crate::models::YAxis::Right2 => transform_y3.unwrap_or(transform),
            crate::models::YAxis::Right3 => transform_y4.unwrap_or(transform),
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

pub(super) fn paint_line_plot_draggable_shapes(
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
            crate::models::YAxis::Left => transform,
            crate::models::YAxis::Right => transform_y2.unwrap_or(transform),
            crate::models::YAxis::Right2 => transform_y3.unwrap_or(transform),
            crate::models::YAxis::Right3 => transform_y4.unwrap_or(transform),
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
        painter.scene().push(fret_core::SceneOp::Quad {
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
            crate::models::YAxis::Left => transform,
            crate::models::YAxis::Right => transform_y2.unwrap_or(transform),
            crate::models::YAxis::Right2 => transform_y3.unwrap_or(transform),
            crate::models::YAxis::Right3 => transform_y4.unwrap_or(transform),
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
        painter.scene().push(fret_core::SceneOp::Quad {
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

#[derive(Debug, Clone, Copy)]
struct LinePlotAnnotationTokens {
    background: Color,
    border: Color,
    text: Color,
    stroke: Color,
    padding: Px,
    radius: Px,
}

fn line_plot_annotation_tokens(
    painter: &mut CanvasPainter<'_>,
    style: LinePlotStyle,
) -> LinePlotAnnotationTokens {
    let theme = painter.theme();
    let tooltip_background = style
        .tooltip_background
        .unwrap_or_else(|| theme.color_required("popover"));
    let tooltip_border = style
        .tooltip_border
        .unwrap_or_else(|| theme.color_required("border"));
    let tooltip_text = style
        .tooltip_text_color
        .or(style.label_color)
        .unwrap_or_else(|| theme.color_required("popover-foreground"));
    let crosshair = style
        .crosshair_color
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    LinePlotAnnotationTokens {
        background: crate::theme_tokens::color(
            theme,
            "fret.plot.annotation.background",
            "plot.annotation.background",
        )
        .unwrap_or(tooltip_background),
        border: crate::theme_tokens::color(
            theme,
            "fret.plot.annotation.border",
            "plot.annotation.border",
        )
        .unwrap_or(tooltip_border),
        text: crate::theme_tokens::color(
            theme,
            "fret.plot.annotation.text",
            "plot.annotation.text",
        )
        .unwrap_or(tooltip_text),
        stroke: crate::theme_tokens::color(
            theme,
            "fret.plot.annotation.stroke",
            "plot.annotation.stroke",
        )
        .unwrap_or(crosshair),
        padding: crate::theme_tokens::metric(
            theme,
            "fret.plot.annotation.padding",
            "plot.annotation.padding",
        )
        .unwrap_or_else(|| theme.metric_token("metric.padding.sm")),
        radius: crate::theme_tokens::metric(
            theme,
            "fret.plot.annotation.radius",
            "plot.annotation.radius",
        )
        .unwrap_or_else(|| theme.metric_token("metric.radius.sm")),
    }
}

fn line_plot_annotation_label(label: Option<&str>, value: Option<String>) -> String {
    match (label, value) {
        (Some(label), Some(value)) => format!("{label}: {value}"),
        (Some(label), None) => label.to_owned(),
        (None, Some(value)) => value,
        (None, None) => String::new(),
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_line_plot_annotation_text_box(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    origin: Point,
    key: u64,
    text: String,
    text_style: &TextStyle,
    constraints: CanvasTextConstraints,
    raster_scale_factor: f32,
    color: Color,
    background: Option<Color>,
    border: Option<Color>,
    padding: Px,
    corner_radius: Px,
) {
    let (_blob, metrics) = painter.prepare_text_with_blob(
        key,
        text.clone(),
        text_style.clone(),
        constraints,
        raster_scale_factor,
    );
    let width = Px(metrics.size.width.0 + padding.0 * 2.0);
    let height = Px(metrics.size.height.0 + padding.0 * 2.0);
    if width.0 < 0.0 || height.0 < 0.0 {
        return;
    }
    let left = line_plot_clamp_plot_left(plot, origin.x.0, width);
    let top = line_plot_clamp_plot_top(plot, origin.y.0, height);
    let rect = Rect::new(Point::new(Px(left), Px(top)), Size::new(width, height));

    if let Some(background) = background {
        painter.scene().push(fret_core::SceneOp::Quad {
            order: DrawOrder(3),
            rect,
            background: Paint::Solid(background).into(),
            border: Edges::all(Px(1.0)),
            border_paint: Paint::Solid(border.unwrap_or(Color::TRANSPARENT)).into(),
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
        text,
        text_style.clone(),
        color,
        constraints,
        raster_scale_factor,
    );
}

#[allow(clippy::too_many_arguments)]
fn paint_line_plot_tag_x_overlay(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    x: Px,
    marker_color: Color,
    key: u64,
    text: String,
    text_style: &TextStyle,
    constraints: CanvasTextConstraints,
    raster_scale_factor: f32,
    tokens: LinePlotAnnotationTokens,
) {
    let (_blob, metrics) = painter.prepare_text_with_blob(
        key,
        text.clone(),
        text_style.clone(),
        constraints,
        raster_scale_factor,
    );
    let pad = tokens.padding;
    let width = Px(metrics.size.width.0 + pad.0 * 2.0);
    let height = Px(metrics.size.height.0 + pad.0 * 2.0);
    let margin = Px(6.0);
    let left = line_plot_clamp_plot_left(plot, x.0 - width.0 * 0.5, width);
    let top = line_plot_clamp_plot_top(
        plot,
        plot.origin.y.0 + plot.size.height.0 - height.0 - margin.0,
        height,
    );
    let rect = Rect::new(Point::new(Px(left), Px(top)), Size::new(width, height));

    painter.scene().push(fret_core::SceneOp::Quad {
        order: DrawOrder(3),
        rect,
        background: Paint::Solid(tokens.background).into(),
        border: Edges::all(Px(1.0)),
        border_paint: Paint::Solid(tokens.border).into(),
        corner_radii: Corners::all(tokens.radius),
    });
    let _ = painter.text(
        key,
        DrawOrder(3),
        Point::new(
            Px(rect.origin.x.0 + pad.0),
            Px(rect.origin.y.0 + pad.0 + metrics.baseline.0),
        ),
        text,
        text_style.clone(),
        tokens.text,
        constraints,
        raster_scale_factor,
    );

    let marker_width = Px(2.0);
    let marker_height = Px(8.0_f32.min(plot.size.height.0.max(0.0)));
    let marker_left = line_plot_clamp_plot_left(plot, x.0 - marker_width.0 * 0.5, marker_width);
    let marker_top = (plot.origin.y.0 + plot.size.height.0 - marker_height.0).max(plot.origin.y.0);
    push_filled_rect(
        painter,
        Rect::new(
            Point::new(Px(marker_left), Px(marker_top)),
            Size::new(marker_width, marker_height),
        ),
        DrawOrder(3),
        marker_color,
    );
}

#[allow(clippy::too_many_arguments)]
fn paint_line_plot_tag_y_overlay(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    y: Px,
    right: bool,
    marker_color: Color,
    key: u64,
    text: String,
    text_style: &TextStyle,
    constraints: CanvasTextConstraints,
    raster_scale_factor: f32,
    tokens: LinePlotAnnotationTokens,
) {
    let (_blob, metrics) = painter.prepare_text_with_blob(
        key,
        text.clone(),
        text_style.clone(),
        constraints,
        raster_scale_factor,
    );
    let pad = tokens.padding;
    let width = Px(metrics.size.width.0 + pad.0 * 2.0);
    let height = Px(metrics.size.height.0 + pad.0 * 2.0);
    let margin = Px(6.0);
    let left = if right {
        (plot.origin.x.0 + plot.size.width.0 - width.0 - margin.0).max(plot.origin.x.0)
    } else {
        plot.origin.x.0 + margin.0
    };
    let top = line_plot_clamp_plot_top(plot, y.0 - height.0 * 0.5, height);
    let rect = Rect::new(Point::new(Px(left), Px(top)), Size::new(width, height));

    painter.scene().push(fret_core::SceneOp::Quad {
        order: DrawOrder(3),
        rect,
        background: Paint::Solid(tokens.background).into(),
        border: Edges::all(Px(1.0)),
        border_paint: Paint::Solid(tokens.border).into(),
        corner_radii: Corners::all(tokens.radius),
    });
    let _ = painter.text(
        key,
        DrawOrder(3),
        Point::new(
            Px(rect.origin.x.0 + pad.0),
            Px(rect.origin.y.0 + pad.0 + metrics.baseline.0),
        ),
        text,
        text_style.clone(),
        tokens.text,
        constraints,
        raster_scale_factor,
    );

    let marker_height = Px(2.0);
    let marker_width = Px(8.0_f32.min(plot.size.width.0.max(0.0)));
    let marker_top = line_plot_clamp_plot_top(plot, y.0 - marker_height.0 * 0.5, marker_height);
    let marker_left = if right {
        (plot.origin.x.0 + plot.size.width.0 - marker_width.0).max(plot.origin.x.0)
    } else {
        plot.origin.x.0
    };
    push_filled_rect(
        painter,
        Rect::new(
            Point::new(Px(marker_left), Px(marker_top)),
            Size::new(marker_width, marker_height),
        ),
        DrawOrder(3),
        marker_color,
    );
}

fn line_plot_clamp_plot_left(plot: Rect, desired_left: f32, width: Px) -> f32 {
    desired_left.clamp(
        plot.origin.x.0,
        plot.origin.x.0 + (plot.size.width.0 - width.0).max(0.0),
    )
}

fn line_plot_clamp_plot_top(plot: Rect, desired_top: f32, height: Px) -> f32 {
    desired_top.clamp(
        plot.origin.y.0,
        plot.origin.y.0 + (plot.size.height.0 - height.0).max(0.0),
    )
}

pub(super) fn paint_line_plot_text_overlays(
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
            painter.scene().push(fret_core::SceneOp::Quad {
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
