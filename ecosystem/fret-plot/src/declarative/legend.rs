//! Declarative line-plot legend paint and hit-test owner.

use fret_core::{
    Color, Corners, DrawOrder, Edges, FontWeight, Paint, Point, Px, Rect, Size, TextOverflow,
    TextStyle, TextWrap,
};
use fret_ui::canvas::{CanvasPainter, CanvasTextConstraints};

use crate::series::SeriesId;
use crate::style::LinePlotStyle;

use super::{PlotPanelModel, series_color};

pub(super) fn paint_line_plot_legend(
    painter: &mut CanvasPainter<'_>,
    model: &PlotPanelModel,
    plot: Rect,
    pinned_series: Option<SeriesId>,
    legend_hover: Option<SeriesId>,
    style: LinePlotStyle,
) {
    if model.series.is_empty() || plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return;
    }

    let theme = painter.theme().snapshot();
    let text_color = style
        .label_color
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    let text_style = TextStyle {
        size: Px(12.0),
        weight: FontWeight::NORMAL,
        ..TextStyle::default()
    };
    let text_constraints = CanvasTextConstraints {
        max_width: Some(Px((plot.size.width.0 - 36.0).max(24.0))),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };

    let series_count = model.series.len();
    let metrics = line_plot_legend_metrics();
    let scope = painter.key_scope(&"fret-plot.declarative.legend");
    let raster_scale_factor = painter.scale_factor();

    for (index, series) in model.series.iter().enumerate() {
        let Some(row) = line_plot_legend_row_rect(plot, index) else {
            break;
        };
        let swatch_rect = line_plot_legend_swatch_rect(row);
        if legend_hover == Some(series.id) || pinned_series == Some(series.id) {
            let mut highlight = style
                .crosshair_color
                .unwrap_or_else(|| theme.color_required("muted-foreground"));
            highlight.a *= if pinned_series == Some(series.id) {
                0.16
            } else {
                0.10
            };
            let inset_x = Px(2.0);
            painter.scene().push(fret_core::SceneOp::Quad {
                order: DrawOrder(29),
                rect: Rect::new(
                    Point::new(Px(row.origin.x.0 + inset_x.0), row.origin.y),
                    Size::new(
                        Px((row.size.width.0 - inset_x.0 * 2.0).max(0.0)),
                        row.size.height,
                    ),
                ),
                background: Paint::Solid(highlight).into(),
                border: Edges::default(),
                border_paint: Paint::Solid(Color::TRANSPARENT).into(),
                corner_radii: Corners::all(Px(4.0)),
            });
        }

        let color = series
            .stroke_color
            .unwrap_or_else(|| series_color(style, index, series_count));
        painter.scene().push(fret_core::SceneOp::Quad {
            order: DrawOrder(30),
            rect: swatch_rect,
            background: Paint::Solid(color).into(),
            border: Edges::default(),
            border_paint: Paint::Solid(Color::TRANSPARENT).into(),
            corner_radii: Corners::default(),
        });

        let key: u64 = painter
            .child_key(scope, &("series", series.id.0, series.label.as_ref()))
            .into();
        let _ = painter.text(
            key,
            DrawOrder(31),
            Point::new(
                Px(swatch_rect.origin.x.0 + swatch_rect.size.width.0 + metrics.gap.0),
                Px(row.origin.y.0 + metrics.text_baseline_offset.0),
            ),
            series.label.clone(),
            text_style.clone(),
            text_color,
            text_constraints,
            raster_scale_factor,
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LinePlotLegendHit {
    Swatch,
    Label,
}

#[derive(Debug, Clone, Copy)]
struct LinePlotLegendMetrics {
    row_height: Px,
    swatch: Size,
    gap: Px,
    inset: Px,
    text_baseline_offset: Px,
}

fn line_plot_legend_metrics() -> LinePlotLegendMetrics {
    LinePlotLegendMetrics {
        row_height: Px(18.0),
        swatch: Size::new(Px(12.0), Px(3.0)),
        gap: Px(6.0),
        inset: Px(8.0),
        text_baseline_offset: Px(12.0),
    }
}

fn line_plot_legend_row_rect(plot: Rect, index: usize) -> Option<Rect> {
    let metrics = line_plot_legend_metrics();
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return None;
    }
    let y = Px(plot.origin.y.0 + metrics.inset.0 + index as f32 * metrics.row_height.0);
    let max_y = plot.origin.y.0 + plot.size.height.0 - metrics.inset.0;
    if y.0 + metrics.row_height.0 > max_y {
        return None;
    }
    Some(Rect::new(
        Point::new(Px(plot.origin.x.0 + metrics.inset.0), y),
        Size::new(
            Px((plot.size.width.0 - metrics.inset.0 * 2.0).max(0.0)),
            metrics.row_height,
        ),
    ))
}

fn line_plot_legend_swatch_rect(row: Rect) -> Rect {
    let metrics = line_plot_legend_metrics();
    let row_mid = row.origin.y.0 + row.size.height.0 * 0.5;
    Rect::new(
        Point::new(row.origin.x, Px(row_mid - metrics.swatch.height.0 * 0.5)),
        metrics.swatch,
    )
}

fn line_plot_legend_swatch_hit_rect(row: Rect) -> Rect {
    let metrics = line_plot_legend_metrics();
    Rect::new(row.origin, Size::new(metrics.swatch.width, row.size.height))
}

pub(super) fn line_plot_legend_hit(
    model: &PlotPanelModel,
    plot: Rect,
    position: Point,
) -> Option<(SeriesId, LinePlotLegendHit)> {
    if model.series.is_empty() {
        return None;
    }
    for (index, series) in model.series.iter().enumerate() {
        let row = line_plot_legend_row_rect(plot, index)?;
        if !row.contains(position) {
            continue;
        }
        let hit = if line_plot_legend_swatch_hit_rect(row).contains(position) {
            LinePlotLegendHit::Swatch
        } else {
            LinePlotLegendHit::Label
        };
        return Some((series.id, hit));
    }
    None
}
