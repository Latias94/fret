//! Declarative line-plot cursor readout paint and row projection owner.

use fret_core::{
    Corners, DrawOrder, Edges, FontWeight, Paint, Point, Px, Rect, Size, TextOverflow, TextStyle,
    TextWrap,
};
use fret_ui::canvas::{CanvasPainter, CanvasTextConstraints};

use crate::cartesian::{AxisScale, DataPoint, PlotTransform};
use crate::plot::axis::AxisLabelFormatter;
use crate::plot::readout::{
    PlotCursorReadoutArgs, PlotCursorReadoutRow, PlotCursorReadoutSeries, plot_cursor_readout,
};
use crate::series::SeriesId;
use crate::state::PlotOutputSnapshot;
use crate::style::{LinePlotStyle, MouseReadoutMode, OverlayAnchor, ReadoutSeriesPolicy};

use super::PlotPanelModel;
use super::paint_primitives::{push_horizontal_line, push_vertical_line};
use super::style_helpers::axis_tick_label_text;

pub(super) fn paint_line_plot_cursor_readout(
    painter: &mut CanvasPainter<'_>,
    model: &PlotPanelModel,
    plot: Rect,
    output: Option<PlotOutputSnapshot>,
    pinned_series: Option<SeriesId>,
    hidden_series: &[SeriesId],
    style: LinePlotStyle,
    y_axis_labels: &AxisLabelFormatter,
    y2_axis_labels: &AxisLabelFormatter,
    y3_axis_labels: &AxisLabelFormatter,
    y4_axis_labels: &AxisLabelFormatter,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    let Some(snapshot) = output else {
        return;
    };
    let Some(cursor) = snapshot.cursor else {
        return;
    };
    if style.mouse_readout == MouseReadoutMode::Disabled {
        return;
    }

    let transform = PlotTransform {
        viewport: plot,
        data: snapshot.view_bounds,
        x_scale,
        y_scale,
    };
    let cursor_px = transform.data_to_px(cursor);
    if !plot.contains(cursor_px) {
        return;
    }

    let theme = painter.theme().snapshot();
    let mut crosshair_color = style
        .crosshair_color
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    crosshair_color.a = (crosshair_color.a * 0.45).clamp(0.05, 1.0);
    push_vertical_line(
        painter,
        Px(cursor_px.x.0.round()),
        plot.origin.y,
        plot.size.height,
        DrawOrder(3),
        crosshair_color,
    );
    push_horizontal_line(
        painter,
        plot.origin.x,
        Px(cursor_px.y.0.round()),
        plot.size.width,
        DrawOrder(3),
        crosshair_color,
    );

    if style.mouse_readout != MouseReadoutMode::Overlay {
        return;
    }

    let tooltip_background = style
        .tooltip_background
        .unwrap_or_else(|| theme.color_required("popover"));
    let tooltip_border = style
        .tooltip_border
        .unwrap_or_else(|| theme.color_required("border"));
    let text_color = style
        .tooltip_text_color
        .or(style.label_color)
        .unwrap_or_else(|| theme.color_required("popover-foreground"));

    let x_span = (snapshot.view_bounds.x_max - snapshot.view_bounds.x_min).abs();
    let y_span = (snapshot.view_bounds.y_max - snapshot.view_bounds.y_min).abs();
    let formatter = AxisLabelFormatter::default();
    let x_text = axis_tick_label_text(x_scale, &formatter, cursor.x, x_span);
    let y_text = axis_tick_label_text(y_scale, &formatter, cursor.y, y_span);
    let rows = line_plot_readout_rows(
        model,
        cursor.x,
        plot.size,
        snapshot.view_bounds,
        x_scale,
        y_scale,
        painter.scale_factor(),
        hidden_series,
    );
    let rows = filter_line_plot_readout_rows(rows, pinned_series, ReadoutSeriesPolicy::PinnedOrAll);
    let text = format_line_plot_readout_text(
        format!("x={x_text}  y={y_text}"),
        rows,
        y_axis_labels,
        y2_axis_labels,
        y3_axis_labels,
        y4_axis_labels,
        y_scale,
    );

    let text_style = TextStyle {
        size: Px(12.0),
        weight: FontWeight::NORMAL,
        ..TextStyle::default()
    };
    let constraints = CanvasTextConstraints {
        max_width: Some(Px(plot.size.width.0.max(24.0))),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let raster_scale_factor = painter.scale_factor();
    let scope = painter.key_scope(&"fret-plot.declarative.cursor-readout");
    let text_key: u64 = painter.child_key(scope, &("text", text.as_str())).into();
    let (_blob, metrics) = painter.prepare_text_with_blob(
        text_key,
        text.clone(),
        text_style.clone(),
        constraints,
        raster_scale_factor,
    );

    let pad = Px(6.0);
    let margin = Px(6.0);
    let overlay_size = Size::new(
        Px(metrics.size.width.0 + pad.0 * 2.0),
        Px(metrics.size.height.0 + pad.0 * 2.0),
    );
    let Some(rect) =
        overlay_rect_in_line_plot(plot, overlay_size, style.mouse_readout_anchor, margin)
    else {
        return;
    };
    painter.scene().push(fret_core::SceneOp::Quad {
        order: DrawOrder(12),
        rect,
        background: Paint::Solid(tooltip_background).into(),
        border: Edges::all(Px(1.0)),
        border_paint: Paint::Solid(tooltip_border).into(),
        corner_radii: Corners::all(Px(6.0)),
    });

    let _ = painter.text(
        text_key,
        DrawOrder(13),
        Point::new(
            Px(rect.origin.x.0 + pad.0),
            Px(rect.origin.y.0 + pad.0 + metrics.baseline.0),
        ),
        text,
        text_style,
        text_color,
        constraints,
        raster_scale_factor,
    );
}

pub(super) fn paint_line_plot_linked_cursor_readout(
    painter: &mut CanvasPainter<'_>,
    model: &PlotPanelModel,
    plot: Rect,
    view_bounds: crate::cartesian::DataRect,
    local_cursor: Option<DataPoint>,
    linked_cursor_x: Option<f64>,
    pinned_series: Option<SeriesId>,
    hidden_series: &[SeriesId],
    style: LinePlotStyle,
    y_axis_labels: &AxisLabelFormatter,
    y2_axis_labels: &AxisLabelFormatter,
    y3_axis_labels: &AxisLabelFormatter,
    y4_axis_labels: &AxisLabelFormatter,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    if local_cursor.is_some() {
        return;
    }
    let Some(linked_x) = linked_cursor_x.filter(|x| x.is_finite()) else {
        return;
    };

    let transform = PlotTransform {
        viewport: plot,
        data: view_bounds,
        x_scale,
        y_scale,
    };
    let Some(cursor_x) = transform.data_x_to_px(linked_x) else {
        return;
    };

    let theme = painter.theme().snapshot();
    let mut crosshair_color = style
        .crosshair_color
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    crosshair_color.a = (crosshair_color.a * 0.55).clamp(0.05, 1.0);
    push_vertical_line(
        painter,
        Px(cursor_x
            .0
            .clamp(plot.origin.x.0, plot.origin.x.0 + plot.size.width.0)
            .round()),
        plot.origin.y,
        plot.size.height,
        DrawOrder(3),
        crosshair_color,
    );

    if style.linked_cursor_readout != MouseReadoutMode::Overlay {
        return;
    }

    let tooltip_background = style
        .tooltip_background
        .unwrap_or_else(|| theme.color_required("popover"));
    let tooltip_border = style
        .tooltip_border
        .unwrap_or_else(|| theme.color_required("border"));
    let text_color = style
        .tooltip_text_color
        .or(style.label_color)
        .unwrap_or_else(|| theme.color_required("popover-foreground"));

    let x_span = (view_bounds.x_max - view_bounds.x_min).abs();
    let formatter = AxisLabelFormatter::default();
    let x_text = axis_tick_label_text(x_scale, &formatter, linked_x, x_span);
    let rows = line_plot_readout_rows(
        model,
        linked_x,
        plot.size,
        view_bounds,
        x_scale,
        y_scale,
        painter.scale_factor(),
        hidden_series,
    );
    let rows =
        filter_line_plot_readout_rows(rows, pinned_series, style.linked_cursor_readout_policy);
    let text = format_line_plot_readout_text(
        format!("x={x_text}"),
        rows,
        y_axis_labels,
        y2_axis_labels,
        y3_axis_labels,
        y4_axis_labels,
        y_scale,
    );

    let text_style = TextStyle {
        size: Px(12.0),
        weight: FontWeight::NORMAL,
        ..TextStyle::default()
    };
    let constraints = CanvasTextConstraints {
        max_width: Some(Px(plot.size.width.0.max(24.0))),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let raster_scale_factor = painter.scale_factor();
    let scope = painter.key_scope(&"fret-plot.declarative.linked-cursor-readout");
    let text_key: u64 = painter.child_key(scope, &("text", text.as_str())).into();
    let (_blob, metrics) = painter.prepare_text_with_blob(
        text_key,
        text.clone(),
        text_style.clone(),
        constraints,
        raster_scale_factor,
    );

    let pad = Px(6.0);
    let margin = Px(6.0);
    let overlay_size = Size::new(
        Px(metrics.size.width.0 + pad.0 * 2.0),
        Px(metrics.size.height.0 + pad.0 * 2.0),
    );
    let Some(rect) = overlay_rect_in_line_plot(
        plot,
        overlay_size,
        style.linked_cursor_readout_anchor,
        margin,
    ) else {
        return;
    };
    painter.scene().push(fret_core::SceneOp::Quad {
        order: DrawOrder(12),
        rect,
        background: Paint::Solid(tooltip_background).into(),
        border: Edges::all(Px(1.0)),
        border_paint: Paint::Solid(tooltip_border).into(),
        corner_radii: Corners::all(Px(6.0)),
    });

    let _ = painter.text(
        text_key,
        DrawOrder(13),
        Point::new(
            Px(rect.origin.x.0 + pad.0),
            Px(rect.origin.y.0 + pad.0 + metrics.baseline.0),
        ),
        text,
        text_style,
        text_color,
        constraints,
        raster_scale_factor,
    );
}

fn line_plot_readout_rows<'a>(
    model: &PlotPanelModel,
    x: f64,
    plot_size: Size,
    view_bounds: crate::cartesian::DataRect,
    x_scale: AxisScale,
    y_scale: AxisScale,
    scale_factor: f32,
    hidden_series: &'a [SeriesId],
) -> Vec<PlotCursorReadoutRow> {
    let hidden: std::collections::HashSet<SeriesId> = hidden_series.iter().copied().collect();
    let mut readout_series: Vec<PlotCursorReadoutSeries<'_>> = Vec::new();
    for series in &model.series {
        if let Some(lower_data) = &series.lower_data {
            readout_series.push(PlotCursorReadoutSeries {
                id: series.id,
                label: std::sync::Arc::from(format!("{} (upper)", series.label)),
                y_axis: series.y_axis,
                data: &*series.data,
            });
            readout_series.push(PlotCursorReadoutSeries {
                id: series.id,
                label: std::sync::Arc::from(format!("{} (lower)", series.label)),
                y_axis: series.y_axis,
                data: &**lower_data,
            });
        } else {
            readout_series.push(PlotCursorReadoutSeries {
                id: series.id,
                label: series.label.clone(),
                y_axis: series.y_axis,
                data: &*series.data,
            });
        }
    }
    plot_cursor_readout(
        readout_series,
        PlotCursorReadoutArgs {
            x,
            plot_size,
            view_bounds,
            x_scale,
            y_scale,
            scale_factor,
            hidden: &hidden,
        },
    )
}

fn format_line_plot_readout_text(
    mut text: String,
    rows: Vec<PlotCursorReadoutRow>,
    y_axis_labels: &AxisLabelFormatter,
    y2_axis_labels: &AxisLabelFormatter,
    y3_axis_labels: &AxisLabelFormatter,
    y4_axis_labels: &AxisLabelFormatter,
    y_scale: AxisScale,
) -> String {
    for row in rows {
        let (formatter, axis_label) = match row.y_axis {
            crate::models::YAxis::Left => (y_axis_labels, "y"),
            crate::models::YAxis::Right => (y2_axis_labels, "y2"),
            crate::models::YAxis::Right2 => (y3_axis_labels, "y3"),
            crate::models::YAxis::Right3 => (y4_axis_labels, "y4"),
        };
        let y_text = row
            .y
            .filter(|y| y.is_finite())
            .map(|y| axis_tick_label_text(y_scale, &formatter, y, 1.0))
            .unwrap_or_else(|| "NA".to_string());
        text.push_str(&format!("\n{}: {axis_label}={y_text}", row.label));
    }
    text
}

fn filter_line_plot_readout_rows(
    rows: Vec<PlotCursorReadoutRow>,
    pinned: Option<SeriesId>,
    policy: ReadoutSeriesPolicy,
) -> Vec<PlotCursorReadoutRow> {
    match (policy, pinned) {
        (ReadoutSeriesPolicy::PinnedOrAll, Some(pinned))
        | (ReadoutSeriesPolicy::PinnedOnly, Some(pinned))
        | (ReadoutSeriesPolicy::PinnedOrLegendHoverOrAll, Some(pinned)) => rows
            .into_iter()
            .filter(|row| row.series_id == pinned)
            .collect(),
        (ReadoutSeriesPolicy::PinnedOnly, None) => Vec::new(),
        (ReadoutSeriesPolicy::PinnedOrAll, None)
        | (ReadoutSeriesPolicy::PinnedOrLegendHoverOrAll, None) => rows,
    }
}

fn overlay_rect_in_line_plot(
    plot: Rect,
    size: Size,
    anchor: OverlayAnchor,
    margin: Px,
) -> Option<Rect> {
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return None;
    }
    if size.width.0 <= 0.0 || size.height.0 <= 0.0 {
        return None;
    }

    let w = size.width.0;
    let h = size.height.0;
    let m = margin.0.max(0.0);
    let x = match anchor {
        OverlayAnchor::TopLeft | OverlayAnchor::BottomLeft => plot.origin.x.0 + m,
        OverlayAnchor::TopRight | OverlayAnchor::BottomRight => {
            plot.origin.x.0 + plot.size.width.0 - m - w
        }
    };
    let y = match anchor {
        OverlayAnchor::TopLeft | OverlayAnchor::TopRight => plot.origin.y.0 + m,
        OverlayAnchor::BottomLeft | OverlayAnchor::BottomRight => {
            plot.origin.y.0 + plot.size.height.0 - m - h
        }
    };

    let max_x = plot.origin.x.0 + plot.size.width.0 - w;
    let max_y = plot.origin.y.0 + plot.size.height.0 - h;
    Some(Rect::new(
        Point::new(
            Px(x.clamp(plot.origin.x.0, max_x)),
            Px(y.clamp(plot.origin.y.0, max_y)),
        ),
        size,
    ))
}
