//! Declarative heatmap and colorbar paint owner.

use fret_core::{
    Color, Corners, DrawOrder, Edges, FontWeight, Paint, Point, Px, Rect, Size, TextOverflow,
    TextStyle, TextWrap,
};
use fret_ui::canvas::{CanvasPainter, CanvasTextConstraints};

use crate::cartesian::PlotTransform;
use crate::style::LinePlotStyle;

use super::{model::PlotPanelHeatmap, push_filled_rect};

pub(super) fn paint_line_plot_heatmap(
    painter: &mut CanvasPainter<'_>,
    transform: PlotTransform,
    heatmap: &PlotPanelHeatmap,
    style: LinePlotStyle,
) {
    if heatmap.cols == 0 || heatmap.rows == 0 || heatmap.values.is_empty() {
        return;
    }
    let Some(prepared) = transform.prepare() else {
        return;
    };

    let dx = (heatmap.data_bounds.x_max - heatmap.data_bounds.x_min) / heatmap.cols as f64;
    let dy = (heatmap.data_bounds.y_max - heatmap.data_bounds.y_min) / heatmap.rows as f64;
    if !dx.is_finite() || !dy.is_finite() || dx <= 0.0 || dy <= 0.0 {
        return;
    }

    let view_x_min = transform.data.x_min.min(transform.data.x_max);
    let view_x_max = transform.data.x_min.max(transform.data.x_max);
    let view_y_min = transform.data.y_min.min(transform.data.y_max);
    let view_y_max = transform.data.y_min.max(transform.data.y_max);
    let clip_min_x = view_x_min.max(heatmap.data_bounds.x_min);
    let clip_max_x = view_x_max.min(heatmap.data_bounds.x_max);
    let clip_min_y = view_y_min.max(heatmap.data_bounds.y_min);
    let clip_max_y = view_y_max.min(heatmap.data_bounds.y_max);
    if clip_max_x <= clip_min_x || clip_max_y <= clip_min_y {
        return;
    }

    let col0 = (((clip_min_x - heatmap.data_bounds.x_min) / dx).floor() as isize)
        .clamp(0, heatmap.cols.saturating_sub(1) as isize) as usize;
    let col1 = (((clip_max_x - heatmap.data_bounds.x_min) / dx).ceil() as isize)
        .clamp(0, heatmap.cols as isize) as usize;
    let row0 = (((clip_min_y - heatmap.data_bounds.y_min) / dy).floor() as isize)
        .clamp(0, heatmap.rows.saturating_sub(1) as isize) as usize;
    let row1 = (((clip_max_y - heatmap.data_bounds.y_min) / dy).ceil() as isize)
        .clamp(0, heatmap.rows as isize) as usize;

    let denom = (heatmap.value_max - heatmap.value_min).max(1.0e-12);
    for row in row0..row1 {
        let y0 = heatmap.data_bounds.y_min + row as f64 * dy;
        let y1 = heatmap.data_bounds.y_min + row.saturating_add(1) as f64 * dy;
        let (Some(py0), Some(py1)) = (prepared.data_y_to_px(y0), prepared.data_y_to_px(y1)) else {
            continue;
        };
        let top = py0.0.min(py1.0);
        let bottom = py0.0.max(py1.0);
        if !top.is_finite() || !bottom.is_finite() || bottom <= top {
            continue;
        }

        for col in col0..col1 {
            let idx = row.saturating_mul(heatmap.cols).saturating_add(col);
            let Some(value) = heatmap.values.get(idx).copied() else {
                continue;
            };
            if !value.is_finite() {
                continue;
            }

            let x0 = heatmap.data_bounds.x_min + col as f64 * dx;
            let x1 = heatmap.data_bounds.x_min + col.saturating_add(1) as f64 * dx;
            let (Some(px0), Some(px1)) = (prepared.data_x_to_px(x0), prepared.data_x_to_px(x1))
            else {
                continue;
            };
            let left = px0.0.min(px1.0);
            let right = px0.0.max(px1.0);
            if !left.is_finite() || !right.is_finite() || right <= left {
                continue;
            }

            let t = ((value - heatmap.value_min) / denom).clamp(0.0, 1.0);
            let color = crate::plot::colormap::sample(style.heatmap_colormap, t);
            push_filled_rect(
                painter,
                Rect::new(
                    Point::new(Px(left), Px(top)),
                    Size::new(Px(right - left), Px(bottom - top)),
                ),
                DrawOrder(2),
                color,
            );
        }
    }
}

fn format_heatmap_colorbar_value(value: f32) -> String {
    if !value.is_finite() {
        return "NA".to_string();
    }
    let abs = value.abs();
    if abs > 1.0e6 || (abs > 0.0 && abs < 1.0e-3) {
        return format!("{value:.3e}");
    }
    if abs >= 1000.0 {
        return format!("{value:.0}");
    }
    if abs >= 10.0 {
        return format!("{value:.2}");
    }
    format!("{value:.3}")
}

