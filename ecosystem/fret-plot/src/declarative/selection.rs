//! Declarative line-plot selection overlay paint and tooltip owner.

use fret_core::{
    Color, Corners, DrawOrder, Edges, FontWeight, Paint, Point, Px, Rect, Size, TextOverflow,
    TextStyle, TextWrap,
};
use fret_ui::canvas::{CanvasPainter, CanvasTextConstraints};

use crate::cartesian::{AxisScale, DataPoint, DataRect, PlotTransform};
use crate::plot::axis::AxisLabelFormatter;
use crate::style::LinePlotStyle;

use super::interaction::{
    LinePlotSelectionKind, LinePlotSelectionOverlay, line_plot_query_rect_from_plot_points_raw,
};
use super::style_helpers::axis_tick_label_text;

pub(super) fn paint_line_plot_query_selection(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    view_bounds: DataRect,
    query_selection: Option<DataRect>,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    let Some(query) = query_selection else {
        return;
    };
    let transform = PlotTransform {
        viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size),
        data: view_bounds,
        x_scale,
        y_scale,
    };
    let a = transform.data_to_px(DataPoint {
        x: query.x_min,
        y: query.y_min,
    });
    let b = transform.data_to_px(DataPoint {
        x: query.x_max,
        y: query.y_max,
    });
    paint_line_plot_selection_rect_from_local(painter, plot, a, b, style);
}

