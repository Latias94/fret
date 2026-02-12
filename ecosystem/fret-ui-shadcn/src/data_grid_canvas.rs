//! Canvas-backed data grid ("performance ceiling").
//!
//! This surface is designed for spreadsheet-scale density by keeping UI node count ~constant and
//! doing dense cell rendering via canvas ops. Rich editing UI is expected to live in overlay
//! layers (selection rectangles, editor popovers), not per-cell widgets.

use std::sync::Arc;

use fret_core::geometry::{Corners, Edges, Point, Rect, Size};
use fret_core::scene::SceneOp;
use fret_core::time::Instant;
use fret_core::{Color, DrawOrder, FontId, Px, TextOverflow, TextStyle, TextWrap};
use fret_runtime::Model;
use fret_ui::canvas::CanvasTextConstraints;
use fret_ui::element::{
    AnyElement, CanvasProps, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow,
    PositionStyle, ScrollAxis, ScrollProps, ScrollbarAxis, ScrollbarProps, ScrollbarStyle,
    SizeStyle, StackProps,
};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_headless::grid_viewport::{
    GridAxisItem, GridAxisMeasureMode, GridAxisMetrics, GridViewport2D, compute_grid_viewport_2d,
};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::theme_tokens;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius};

fn border_color(theme: &Theme) -> Color {
    theme.color_required("border")
}

fn background_color(theme: &Theme) -> Color {
    theme.color_required("background")
}

fn foreground_color(theme: &Theme) -> Color {
    theme.color_required("foreground")
}

fn font_size(theme: &Theme) -> Px {
    theme
        .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_PX)
        .unwrap_or_else(|| theme.metric_required("font.size"))
}

fn font_line_height(theme: &Theme) -> Px {
    theme
        .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT)
        .unwrap_or_else(|| theme.metric_required("font.line_height"))
}

