//! Declarative line-plot annotation overlay helper owner.

use fret_core::{
    Color, Corners, DrawOrder, Edges, Paint, Point, Px, Rect, SceneOp, Size, TextStyle,
};
use fret_ui::canvas::{CanvasPainter, CanvasTextConstraints};

use crate::style::LinePlotStyle;

use super::super::paint_primitives::push_filled_rect;

#[derive(Debug, Clone, Copy)]
pub(in crate::declarative) struct LinePlotAnnotationTokens {
    pub(in crate::declarative) background: Color,
    pub(in crate::declarative) border: Color,
    pub(in crate::declarative) text: Color,
    pub(in crate::declarative) stroke: Color,
    pub(in crate::declarative) padding: Px,
    pub(in crate::declarative) radius: Px,
}

pub(in crate::declarative) fn line_plot_annotation_tokens(
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

pub(in crate::declarative) fn line_plot_annotation_label(
    label: Option<&str>,
    value: Option<String>,
) -> String {
    match (label, value) {
        (Some(label), Some(value)) => format!("{label}: {value}"),
        (Some(label), None) => label.to_owned(),
        (None, Some(value)) => value,
        (None, None) => String::new(),
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::declarative) fn paint_line_plot_annotation_text_box(
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
        painter.scene().push(SceneOp::Quad {
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
pub(in crate::declarative) fn paint_line_plot_tag_x_overlay(
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

    painter.scene().push(SceneOp::Quad {
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
pub(in crate::declarative) fn paint_line_plot_tag_y_overlay(
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

    painter.scene().push(SceneOp::Quad {
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

pub(in crate::declarative) fn line_plot_clamp_plot_left(
    plot: Rect,
    desired_left: f32,
    width: Px,
) -> f32 {
    desired_left.clamp(
        plot.origin.x.0,
        plot.origin.x.0 + (plot.size.width.0 - width.0).max(0.0),
    )
}

pub(in crate::declarative) fn line_plot_clamp_plot_top(
    plot: Rect,
    desired_top: f32,
    height: Px,
) -> f32 {
    desired_top.clamp(
        plot.origin.y.0,
        plot.origin.y.0 + (plot.size.height.0 - height.0).max(0.0),
    )
}