pub(super) fn paint_line_plot_active_selection(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    active_selection: Option<LinePlotSelectionOverlay>,
    style: LinePlotStyle,
) {
    let Some(selection) = active_selection else {
        return;
    };
    paint_line_plot_selection_rect_from_local(
        painter,
        plot,
        selection.start,
        selection.current,
        style,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_line_plot_selection_tooltip(
    painter: &mut CanvasPainter<'_>,
    bounds: Rect,
    plot: Rect,
    view_bounds: DataRect,
    active_selection: Option<LinePlotSelectionOverlay>,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> bool {
    let Some(selection) = active_selection else {
        return false;
    };
    let Some(text) =
        line_plot_selection_tooltip_text(view_bounds, plot.size, selection, x_scale, y_scale)
    else {
        return false;
    };

    let theme = painter.theme().snapshot();
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
    let text_style = TextStyle {
        size: Px(12.0),
        weight: FontWeight::NORMAL,
        ..TextStyle::default()
    };
    let constraints = CanvasTextConstraints {
        max_width: Some(Px(bounds.size.width.0.max(24.0))),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let raster_scale_factor = painter.scale_factor();
    let scope = painter.key_scope(&"fret-plot.declarative.selection-tooltip");
    let text_key: u64 = painter
        .child_key(
            scope,
            &(
                "text",
                line_plot_selection_kind_label(selection.kind),
                text.as_str(),
            ),
        )
        .into();
    let (_blob, metrics) = painter.prepare_text_with_blob(
        text_key,
        text.clone(),
        text_style.clone(),
        constraints,
        raster_scale_factor,
    );

    let pad = Px(6.0);
    let tooltip_size = Size::new(
        Px(metrics.size.width.0 + pad.0 * 2.0),
        Px(metrics.size.height.0 + pad.0 * 2.0),
    );
    let Some(rect) =
        line_plot_selection_tooltip_rect(bounds, plot, selection.current, tooltip_size)
    else {
        return false;
    };

    painter.scene().push(fret_core::SceneOp::Quad {
        order: DrawOrder(20),
        rect,
        background: Paint::Solid(tooltip_background).into(),
        border: Edges::all(Px(1.0)),
        border_paint: Paint::Solid(tooltip_border).into(),
        corner_radii: Corners::all(Px(6.0)),
    });
    let _ = painter.text(
        text_key,
        DrawOrder(21),
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
    true
}

fn line_plot_selection_tooltip_text(
    view_bounds: DataRect,
    plot_size: Size,
    selection: LinePlotSelectionOverlay,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Option<String> {
    let rect = line_plot_query_rect_from_plot_points_raw(
        view_bounds,
        plot_size,
        selection.start,
        selection.current,
        x_scale,
        y_scale,
    )?;
    let x_span = (view_bounds.x_max - view_bounds.x_min).abs();
    let y_span = (view_bounds.y_max - view_bounds.y_min).abs();
    let formatter = AxisLabelFormatter::default();
    let x0 = axis_tick_label_text(x_scale, &formatter, rect.x_min, x_span);
    let x1 = axis_tick_label_text(x_scale, &formatter, rect.x_max, x_span);
    let y0 = axis_tick_label_text(y_scale, &formatter, rect.y_min, y_span);
    let y1 = axis_tick_label_text(y_scale, &formatter, rect.y_max, y_span);
    Some(format!(
        "{}\nx=[{x0}, {x1}]\ny=[{y0}, {y1}]",
        line_plot_selection_kind_label(selection.kind)
    ))
}

fn line_plot_selection_kind_label(kind: LinePlotSelectionKind) -> &'static str {
    match kind {
        LinePlotSelectionKind::Query => "query",
        LinePlotSelectionKind::BoxZoom => "zoom",
    }
}

fn line_plot_selection_tooltip_rect(
    bounds: Rect,
    plot: Rect,
    anchor_local: Point,
    size: Size,
) -> Option<Rect> {
    if bounds.size.width.0 <= 0.0 || bounds.size.height.0 <= 0.0 {
        return None;
    }
    if size.width.0 <= 0.0 || size.height.0 <= 0.0 {
        return None;
    }

    let anchor = Point::new(
        Px(plot.origin.x.0 + anchor_local.x.0),
        Px(plot.origin.y.0 + anchor_local.y.0),
    );
    let gap = 10.0;
    let mut x = anchor.x.0 + gap;
    let mut y = anchor.y.0 + gap;
    let bounds_right = bounds.origin.x.0 + bounds.size.width.0;
    let bounds_bottom = bounds.origin.y.0 + bounds.size.height.0;
    if x + size.width.0 > bounds_right {
        x = anchor.x.0 - gap - size.width.0;
    }
    if y + size.height.0 > bounds_bottom {
        y = anchor.y.0 - gap - size.height.0;
    }

    let min_x = bounds.origin.x.0;
    let min_y = bounds.origin.y.0;
    let max_x = (bounds_right - size.width.0).max(min_x);
    let max_y = (bounds_bottom - size.height.0).max(min_y);
    Some(Rect::new(
        Point::new(Px(x.clamp(min_x, max_x)), Px(y.clamp(min_y, max_y))),
        size,
    ))
}

fn paint_line_plot_selection_rect_from_local(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    start: Point,
    end: Point,
    style: LinePlotStyle,
) {
    let Some(rect) = line_plot_selection_rect_from_local(plot, start, end) else {
        return;
    };
    let (selection_border, selection_fill) = line_plot_selection_colors(painter, style);
    painter.scene().push(fret_core::SceneOp::Quad {
        order: DrawOrder(5),
        rect,
        background: Paint::Solid(selection_fill).into(),
        border: Edges::all(Px(1.0)),
        border_paint: Paint::Solid(selection_border).into(),
        corner_radii: Corners::default(),
    });
}

fn line_plot_selection_rect_from_local(plot: Rect, start: Point, end: Point) -> Option<Rect> {
    let x0 = start.x.0.min(end.x.0).clamp(0.0, plot.size.width.0);
    let x1 = start.x.0.max(end.x.0).clamp(0.0, plot.size.width.0);
    let y0 = start.y.0.min(end.y.0).clamp(0.0, plot.size.height.0);
    let y1 = start.y.0.max(end.y.0).clamp(0.0, plot.size.height.0);
    let width = x1 - x0;
    let height = y1 - y0;
    (width >= 1.0 && height >= 1.0).then(|| {
        Rect::new(
            Point::new(Px(plot.origin.x.0 + x0), Px(plot.origin.y.0 + y0)),
            Size::new(Px(width), Px(height)),
        )
    })
}

fn line_plot_selection_colors(
    painter: &mut CanvasPainter<'_>,
    style: LinePlotStyle,
) -> (Color, Color) {
    let theme = painter.theme().snapshot();
    let selection_border = style
        .crosshair_color
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    let selection_fill = Color {
        a: (selection_border.a * 0.18).clamp(0.06, 0.22),
        ..selection_border
    };
    (selection_border, selection_fill)
}