fn text_style(theme: &Theme) -> TextStyle {
    TextStyle {
        font: FontId::default(),
        size: font_size(theme),
        line_height: Some(font_line_height(theme)),
        ..Default::default()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DataGridCanvasOutput {
    pub visible_rows: usize,
    pub visible_cols: usize,
    pub visible_cells: usize,
    pub ensure_axes_us: u32,
    pub apply_overrides_us: u32,
    pub compute_viewport_us: u32,
    pub build_visible_items_us: u32,
}

#[derive(Clone)]
pub struct DataGridCanvasAxis {
    pub keys: Arc<Vec<u64>>,
    pub revision: u64,
    pub mode: GridAxisMeasureMode,
    pub estimate: Px,
    pub gap: Px,
    pub padding_start: Px,
    pub min: Px,
    pub max: Option<Px>,
    pub reset_measurements_on_revision_change: bool,
    pub size_override: Option<Arc<dyn Fn(u64) -> Option<Px> + Send + Sync + 'static>>,
}

impl DataGridCanvasAxis {
    pub fn new(keys: Arc<Vec<u64>>, revision: u64, estimate: Px) -> Self {
        Self {
            keys,
            revision,
            mode: GridAxisMeasureMode::Measured,
            estimate,
            gap: Px(0.0),
            padding_start: Px(0.0),
            min: Px(0.0),
            max: None,
            reset_measurements_on_revision_change: false,
            size_override: None,
        }
    }

    pub fn fixed(mut self) -> Self {
        self.mode = GridAxisMeasureMode::Fixed;
        self
    }

    pub fn measured(mut self) -> Self {
        self.mode = GridAxisMeasureMode::Measured;
        self
    }

    pub fn gap(mut self, gap: Px) -> Self {
        self.gap = gap;
        self
    }

    pub fn padding_start(mut self, padding_start: Px) -> Self {
        self.padding_start = padding_start;
        self
    }

    pub fn min(mut self, min: Px) -> Self {
        self.min = min;
        self
    }

    pub fn max(mut self, max: Px) -> Self {
        self.max = Some(max);
        self
    }

    pub fn reset_measurements_on_revision_change(mut self, enabled: bool) -> Self {
        self.reset_measurements_on_revision_change = enabled;
        self
    }

    pub fn size_override(mut self, f: impl Fn(u64) -> Option<Px> + Send + Sync + 'static) -> Self {
        self.size_override = Some(Arc::new(f));
        self
    }

    fn clamp_size(&self, size: Px) -> Px {
        let mut size = Px(size.0.max(self.min.0));
        if let Some(max) = self.max {
            size = Px(size.0.min(max.0));
        }
        size
    }
}

#[derive(Clone)]
pub struct DataGridCanvas {
    pub rows: DataGridCanvasAxis,
    pub cols: DataGridCanvasAxis,
    pub overscan_rows: usize,
    pub overscan_cols: usize,
    pub chrome: ChromeRefinement,
    pub layout: LayoutRefinement,
    pub output: Option<Model<DataGridCanvasOutput>>,
}

impl DataGridCanvas {
    pub fn new(rows: DataGridCanvasAxis, cols: DataGridCanvasAxis) -> Self {
        Self {
            rows,
            cols,
            overscan_rows: 4,
            overscan_cols: 2,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            output: None,
        }
    }

    pub fn overscan_rows(mut self, overscan: usize) -> Self {
        self.overscan_rows = overscan;
        self
    }

    pub fn overscan_cols(mut self, overscan: usize) -> Self {
        self.overscan_cols = overscan;
        self
    }

    pub fn output_model(mut self, output: Model<DataGridCanvasOutput>) -> Self {
        self.output = Some(output);
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        cell_text_at: impl Fn(u64, u64) -> Arc<str> + Send + Sync + 'static,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let root_chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .border_color(ColorRef::Color(border_color(&theme)))
            .merge(self.chrome);
        let mut root_props = decl_style::container_props(&theme, root_chrome, self.layout.w_full());
        root_props.layout.overflow = Overflow::Clip;

        let rows = self.rows;
        let cols = self.cols;
        let overscan_rows = self.overscan_rows;
        let overscan_cols = self.overscan_cols;
        let output_model = self.output;
        let wants_output = output_model.is_some();

        let cell_text_at: Arc<dyn Fn(u64, u64) -> Arc<str> + Send + Sync + 'static> =
            Arc::new(cell_text_at);

        cx.container(root_props, move |cx| {
            let theme = Theme::global(&*cx.app).clone();

            let scrollbar_w = theme.metric_required("metric.scrollbar.width");
            let thumb = theme.color_required("scrollbar.thumb.background");
            let thumb_hover = theme.color_required("scrollbar.thumb.hover.background");

            let (scroll_handle, paint_data, total_w, total_h, output) =
                cx.with_state(DataGridCanvasState::default, |state| {
                    let t0 = wants_output.then(Instant::now);
                    state.ensure_axes(&rows, &cols);
                    let ensure_axes_us = t0
                        .map(|t| (t.elapsed().as_micros()).min(u32::MAX as u128) as u32)
                        .unwrap_or_default();

                    let viewport = state.scroll.viewport_size();
                    let offset = state.scroll.offset();

                    let mut apply_overrides_us = 0u32;
                    if rows.size_override.is_some() || cols.size_override.is_some() {
                        // Apply size overrides for currently visible rows/cols, then recompute the viewport.
                        let t = wants_output.then(Instant::now);
                        if let Some(vp) = compute_grid_viewport_2d(
                            &state.row_metrics,
                            &state.col_metrics,
                            offset.x,
                            offset.y,
                            viewport.width,
                            viewport.height,
                            overscan_rows,
                            overscan_cols,
                        ) {
                            state.apply_overrides(&rows, &cols, &vp);
                        }
                        apply_overrides_us = t
                            .map(|t| (t.elapsed().as_micros()).min(u32::MAX as u128) as u32)
                            .unwrap_or_default();
                    }

                    let viewport = state.scroll.viewport_size();
                    let offset = state.scroll.offset();
                    let t = wants_output.then(Instant::now);
                    let vp = compute_grid_viewport_2d(
                        &state.row_metrics,
                        &state.col_metrics,
                        offset.x,
                        offset.y,
                        viewport.width,
                        viewport.height,
                        overscan_rows,
                        overscan_cols,
                    );
                    let compute_viewport_us = t
                        .map(|t| (t.elapsed().as_micros()).min(u32::MAX as u128) as u32)
                        .unwrap_or_default();

                    let mut rows_visible = Vec::new();
                    let mut cols_visible = Vec::new();
                    let mut rows_clamped = Vec::new();
                    let mut build_visible_items_us = 0u32;
                    if let Some(vp) = vp {
                        let t = wants_output.then(Instant::now);
                        let row_start = vp
                            .row_range
                            .start_index
                            .saturating_sub(vp.row_range.overscan);
                        let row_end = (vp.row_range.end_index + vp.row_range.overscan)
                            .min(vp.row_range.count.saturating_sub(1));
                        let col_start = vp
                            .col_range
                            .start_index
                            .saturating_sub(vp.col_range.overscan);
                        let col_end = (vp.col_range.end_index + vp.col_range.overscan)
                            .min(vp.col_range.count.saturating_sub(1));

                        for i in row_start..=row_end {
                            if let Some(item) = state.row_metrics.axis_item(i) {
                                rows_visible.push(item);
                            }
                        }
                        for i in col_start..=col_end {
                            if let Some(item) = state.col_metrics.axis_item(i) {
                                cols_visible.push(item);
                            }
                        }
                        build_visible_items_us = t
                            .map(|t| (t.elapsed().as_micros()).min(u32::MAX as u128) as u32)
                            .unwrap_or_default();

                        rows_clamped = rows_visible
                            .iter()
                            .map(|row| {
                                let Some(max) = rows.max else {
                                    return false;
                                };
                                let Some(f) = &rows.size_override else {
                                    return false;
                                };
                                f(row.key).is_some_and(|size| size.0 > max.0)
                            })
                            .collect();
                    }

                    let viewport_rect = Rect::new(
                        Point::new(offset.x, offset.y),
                        Size::new(viewport.width, viewport.height),
                    );

                    let visible_rows = rows_visible.len();
                    let visible_cols = cols_visible.len();

                    let paint_data = DataGridCanvasPaintData {
                        viewport: viewport_rect,
                        rows: rows_visible,
                        cols: cols_visible,
                        rows_clamped,
                        bg: background_color(&theme),
                        grid: border_color(&theme),
                        text: foreground_color(&theme),
                        style: text_style(&theme),
                        raster_scale_factor: 1.0,
                        cell_pad_x: Px(8.0),
                        cell_pad_y: Px(6.0),
                        cell_text_at: Arc::clone(&cell_text_at),
                    };

                    let output = wants_output.then(|| DataGridCanvasOutput {
                        visible_rows,
                        visible_cols,
                        visible_cells: visible_rows.saturating_mul(visible_cols),
                        ensure_axes_us,
                        apply_overrides_us,
                        compute_viewport_us,
                        build_visible_items_us,
                    });

                    (
                        state.scroll.clone(),
                        Arc::new(paint_data),
                        state.col_metrics.total_size(),
                        state.row_metrics.total_size(),
                        output,
                    )
                });

            if let (Some(out_model), Some(output)) = (output_model, output) {
                let _ = cx.app.models_mut().update(&out_model, |v| {
                    if *v != output {
                        *v = output;
                    }
                });
            }

            let stack = cx.stack_props(
                StackProps {
                    layout: LayoutStyle {
                        overflow: Overflow::Clip,
                        ..Default::default()
                    },
                },
                move |cx| {
                    let mut scroll_layout = LayoutStyle::default();
                    scroll_layout.size.width = Length::Fill;
                    scroll_layout.size.height = Length::Fill;
                    scroll_layout.overflow = Overflow::Clip;

                    let scroll = cx.scroll(
                        ScrollProps {
                            layout: scroll_layout,
                            axis: ScrollAxis::Both,
                            scroll_handle: Some(scroll_handle.clone()),
                            ..Default::default()
                        },
                        move |cx| {
                            let mut content_layout = LayoutStyle::default();
                            content_layout.size.width = Length::Px(total_w);
                            content_layout.size.height = Length::Px(total_h);
                            content_layout.overflow = Overflow::Clip;

                            let paint_data = Arc::clone(&paint_data);

                            vec![cx.container(
                                ContainerProps {
                                    layout: content_layout,
                                    ..Default::default()
                                },
                                move |cx| {
                                    let mut canvas_layout = LayoutStyle::default();
                                    canvas_layout.size.width = Length::Fill;
                                    canvas_layout.size.height = Length::Fill;

                                    vec![cx.canvas(
                                        CanvasProps {
                                            layout: canvas_layout,
                                            ..Default::default()
                                        },
                                        move |p| paint_grid_canvas(p, &paint_data),
                                    )]
                                },
                            )]
                        },
                    );

                    let scroll_id = scroll.id;
                    let mut children = vec![scroll];

                    // Vertical scrollbar
                    let v_layout = LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: InsetStyle {
                            top: Some(Px(0.0)),
                            right: Some(Px(0.0)),
                            bottom: Some(scrollbar_w),
                            left: None,
                        },
                        size: SizeStyle {
                            width: Length::Px(scrollbar_w),
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    children.push(cx.scrollbar(ScrollbarProps {
                        layout: v_layout,
                        axis: ScrollbarAxis::Vertical,
                        scroll_target: Some(scroll_id),
                        scroll_handle: scroll_handle.clone(),
                        style: ScrollbarStyle {
                            thumb,
                            thumb_hover,
                            ..Default::default()
                        },
                    }));

                    // Horizontal scrollbar
                    let h_layout = LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: InsetStyle {
                            top: None,
                            right: Some(scrollbar_w),
                            bottom: Some(Px(0.0)),
                            left: Some(Px(0.0)),
                        },
                        size: SizeStyle {
                            height: Length::Px(scrollbar_w),
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    children.push(cx.scrollbar(ScrollbarProps {
                        layout: h_layout,
                        axis: ScrollbarAxis::Horizontal,
                        scroll_target: Some(scroll_id),
                        scroll_handle,
                        style: ScrollbarStyle {
                            thumb,
                            thumb_hover,
                            ..Default::default()
                        },
                    }));

                    children
                },
            );

            vec![stack]
        })
    }
}

#[derive(Debug, Default)]
struct DataGridCanvasState {
    scroll: ScrollHandle,
    row_metrics: GridAxisMetrics<u64>,
    col_metrics: GridAxisMetrics<u64>,
    last_row_revision: u64,
    last_col_revision: u64,
}

impl DataGridCanvasState {
    fn ensure_axes(&mut self, rows: &DataGridCanvasAxis, cols: &DataGridCanvasAxis) {
        if rows.reset_measurements_on_revision_change && self.last_row_revision != rows.revision {
            self.row_metrics.reset_measurements();
        }
        if cols.reset_measurements_on_revision_change && self.last_col_revision != cols.revision {
            self.col_metrics.reset_measurements();
        }
        self.last_row_revision = rows.revision;
        self.last_col_revision = cols.revision;

        self.row_metrics.ensure_with_mode(
            rows.mode,
            Arc::clone(&rows.keys),
            rows.revision,
            rows.estimate,
            rows.gap,
            rows.padding_start,
        );
        self.col_metrics.ensure_with_mode(
            cols.mode,
            Arc::clone(&cols.keys),
            cols.revision,
            cols.estimate,
            cols.gap,
            cols.padding_start,
        );
    }

    fn apply_overrides(
        &mut self,
        rows: &DataGridCanvasAxis,
        cols: &DataGridCanvasAxis,
        vp: &GridViewport2D,
    ) {
        if let Some(f) = &rows.size_override {
            let start = vp
                .row_range
                .start_index
                .saturating_sub(vp.row_range.overscan);
            let end = (vp.row_range.end_index + vp.row_range.overscan)
                .min(vp.row_range.count.saturating_sub(1));
            for idx in start..=end {
                let Some(key) = rows.keys.get(idx).copied() else {
                    continue;
                };
                let Some(size) = f(key) else {
                    continue;
                };
                self.row_metrics.measure(idx, rows.clamp_size(size));
            }
        }

        if let Some(f) = &cols.size_override {
            let start = vp
                .col_range
                .start_index
                .saturating_sub(vp.col_range.overscan);
            let end = (vp.col_range.end_index + vp.col_range.overscan)
                .min(vp.col_range.count.saturating_sub(1));
            for idx in start..=end {
                let Some(key) = cols.keys.get(idx).copied() else {
                    continue;
                };
                let Some(size) = f(key) else {
                    continue;
                };
                self.col_metrics.measure(idx, cols.clamp_size(size));
            }
        }
    }
}

#[derive(Clone)]
struct DataGridCanvasPaintData {
    viewport: Rect,
    rows: Vec<GridAxisItem<u64>>,
    cols: Vec<GridAxisItem<u64>>,
    rows_clamped: Vec<bool>,
    bg: Color,
    grid: Color,
    text: Color,
    style: TextStyle,
    raster_scale_factor: f32,
    cell_pad_x: Px,
    cell_pad_y: Px,
    cell_text_at: Arc<dyn Fn(u64, u64) -> Arc<str> + Send + Sync + 'static>,
}

fn paint_grid_canvas(p: &mut fret_ui::canvas::CanvasPainter<'_>, data: &DataGridCanvasPaintData) {
    let viewport = data.viewport;
    let x0 = viewport.origin.x;
    let y0 = viewport.origin.y;
    let w = viewport.size.width;
    let h = viewport.size.height;

    let raster_scale_factor = (p.scale_factor() * data.raster_scale_factor).max(1.0);

    p.with_clip_rect(viewport, |p| {
        p.scene().push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: viewport,
            background: fret_core::Paint::Solid(data.bg),
            border: Edges::default(),
            border_paint: fret_core::Paint::TRANSPARENT,
            corner_radii: Corners::default(),
        });

        // Grid lines (1px quads).
        let line_w = Px(1.0);
        for col in &data.cols {
            let x = col.end;
            let rect = Rect::new(Point::new(x, y0), Size::new(line_w, h));
            p.scene().push(SceneOp::Quad {
                order: DrawOrder(1),
                rect,
                background: fret_core::Paint::Solid(data.grid),
                border: Edges::default(),
                border_paint: fret_core::Paint::TRANSPARENT,
                corner_radii: Corners::default(),
            });
        }
        for row in &data.rows {
            let y = row.end;
            let rect = Rect::new(Point::new(x0, y), Size::new(w, line_w));
            p.scene().push(SceneOp::Quad {
                order: DrawOrder(1),
                rect,
                background: fret_core::Paint::Solid(data.grid),
                border: Edges::default(),
                border_paint: fret_core::Paint::TRANSPARENT,
                corner_radii: Corners::default(),
            });
        }

        // Text.
        let scope = p.key_scope(&"DataGridCanvas.cell_text");
        for row in &data.rows {
            for col in &data.cols {
                let max_width = Px((col.size.0 - 2.0 * data.cell_pad_x.0).max(0.0));
                let constraints = CanvasTextConstraints {
                    max_width: Some(max_width),
                    wrap: TextWrap::Word,
                    overflow: TextOverflow::Clip,
                };
                let origin = Point::new(
                    Px(col.start.0 + data.cell_pad_x.0),
                    Px(row.start.0 + data.cell_pad_y.0),
                );
                let cache_key: u64 = p.child_key(scope, &(row.key, col.key)).into();
                let text = (data.cell_text_at)(row.key, col.key);
                p.text(
                    cache_key,
                    DrawOrder(2),
                    origin,
                    text,
                    data.style.clone(),
                    data.text,
                    constraints,
                    raster_scale_factor,
                );
            }
        }

        // Clamp marker: when the caller-provided row height is clamped, draw a subtle “…” in the
        // last visible column to indicate truncation.
        if let Some(last_col) = data.cols.last() {
            let marker_color = Color {
                a: (data.text.a * 0.75).min(1.0),
                ..data.text
            };
            let marker_constraints = CanvasTextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            };
            for (row, clamped) in data.rows.iter().zip(data.rows_clamped.iter().copied()) {
                if !clamped {
                    continue;
                }

                let line_height = data.style.line_height.unwrap_or(data.style.size);
                let origin = Point::new(
                    Px(last_col.end.0 - data.cell_pad_x.0 - data.style.size.0),
                    Px(row.end.0 - data.cell_pad_y.0 - line_height.0),
                );
                let cache_key: u64 = p.child_key(scope, &(row.key, u64::MAX)).into();
                p.text(
                    cache_key,
                    DrawOrder(3),
                    origin,
                    Arc::from("…"),
                    data.style.clone(),
                    marker_color,
                    marker_constraints,
                    raster_scale_factor,
                );
            }
        }
    });
}