pub(super) fn paint_line_plot_heatmap_colorbar(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    heatmap: &PlotPanelHeatmap,
    style: LinePlotStyle,
) {
    if !style.heatmap_show_colorbar
        || !heatmap.value_min.is_finite()
        || !heatmap.value_max.is_finite()
        || heatmap.value_max <= heatmap.value_min
    {
        return;
    }

    let padding = style.heatmap_colorbar_padding.0.max(0.0);
    let bar_width = style.heatmap_colorbar_width.0.max(1.0);
    let steps = style.heatmap_colorbar_steps.clamp(8, 512);
    let bar_height = (plot.size.height.0 - padding * 2.0).max(0.0);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 || bar_height < 24.0 {
        return;
    }

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
        size: Px(11.0),
        weight: FontWeight::NORMAL,
        ..TextStyle::default()
    };
    let constraints = CanvasTextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let raster_scale_factor = painter.scale_factor();
    let scope = painter.key_scope(&"fret-plot.declarative.heatmap-colorbar");
    let max_label = format_heatmap_colorbar_value(heatmap.value_max);
    let min_label = format_heatmap_colorbar_value(heatmap.value_min);
    let max_key: u64 = painter
        .child_key(scope, &("max", max_label.as_str()))
        .into();
    let min_key: u64 = painter
        .child_key(scope, &("min", min_label.as_str()))
        .into();
    let (_max_blob, max_metrics) = painter.prepare_text_with_blob(
        max_key,
        max_label.clone(),
        text_style.clone(),
        constraints,
        raster_scale_factor,
    );
    let (_min_blob, min_metrics) = painter.prepare_text_with_blob(
        min_key,
        min_label.clone(),
        text_style.clone(),
        constraints,
        raster_scale_factor,
    );

    let label_gap = 6.0_f32;
    let label_width = max_metrics.size.width.0.max(min_metrics.size.width.0);
    let panel_width = (bar_width + label_gap + label_width).max(bar_width);
    let panel_left = (plot.origin.x.0 + plot.size.width.0 - padding - panel_width)
        .max(plot.origin.x.0 + padding);
    let panel_top = plot.origin.y.0 + padding;
    let bar_left = panel_left;
    let bar_top = panel_top;
    let label_x = bar_left + bar_width + label_gap;

    painter.scene().push(fret_core::SceneOp::Quad {
        order: DrawOrder(3),
        rect: Rect::new(
            Point::new(Px(panel_left), Px(panel_top)),
            Size::new(Px(panel_width), Px(bar_height)),
        ),
        background: Paint::Solid(Color {
            a: 0.88,
            ..tooltip_background
        })
        .into(),
        border: Edges::all(Px(1.0)),
        border_paint: Paint::Solid(tooltip_border).into(),
        corner_radii: Corners::all(Px(6.0)),
    });

    for index in 0..steps {
        let t0 = index as f32 / steps as f32;
        let t1 = index.saturating_add(1) as f32 / steps as f32;
        let t = (t0 + t1) * 0.5;
        let y0 = bar_top + (1.0 - t1) * bar_height;
        let height = ((t1 - t0) * bar_height).max(1.0);
        let color = crate::plot::colormap::sample(style.heatmap_colormap, t);
        push_filled_rect(
            painter,
            Rect::new(
                Point::new(Px(bar_left), Px(y0)),
                Size::new(Px(bar_width), Px(height)),
            ),
            DrawOrder(4),
            color,
        );
    }

    painter.scene().push(fret_core::SceneOp::Quad {
        order: DrawOrder(5),
        rect: Rect::new(
            Point::new(Px(bar_left), Px(bar_top)),
            Size::new(Px(bar_width), Px(bar_height)),
        ),
        background: Paint::TRANSPARENT.into(),
        border: Edges::all(Px(1.0)),
        border_paint: Paint::Solid(tooltip_border).into(),
        corner_radii: Corners::default(),
    });

    let text_margin = 2.0_f32;
    let _ = painter.text(
        max_key,
        DrawOrder(6),
        Point::new(
            Px(label_x),
            Px(bar_top + text_margin + max_metrics.baseline.0),
        ),
        max_label,
        text_style.clone(),
        text_color,
        constraints,
        raster_scale_factor,
    );
    let _ = painter.text(
        min_key,
        DrawOrder(6),
        Point::new(
            Px(label_x),
            Px(
                bar_top + bar_height - text_margin - min_metrics.size.height.0
                    + min_metrics.baseline.0,
            ),
        ),
        min_label,
        text_style,
        text_color,
        constraints,
        raster_scale_factor,
    );
}
