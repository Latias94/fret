use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use delinea::engine::AxisPointerOutput;
use delinea::ids::SeriesId;
use fret_core::{Color, Corners, DrawOrder, Edges, Point, Px, Rect, Size};
use fret_ui::canvas::{CanvasPainter, CanvasTextConstraints};
use fret_ui_kit::recipes::canvas_pan_zoom::PanZoomCanvasPaintCx;
use fret_ui_kit::recipes::canvas_tool_router::{CanvasToolEntry, CanvasToolHandlers, CanvasToolId};

use crate::retained::ChartStyle;
use crate::tooltip_layout::split_tooltip_text_for_columns;
use crate::{TooltipTextLine, TooltipTextLineKind};

#[derive(Debug, Clone)]
pub(crate) struct AxisPointerLabelOverlay {
    pub axis_kind: delinea::AxisKind,
    pub text: Arc<str>,
}

#[derive(Debug, Default)]
pub(crate) struct TooltipOverlayState {
    pub axis_pointer: Option<AxisPointerOutput>,
    pub axis_pointer_labels: Vec<AxisPointerLabelOverlay>,
    pub lines: Vec<TooltipTextLine>,
    pub series_rank_by_id: BTreeMap<SeriesId, usize>,
}

pub(crate) fn tooltip_overlay_tool(
    state: Arc<Mutex<TooltipOverlayState>>,
    style: ChartStyle,
) -> CanvasToolEntry {
    let paint_state = state.clone();
    let on_paint = Arc::new(
        move |painter: &mut CanvasPainter<'_>, paint_cx: PanZoomCanvasPaintCx| {
            let Ok(state) = paint_state.lock() else {
                return;
            };

            let Some(axis_pointer) = state.axis_pointer.as_ref() else {
                return;
            };

            let bounds = painter.bounds();
            if bounds.size.width.0 <= 0.0 || bounds.size.height.0 <= 0.0 {
                return;
            }

            let crosshair = axis_pointer.crosshair_px;

            let mut axis_pointer_label_rect: Option<Rect> = None;
            if !state.axis_pointer_labels.is_empty() {
                let label_scope = painter.key_scope(&"fret-chart.declarative.axis_pointer.label");

                let text_style = fret_core::TextStyle {
                    size: Px(11.0),
                    weight: fret_core::FontWeight::MEDIUM,
                    ..fret_core::TextStyle::default()
                };
                let constraints = CanvasTextConstraints {
                    max_width: None,
                    wrap: fret_core::TextWrap::None,
                    overflow: fret_core::TextOverflow::Clip,
                };

                let bx0 = bounds.origin.x.0;
                let by0 = bounds.origin.y.0;
                let bx1 = bx0 + bounds.size.width.0;
                let by1 = by0 + bounds.size.height.0;

                let pad_x = 6.0f32;
                let pad_y = 3.0f32;

                let union = |a: Rect, b: Rect| -> Rect {
                    let ax0 = a.origin.x.0;
                    let ay0 = a.origin.y.0;
                    let ax1 = ax0 + a.size.width.0;
                    let ay1 = ay0 + a.size.height.0;

                    let bx0 = b.origin.x.0;
                    let by0 = b.origin.y.0;
                    let bx1 = bx0 + b.size.width.0;
                    let by1 = by0 + b.size.height.0;

                    let x0 = ax0.min(bx0);
                    let y0 = ay0.min(by0);
                    let x1 = ax1.max(bx1);
                    let y1 = ay1.max(by1);

                    Rect::new(
                        Point::new(Px(x0), Px(y0)),
                        Size::new(Px((x1 - x0).max(0.0)), Px((y1 - y0).max(0.0))),
                    )
                };

                for label in &state.axis_pointer_labels {
                    let text: Arc<str> = label.text.clone();
                    let kind_key: u8 = match label.axis_kind {
                        delinea::AxisKind::X => 0,
                        delinea::AxisKind::Y => 1,
                    };
                    let key: u64 = painter
                        .child_key(label_scope, &("label", kind_key, text.as_ref()))
                        .into();
                    let metrics = painter.text(
                        key,
                        DrawOrder(0),
                        Point::new(Px(0.0), Px(0.0)),
                        text.clone(),
                        text_style.clone(),
                        Color::TRANSPARENT,
                        constraints,
                        paint_cx.raster_scale_factor,
                    );

                    let w = (metrics.size.width.0 + 2.0 * pad_x).max(1.0);
                    let h = (metrics.size.height.0 + 2.0 * pad_y).max(1.0);

                    let (mut box_x, mut box_y) = match label.axis_kind {
                        delinea::AxisKind::X => (crosshair.x.0 - 0.5 * w, by1 - h),
                        delinea::AxisKind::Y => (bx0, crosshair.y.0 - 0.5 * h),
                    };
                    box_x = box_x.clamp(bx0, (bx1 - w).max(bx0));
                    box_y = box_y.clamp(by0, (by1 - h).max(by0));

                    let rect = Rect::new(Point::new(Px(box_x), Px(box_y)), Size::new(Px(w), Px(h)));
                    axis_pointer_label_rect = Some(match axis_pointer_label_rect {
                        Some(prev) => union(prev, rect),
                        None => rect,
                    });

                    let label_order = DrawOrder(
                        style
                            .draw_order
                            .0
                            .saturating_add(9_020 + (kind_key as u32).saturating_mul(4)),
                    );
                    painter.scene().push(fret_core::SceneOp::Quad {
                        order: label_order,
                        rect,
                        background: fret_core::Paint::Solid(style.tooltip_background),

                        border: Edges::all(style.tooltip_border_width),
                        border_paint: fret_core::Paint::Solid(style.tooltip_border_color),

                        corner_radii: Corners::all(Px(4.0)),
                    });
                    let _ = painter.text(
                        key,
                        DrawOrder(label_order.0.saturating_add(1)),
                        Point::new(Px(box_x + pad_x), Px(box_y + pad_y)),
                        text,
                        text_style.clone(),
                        style.tooltip_text_color,
                        constraints,
                        paint_cx.raster_scale_factor,
                    );
                }
            }

            // Tooltip box.
            if state.lines.is_empty() {
                return;
            }

            let text_style = fret_core::TextStyle {
                size: Px(12.0),
                weight: fret_core::FontWeight::NORMAL,
                ..fret_core::TextStyle::default()
            };
            let mut header_text_style = text_style.clone();
            header_text_style.weight = fret_core::FontWeight::BOLD;
            let mut value_text_style = text_style.clone();
            value_text_style.weight = fret_core::FontWeight::MEDIUM;
            let constraints = CanvasTextConstraints {
                max_width: None,
                wrap: fret_core::TextWrap::None,
                overflow: fret_core::TextOverflow::Clip,
            };

            let pad = style.tooltip_padding;
            let swatch_w = style.tooltip_marker_size.0.max(0.0);
            let swatch_gap = style.tooltip_marker_gap.0.max(0.0);
            let col_gap = style.tooltip_column_gap.0.max(0.0);
            let reserve_swatch =
                swatch_w > 0.0 && state.lines.iter().any(|l| l.source_series.is_some());
            let swatch_space = if reserve_swatch {
                (swatch_w + swatch_gap).max(0.0)
            } else {
                0.0
            };

            enum TooltipLineLayout {
                Single {
                    key: u64,
                    metrics: fret_core::TextMetrics,
                    text: Arc<str>,
                },
                Columns {
                    left_key: u64,
                    left_metrics: fret_core::TextMetrics,
                    left: Arc<str>,
                    right_key: u64,
                    right_metrics: fret_core::TextMetrics,
                    right: Arc<str>,
                },
            }

            struct PreparedTooltipLine {
                source_series: Option<SeriesId>,
                kind: TooltipTextLineKind,
                value_emphasis: bool,
                is_missing: bool,
                layout: TooltipLineLayout,
            }

            let scope = painter.key_scope(&"fret-chart.declarative.tooltip");

            let mut prepared_lines = Vec::with_capacity(state.lines.len());
            let mut max_left_w = 0.0f32;
            let mut max_right_w = 0.0f32;
            let mut max_single_w = 0.0f32;
            let mut total_h = 0.0f32;

            for (i, line) in state.lines.iter().enumerate() {
                let label_style_for_line = if line.kind == TooltipTextLineKind::AxisHeader {
                    header_text_style.clone()
                } else {
                    text_style.clone()
                };
                let value_style_for_line =
                    if line.value_emphasis && line.kind != TooltipTextLineKind::AxisHeader {
                        value_text_style.clone()
                    } else {
                        label_style_for_line.clone()
                    };

                let columns = line
                    .columns
                    .as_ref()
                    .map(|(left, right)| (left.as_str(), right.as_str()))
                    .or_else(|| split_tooltip_text_for_columns(&line.text));

                if let Some((left, right)) = columns {
                    let left: Arc<str> = left.into();
                    let right: Arc<str> = right.into();

                    let kind_key: u8 = match line.kind {
                        TooltipTextLineKind::Body => 0,
                        TooltipTextLineKind::AxisHeader => 1,
                        TooltipTextLineKind::SeriesRow => 2,
                    };
                    let emphasis_key: u8 = line.value_emphasis as u8;
                    let missing_key: u8 = line.is_missing as u8;
                    let left_key: u64 = painter
                        .child_key(scope, &("l", kind_key, emphasis_key, i, left.as_ref()))
                        .into();
                    let right_key: u64 = painter
                        .child_key(
                            scope,
                            &("r", kind_key, emphasis_key, missing_key, i, right.as_ref()),
                        )
                        .into();

                    let left_metrics = painter.text(
                        left_key,
                        DrawOrder(0),
                        Point::new(Px(0.0), Px(0.0)),
                        left.clone(),
                        label_style_for_line.clone(),
                        Color::TRANSPARENT,
                        constraints,
                        paint_cx.raster_scale_factor,
                    );
                    let right_metrics = painter.text(
                        right_key,
                        DrawOrder(0),
                        Point::new(Px(0.0), Px(0.0)),
                        right.clone(),
                        value_style_for_line.clone(),
                        Color::TRANSPARENT,
                        constraints,
                        paint_cx.raster_scale_factor,
                    );

                    max_left_w = max_left_w.max(left_metrics.size.width.0);
                    max_right_w = max_right_w.max(right_metrics.size.width.0);
                    total_h += left_metrics
                        .size
                        .height
                        .0
                        .max(right_metrics.size.height.0)
                        .max(1.0);

                    prepared_lines.push(PreparedTooltipLine {
                        source_series: line.source_series,
                        kind: line.kind,
                        value_emphasis: line.value_emphasis,
                        is_missing: line.is_missing,
                        layout: TooltipLineLayout::Columns {
                            left_key,
                            left_metrics,
                            left,
                            right_key,
                            right_metrics,
                            right,
                        },
                    });
                } else {
                    let text: Arc<str> = line.text.as_str().into();
                    let kind_key: u8 = match line.kind {
                        TooltipTextLineKind::Body => 0,
                        TooltipTextLineKind::AxisHeader => 1,
                        TooltipTextLineKind::SeriesRow => 2,
                    };
                    let emphasis_key: u8 = line.value_emphasis as u8;
                    let missing_key: u8 = line.is_missing as u8;
                    let key: u64 = painter
                        .child_key(
                            scope,
                            &("s", kind_key, emphasis_key, missing_key, i, text.as_ref()),
                        )
                        .into();
                    let metrics = painter.text(
                        key,
                        DrawOrder(0),
                        Point::new(Px(0.0), Px(0.0)),
                        text.clone(),
                        label_style_for_line.clone(),
                        Color::TRANSPARENT,
                        constraints,
                        paint_cx.raster_scale_factor,
                    );
                    max_single_w = max_single_w.max(metrics.size.width.0);
                    total_h += metrics.size.height.0.max(1.0);
                    prepared_lines.push(PreparedTooltipLine {
                        source_series: line.source_series,
                        kind: line.kind,
                        value_emphasis: line.value_emphasis,
                        is_missing: line.is_missing,
                        layout: TooltipLineLayout::Single { key, metrics, text },
                    });
                }
            }

            let mut w = 1.0f32;
            if max_left_w > 0.0 || max_right_w > 0.0 {
                w = w.max(max_left_w + col_gap + max_right_w);
            }
            w = w.max(max_single_w);
            w = (w + swatch_space + pad.left.0 + pad.right.0).max(1.0);
            let h = (total_h + pad.top.0 + pad.bottom.0).max(1.0);

            let anchor = match &axis_pointer.tooltip {
                delinea::TooltipOutput::Axis(_) => axis_pointer.crosshair_px,
                delinea::TooltipOutput::Item(_) => axis_pointer
                    .hit
                    .map(|h| h.point_px)
                    .unwrap_or(axis_pointer.crosshair_px),
            };

            let offset = 10.0f32;
            let tooltip_rect = crate::tooltip_layout::place_tooltip_rect(
                bounds,
                anchor,
                Size::new(Px(w), Px(h)),
                offset,
                axis_pointer_label_rect,
            );
            let tip_x = tooltip_rect.origin.x.0;
            let tip_y = tooltip_rect.origin.y.0;

            let tooltip_order = DrawOrder(style.draw_order.0.saturating_add(9_100));
            painter.scene().push(fret_core::SceneOp::Quad {
                order: tooltip_order,
                rect: Rect::new(Point::new(Px(tip_x), Px(tip_y)), Size::new(Px(w), Px(h))),
                background: fret_core::Paint::Solid(style.tooltip_background),

                border: Edges::all(style.tooltip_border_width),
                border_paint: fret_core::Paint::Solid(style.tooltip_border_color),

                corner_radii: Corners::all(style.tooltip_corner_radius),
            });

            let mut y = tip_y + pad.top.0;
            let missing_text_color = Color {
                a: (style.tooltip_text_color.a * 0.55).clamp(0.0, 1.0),
                ..style.tooltip_text_color
            };
            for (i, line) in prepared_lines.into_iter().enumerate() {
                let order_base = tooltip_order
                    .0
                    .saturating_add(1 + (i as u32).saturating_mul(3));
                let swatch_x = tip_x + pad.left.0;
                let text_x0 = swatch_x + swatch_space;

                let side = style.tooltip_marker_size.0.max(0.0);
                if side > 0.0
                    && reserve_swatch
                    && let Some(series) = line.source_series
                {
                    let color = state
                        .series_rank_by_id
                        .get(&series)
                        .copied()
                        .map(|rank| style.series_palette[rank % style.series_palette.len()])
                        .unwrap_or_else(|| {
                            style.series_palette[(series.0 as usize) % style.series_palette.len()]
                        });

                    let line_height = match &line.layout {
                        TooltipLineLayout::Single { metrics, .. } => metrics.size.height.0.max(1.0),
                        TooltipLineLayout::Columns {
                            left_metrics,
                            right_metrics,
                            ..
                        } => left_metrics
                            .size
                            .height
                            .0
                            .max(right_metrics.size.height.0)
                            .max(1.0),
                    };
                    let marker_y = y + (line_height - side) * 0.5;
                    painter.scene().push(fret_core::SceneOp::Quad {
                        order: DrawOrder(order_base),
                        rect: Rect::new(
                            Point::new(Px(swatch_x), Px(marker_y)),
                            Size::new(Px(side), Px(side)),
                        ),
                        background: fret_core::Paint::Solid(color),

                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT,

                        corner_radii: Corners::all(Px((side * 0.25).max(0.0))),
                    });
                }

                match line.layout {
                    TooltipLineLayout::Single { key, metrics, text } => {
                        let label_style_for_line = if line.kind == TooltipTextLineKind::AxisHeader {
                            header_text_style.clone()
                        } else {
                            text_style.clone()
                        };
                        let color = if line.is_missing {
                            missing_text_color
                        } else {
                            style.tooltip_text_color
                        };
                        let _ = painter.text(
                            key,
                            DrawOrder(order_base.saturating_add(1)),
                            Point::new(Px(text_x0), Px(y)),
                            text,
                            label_style_for_line,
                            color,
                            constraints,
                            paint_cx.raster_scale_factor,
                        );
                        y += metrics.size.height.0.max(1.0);
                    }
                    TooltipLineLayout::Columns {
                        left_key,
                        left_metrics,
                        left,
                        right_key,
                        right_metrics,
                        right,
                    } => {
                        let label_style_for_line = if line.kind == TooltipTextLineKind::AxisHeader {
                            header_text_style.clone()
                        } else {
                            text_style.clone()
                        };
                        let value_style_for_line = if line.value_emphasis
                            && line.kind != TooltipTextLineKind::AxisHeader
                        {
                            value_text_style.clone()
                        } else {
                            label_style_for_line.clone()
                        };
                        let line_height = left_metrics
                            .size
                            .height
                            .0
                            .max(right_metrics.size.height.0)
                            .max(1.0);
                        let value_x = text_x0
                            + max_left_w
                            + col_gap
                            + (max_right_w - right_metrics.size.width.0).max(0.0);

                        let _ = painter.text(
                            left_key,
                            DrawOrder(order_base.saturating_add(1)),
                            Point::new(Px(text_x0), Px(y)),
                            left,
                            label_style_for_line.clone(),
                            style.tooltip_text_color,
                            constraints,
                            paint_cx.raster_scale_factor,
                        );
                        let value_color = if line.is_missing {
                            missing_text_color
                        } else {
                            style.tooltip_text_color
                        };
                        let _ = painter.text(
                            right_key,
                            DrawOrder(order_base.saturating_add(2)),
                            Point::new(Px(value_x), Px(y)),
                            right,
                            value_style_for_line,
                            value_color,
                            constraints,
                            paint_cx.raster_scale_factor,
                        );

                        y += line_height;
                    }
                }
            }
        },
    );

    CanvasToolEntry {
        id: CanvasToolId::new(11),
        priority: 190,
        handlers: CanvasToolHandlers {
            on_paint: Some(on_paint),
            ..Default::default()
        },
    }
}
