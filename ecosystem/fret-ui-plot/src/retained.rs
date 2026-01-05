use fret_core::geometry::{Point, Px, Rect, Size};
use fret_core::scene::{Color, DrawOrder, SceneOp};
use fret_core::{
    Event, FontId, FontWeight, KeyCode, MouseButton, PathConstraints, PathId, PathStyle,
    PointerEvent, SemanticsRole, TextBlobId, TextConstraints, TextMetrics, TextOverflow, TextStyle,
    TextWrap, UiServices,
};
use fret_runtime::{Model, TextFontStackKey};
use fret_ui::UiHost;
use fret_ui::retained_bridge::{
    Invalidation, LayoutCx, PaintCx, SemanticsCx, UiTreeRetainedExt, Widget,
};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::cartesian::{DataPoint, DataRect, PlotTransform};
use crate::plot::axis::linear_ticks;
use crate::plot::decimate::{SamplePoint, decimate_polyline, decimate_samples};
use crate::plot::grid::GridLines;
use crate::plot::view::{
    clamp_view_to_data, clamp_zoom_factors, data_rect_from_plot_points, data_rect_key,
    expand_data_bounds, local_from_absolute, pan_view_by_px, sanitize_data_rect, zoom_view_at_px,
};
use crate::series::{Series, SeriesData, SeriesId};

#[derive(Debug, Clone)]
pub struct LineSeries {
    pub id: SeriesId,
    pub label: Arc<str>,
    pub data: Series,
    pub stroke_color: Option<Color>,
}

impl LineSeries {
    pub fn new(label: impl Into<Arc<str>>, data: Series) -> Self {
        let label = label.into();
        Self {
            id: SeriesId::from_label(&label),
            label,
            data,
            stroke_color: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.stroke_color = Some(color);
        self
    }

    pub fn id(mut self, id: SeriesId) -> Self {
        self.id = id;
        self
    }
}

#[derive(Debug, Clone)]
pub struct LinePlotModel {
    pub data_bounds: DataRect,
    pub series: Vec<LineSeries>,
}

impl LinePlotModel {
    pub fn from_series(series: Vec<LineSeries>) -> Self {
        let bounds =
            compute_data_bounds_from_series_data(&series, |s| &s.data).unwrap_or(DataRect {
                x_min: 0.0,
                x_max: 1.0,
                y_min: 0.0,
                y_max: 1.0,
            });

        Self {
            data_bounds: bounds,
            series,
        }
    }

    pub fn from_series_with_bounds(series: Vec<LineSeries>, data_bounds: DataRect) -> Self {
        Self {
            data_bounds: sanitize_data_rect(data_bounds),
            series,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScatterSeries {
    pub id: SeriesId,
    pub label: Arc<str>,
    pub data: Series,
    pub stroke_color: Option<Color>,
}

impl ScatterSeries {
    pub fn new(label: impl Into<Arc<str>>, data: Series) -> Self {
        let label = label.into();
        Self {
            id: SeriesId::from_label(&label),
            label,
            data,
            stroke_color: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.stroke_color = Some(color);
        self
    }

    pub fn id(mut self, id: SeriesId) -> Self {
        self.id = id;
        self
    }
}

#[derive(Debug, Clone)]
pub struct ScatterPlotModel {
    pub data_bounds: DataRect,
    pub series: Vec<ScatterSeries>,
}

impl ScatterPlotModel {
    pub fn from_series(series: Vec<ScatterSeries>) -> Self {
        let bounds =
            compute_data_bounds_from_series_data(&series, |s| &s.data).unwrap_or(DataRect {
                x_min: 0.0,
                x_max: 1.0,
                y_min: 0.0,
                y_max: 1.0,
            });

        Self {
            data_bounds: bounds,
            series,
        }
    }

    pub fn from_series_with_bounds(series: Vec<ScatterSeries>, data_bounds: DataRect) -> Self {
        Self {
            data_bounds: sanitize_data_rect(data_bounds),
            series,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SeriesStyle {
    stroke_color: Color,
}

const SERIES_PALETTE: [Color; 10] = [
    Color {
        r: 0.35,
        g: 0.65,
        b: 0.95,
        a: 1.0,
    },
    Color {
        r: 0.95,
        g: 0.45,
        b: 0.55,
        a: 1.0,
    },
    Color {
        r: 0.45,
        g: 0.85,
        b: 0.55,
        a: 1.0,
    },
    Color {
        r: 0.95,
        g: 0.75,
        b: 0.35,
        a: 1.0,
    },
    Color {
        r: 0.75,
        g: 0.55,
        b: 0.95,
        a: 1.0,
    },
    Color {
        r: 0.35,
        g: 0.85,
        b: 0.85,
        a: 1.0,
    },
    Color {
        r: 0.95,
        g: 0.35,
        b: 0.85,
        a: 1.0,
    },
    Color {
        r: 0.65,
        g: 0.65,
        b: 0.65,
        a: 1.0,
    },
    Color {
        r: 0.55,
        g: 0.75,
        b: 0.35,
        a: 1.0,
    },
    Color {
        r: 0.35,
        g: 0.55,
        b: 0.95,
        a: 1.0,
    },
];

fn resolve_series_color(
    series_index: usize,
    plot_style: LinePlotStyle,
    series_count: usize,
    override_color: Option<Color>,
) -> Color {
    if series_count <= 1 {
        return override_color.unwrap_or(plot_style.stroke_color);
    }
    override_color.unwrap_or(SERIES_PALETTE[series_index % SERIES_PALETTE.len()])
}

fn series_style(
    series: &LineSeries,
    series_index: usize,
    plot_style: LinePlotStyle,
    series_count: usize,
) -> SeriesStyle {
    SeriesStyle {
        stroke_color: resolve_series_color(
            series_index,
            plot_style,
            series_count,
            series.stroke_color,
        ),
    }
}

fn contains_point(rect: Rect, p: Point) -> bool {
    p.x.0 >= rect.origin.x.0
        && p.y.0 >= rect.origin.y.0
        && p.x.0 <= rect.origin.x.0 + rect.size.width.0
        && p.y.0 <= rect.origin.y.0 + rect.size.height.0
}

fn dim_color(color: Color, factor: f32) -> Color {
    let factor = factor.clamp(0.0, 1.0);
    Color {
        a: (color.a * factor).clamp(0.0, 1.0),
        ..color
    }
}

fn query_rect_from_plot_points_raw(
    view_bounds: DataRect,
    viewport: Size,
    a: Point,
    b: Point,
) -> Option<DataRect> {
    let viewport_w = viewport.width.0;
    let viewport_h = viewport.height.0;
    if !viewport_w.is_finite() || !viewport_h.is_finite() || viewport_w <= 0.0 || viewport_h <= 0.0
    {
        return None;
    }

    let x0 = a.x.0.min(b.x.0).clamp(0.0, viewport_w);
    let x1 = a.x.0.max(b.x.0).clamp(0.0, viewport_w);
    let y0 = a.y.0.min(b.y.0).clamp(0.0, viewport_h);
    let y1 = a.y.0.max(b.y.0).clamp(0.0, viewport_h);

    let transform = PlotTransform {
        viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), viewport),
        data: view_bounds,
    };

    let da = transform.px_to_data(Point::new(Px(x0), Px(y0)));
    let db = transform.px_to_data(Point::new(Px(x1), Px(y1)));

    if !da.x.is_finite() || !da.y.is_finite() || !db.x.is_finite() || !db.y.is_finite() {
        return None;
    }

    Some(DataRect {
        x_min: da.x.min(db.x),
        x_max: da.x.max(db.x),
        y_min: da.y.min(db.y),
        y_max: da.y.max(db.y),
    })
}

fn scatter_marker_commands(samples: &[SamplePoint], radius: Px) -> Vec<fret_core::PathCommand> {
    if samples.is_empty() {
        return Vec::new();
    }

    let r = radius.0.max(0.0);
    let mut out: Vec<fret_core::PathCommand> = Vec::with_capacity(samples.len().saturating_mul(4));

    for s in samples {
        let x = s.plot_px.x.0;
        let y = s.plot_px.y.0;
        if !x.is_finite() || !y.is_finite() {
            continue;
        }

        // Cross marker: horizontal then vertical.
        out.push(fret_core::PathCommand::MoveTo(Point::new(Px(x - r), Px(y))));
        out.push(fret_core::PathCommand::LineTo(Point::new(Px(x + r), Px(y))));
        out.push(fret_core::PathCommand::MoveTo(Point::new(Px(x), Px(y - r))));
        out.push(fret_core::PathCommand::LineTo(Point::new(Px(x), Px(y + r))));
    }

    out
}

#[derive(Debug, Clone, Copy)]
pub struct LinePlotStyle {
    pub background: Option<Color>,
    pub border: Option<Color>,
    pub border_width: Px,
    pub padding: Px,
    pub axis_gap: Px,
    pub axis_color: Option<Color>,
    pub grid_color: Option<Color>,
    pub label_color: Option<Color>,
    pub crosshair_color: Option<Color>,
    pub tooltip_background: Option<Color>,
    pub tooltip_border: Option<Color>,
    pub tooltip_text_color: Option<Color>,
    pub hover_threshold: Px,
    pub tick_count: usize,
    pub stroke_color: Color,
    pub stroke_width: Px,
    pub clamp_to_data_bounds: bool,
    /// Extra range around `data_bounds` used by clamping and auto-fit.
    ///
    /// This is expressed as a fraction of the data span (e.g. `0.03` means 3%).
    pub overscroll_fraction: f32,
    pub emphasize_hovered_series: bool,
    pub dimmed_series_alpha: f32,
}

impl Default for LinePlotStyle {
    fn default() -> Self {
        Self {
            background: None,
            border: None,
            border_width: Px(1.0),
            padding: Px(8.0),
            axis_gap: Px(18.0),
            axis_color: None,
            grid_color: None,
            label_color: None,
            crosshair_color: None,
            tooltip_background: None,
            tooltip_border: None,
            tooltip_text_color: None,
            hover_threshold: Px(10.0),
            tick_count: 5,
            stroke_color: Color {
                r: 0.35,
                g: 0.65,
                b: 0.95,
                a: 1.0,
            },
            stroke_width: Px(1.5),
            clamp_to_data_bounds: true,
            overscroll_fraction: 0.03,
            emphasize_hovered_series: true,
            dimmed_series_alpha: 0.35,
        }
    }
}

#[derive(Debug)]
struct CachedPath {
    id: Option<PathId>,
    series_id: SeriesId,
    model_revision: u64,
    scale_factor_bits: u32,
    viewport_w_bits: u32,
    viewport_h_bits: u32,
    stroke_width: Px,
    view_key: u64,
    samples: Vec<SamplePoint>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotHover {
    pub series_id: SeriesId,
    pub index: usize,
    pub data: DataPoint,
    pub plot_px: Point,
}

#[derive(Debug, Clone, Copy)]
struct PreparedText {
    blob: TextBlobId,
    metrics: TextMetrics,
    key: u64,
}

#[derive(Debug, Clone)]
struct LegendEntry {
    id: SeriesId,
    text: PreparedText,
}

#[derive(Debug, Clone, Copy)]
struct PlotLayout {
    plot: Rect,
    y_axis: Rect,
    x_axis: Rect,
}

impl PlotLayout {
    fn from_bounds(bounds: Rect, padding: Px, axis_gap: Px) -> Self {
        let pad = padding.0.max(0.0);
        let axis_gap = axis_gap.0.max(0.0);

        let content = Rect::new(
            Point::new(Px(bounds.origin.x.0 + pad), Px(bounds.origin.y.0 + pad)),
            Size::new(
                Px((bounds.size.width.0 - pad * 2.0).max(0.0)),
                Px((bounds.size.height.0 - pad * 2.0).max(0.0)),
            ),
        );

        let plot_w = (content.size.width.0 - axis_gap).max(0.0);
        let plot_h = (content.size.height.0 - axis_gap).max(0.0);

        let plot = Rect::new(
            Point::new(Px(content.origin.x.0 + axis_gap), content.origin.y),
            Size::new(Px(plot_w), Px(plot_h)),
        );

        let y_axis = Rect::new(content.origin, Size::new(Px(axis_gap), Px(plot_h)));

        let x_axis = Rect::new(
            Point::new(plot.origin.x, Px(plot.origin.y.0 + plot.size.height.0)),
            Size::new(Px(plot_w), Px(axis_gap)),
        );

        Self {
            plot,
            y_axis,
            x_axis,
        }
    }
}

#[derive(Debug)]
pub struct SeriesMeta {
    pub id: SeriesId,
    pub label: Arc<str>,
    pub stroke_color: Option<Color>,
}

pub trait PlotLayer {
    type Model: Clone + 'static;

    fn data_bounds(model: &Self::Model) -> DataRect;
    fn series_meta(model: &Self::Model) -> Vec<SeriesMeta>;
    fn series_label(model: &Self::Model, series_id: SeriesId) -> Option<String>;

    fn paint_paths<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        model: &Self::Model,
        args: PlotPaintArgs<'_>,
    ) -> Vec<(SeriesId, PathId, Color)>;

    fn hit_test(&mut self, model: &Self::Model, args: PlotHitTestArgs<'_>) -> Option<PlotHover>;

    fn cleanup_resources(&mut self, services: &mut dyn UiServices);
}

#[derive(Debug, Clone, Copy)]
pub struct PlotPaintArgs<'a> {
    pub model_revision: u64,
    pub plot: Rect,
    pub view_bounds: DataRect,
    pub style: LinePlotStyle,
    pub hidden: &'a HashSet<SeriesId>,
}

#[derive(Debug, Clone, Copy)]
pub struct PlotHitTestArgs<'a> {
    pub model_revision: u64,
    pub plot_size: Size,
    pub view_bounds: DataRect,
    pub scale_factor: f32,
    pub local: Point,
    pub style: LinePlotStyle,
    pub hover_threshold: Px,
    pub hidden: &'a HashSet<SeriesId>,
    pub pinned: Option<SeriesId>,
}

#[derive(Debug, Default)]
pub struct LinePlotLayer {
    cached_paths: Vec<CachedPath>,
}

pub type LinePlotCanvas = PlotCanvas<LinePlotLayer>;

#[derive(Debug, Default)]
pub struct ScatterPlotLayer {
    cached_paths: Vec<CachedPath>,
}

pub type ScatterPlotCanvas = PlotCanvas<ScatterPlotLayer>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotHoverOutput {
    pub series_id: SeriesId,
    pub data: DataPoint,
}

/// A caller-owned output snapshot for plot interaction state.
///
/// This is intended for building higher-level behaviors such as linked plots, inspectors, and
/// multi-pane coordination without requiring direct access to the plot internals.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotOutputSnapshot {
    pub view_bounds: DataRect,
    pub cursor: Option<DataPoint>,
    pub hover: Option<PlotHoverOutput>,
    pub query: Option<DataRect>,
}

/// Plot output state written by the plot widget.
///
/// Callers are expected to treat this as write-only from the widget side (i.e. do not mutate it
/// directly from application code). Use it as an observation point for interaction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotOutput {
    pub revision: u64,
    pub snapshot: PlotOutputSnapshot,
}

impl Default for PlotOutput {
    fn default() -> Self {
        Self {
            revision: 0,
            snapshot: PlotOutputSnapshot {
                view_bounds: DataRect {
                    x_min: 0.0,
                    x_max: 1.0,
                    y_min: 0.0,
                    y_max: 1.0,
                },
                cursor: None,
                hover: None,
                query: None,
            },
        }
    }
}

/// Persistent plot interaction state owned by the caller (optional).
///
/// This mirrors common plotting libraries (e.g. ImPlot / egui_plot) where plot view state and user
/// preferences (hidden series, pinned series) outlive a single render pass.
///
/// By default, `PlotCanvas` owns an internal `PlotState`. Callers can provide a `Model<PlotState>`
/// to store this state externally (so it can be persisted, shared, or controlled programmatically).
#[derive(Debug, Clone)]
pub struct PlotState {
    /// Current view bounds in data space when `view_is_auto == false`.
    pub view_bounds: Option<DataRect>,
    /// If true, the plot view is derived from `data_bounds` each frame (auto-fit).
    pub view_is_auto: bool,
    /// User-controlled series visibility.
    pub hidden_series: HashSet<SeriesId>,
    /// Optional pinned series ID for emphasis and tooltip pinning.
    pub pinned_series: Option<SeriesId>,
    /// Optional user query selection in data space.
    pub query: Option<DataRect>,
}

impl Default for PlotState {
    fn default() -> Self {
        Self {
            view_bounds: None,
            view_is_auto: true,
            hidden_series: HashSet::new(),
            pinned_series: None,
            query: None,
        }
    }
}

#[derive(Debug)]
pub struct PlotCanvas<L: PlotLayer + 'static> {
    model: Model<L::Model>,
    style: LinePlotStyle,
    layer: L,
    hover: Option<PlotHover>,
    plot_state: PlotState,
    plot_state_model: Option<Model<PlotState>>,
    plot_output: PlotOutput,
    plot_output_model: Option<Model<PlotOutput>>,
    legend_hover: Option<SeriesId>,
    cursor_px: Option<Point>,
    last_scale_factor: f32,
    pan_last_pos: Option<Point>,
    box_zoom_start: Option<Point>,
    box_zoom_current: Option<Point>,
    query_drag_start: Option<Point>,
    query_drag_current: Option<Point>,
    axis_label_key: Option<u64>,
    axis_labels_x: Vec<PreparedText>,
    axis_labels_y: Vec<PreparedText>,
    legend_key: Option<u64>,
    legend_entries: Vec<LegendEntry>,
    tooltip_text: Option<PreparedText>,
}

impl PlotCanvas<LinePlotLayer> {
    pub fn new(model: Model<LinePlotModel>) -> Self {
        Self::with_layer(model, LinePlotLayer::default())
    }
}

impl PlotCanvas<ScatterPlotLayer> {
    pub fn new(model: Model<ScatterPlotModel>) -> Self {
        Self::with_layer(model, ScatterPlotLayer::default())
    }
}

impl<L: PlotLayer + 'static> PlotCanvas<L> {
    pub fn with_layer(model: Model<L::Model>, layer: L) -> Self {
        Self {
            model,
            style: LinePlotStyle::default(),
            layer,
            hover: None,
            plot_state: PlotState::default(),
            plot_state_model: None,
            plot_output: PlotOutput::default(),
            plot_output_model: None,
            legend_hover: None,
            cursor_px: None,
            last_scale_factor: 1.0,
            pan_last_pos: None,
            box_zoom_start: None,
            box_zoom_current: None,
            query_drag_start: None,
            query_drag_current: None,
            axis_label_key: None,
            axis_labels_x: Vec::new(),
            axis_labels_y: Vec::new(),
            legend_key: None,
            legend_entries: Vec::new(),
            tooltip_text: None,
        }
    }

    pub fn style(mut self, style: LinePlotStyle) -> Self {
        self.style = style;
        self
    }

    pub fn state(mut self, state: Model<PlotState>) -> Self {
        self.plot_state_model = Some(state);
        self
    }

    pub fn output(mut self, output: Model<PlotOutput>) -> Self {
        self.plot_output_model = Some(output);
        self
    }

    pub fn create_node<H: UiHost>(ui: &mut fret_ui::UiTree<H>, canvas: Self) -> fret_core::NodeId {
        ui.create_node_retained(canvas)
    }

    fn read_plot_state<H: UiHost>(&self, app: &mut H) -> PlotState {
        if let Some(state) = &self.plot_state_model {
            state
                .read(app, |_app, s| s.clone())
                .unwrap_or_else(|_| self.plot_state.clone())
        } else {
            self.plot_state.clone()
        }
    }

    fn update_plot_state<H: UiHost>(
        &mut self,
        app: &mut H,
        f: impl FnOnce(&mut PlotState),
    ) -> bool {
        if let Some(state) = &self.plot_state_model {
            state.update(app, |s, _cx| f(s)).is_ok()
        } else {
            f(&mut self.plot_state);
            true
        }
    }

    fn publish_plot_output<H: UiHost>(&mut self, app: &mut H, snapshot: PlotOutputSnapshot) {
        if self.plot_output.snapshot == snapshot {
            return;
        }

        self.plot_output.revision = self.plot_output.revision.wrapping_add(1);
        self.plot_output.snapshot = snapshot;

        if let Some(model) = &self.plot_output_model {
            let next = self.plot_output;
            let _ = model.update(app, |s, _cx| {
                *s = next;
            });
        }
    }

    fn current_view_bounds<H: UiHost>(&self, app: &mut H, state: &PlotState) -> DataRect {
        if state.view_is_auto {
            let data_bounds = self.read_data_bounds(app);
            if self.style.clamp_to_data_bounds {
                expand_data_bounds(data_bounds, self.style.overscroll_fraction)
            } else {
                data_bounds
            }
        } else if let Some(view) = state.view_bounds {
            sanitize_data_rect(view)
        } else {
            self.read_data_bounds(app)
        }
    }

    fn rebuild_paths_if_needed<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        plot: Rect,
        view_bounds: DataRect,
        hidden: &HashSet<SeriesId>,
    ) -> Vec<(SeriesId, PathId, Color)> {
        let model_revision = self.model.revision(cx.app).unwrap_or(0);
        let Ok(model) = self.model.read(cx.app, |_app, m| m.clone()) else {
            return Vec::new();
        };

        self.layer.paint_paths(
            cx,
            &model,
            PlotPaintArgs {
                model_revision,
                plot,
                view_bounds,
                style: self.style,
                hidden,
            },
        )
    }

    fn clear_axis_label_cache(&mut self, services: &mut dyn UiServices) {
        for t in self.axis_labels_x.drain(..) {
            services.text().release(t.blob);
        }
        for t in self.axis_labels_y.drain(..) {
            services.text().release(t.blob);
        }
        self.axis_label_key = None;
    }

    fn clear_legend_cache(&mut self, services: &mut dyn UiServices) {
        for e in self.legend_entries.drain(..) {
            services.text().release(e.text.blob);
        }
        self.legend_key = None;
    }

    fn legend_layout(&self, layout: PlotLayout) -> Option<(Rect, Vec<Rect>)> {
        if self.legend_entries.len() <= 1 {
            return None;
        }
        if layout.plot.size.width.0 <= 0.0 || layout.plot.size.height.0 <= 0.0 {
            return None;
        }

        let margin = Px(8.0);
        let pad = Px(8.0);
        let gap = Px(8.0);
        let row_gap = Px(4.0);
        let swatch_w = Px(14.0);
        let swatch_h = Px(self.style.stroke_width.0.clamp(2.0, 6.0));

        let mut max_label_w = 0.0f32;
        let mut total_h = 0.0f32;
        for (i, entry) in self.legend_entries.iter().enumerate() {
            if i > 0 {
                total_h += row_gap.0;
            }
            max_label_w = max_label_w.max(entry.text.metrics.size.width.0);
            total_h += entry.text.metrics.size.height.0.max(swatch_h.0);
        }

        let legend_w = Px(pad.0 * 2.0 + swatch_w.0 + gap.0 + max_label_w);
        let legend_h = Px(pad.0 * 2.0 + total_h);

        let mut x = Px(layout.plot.origin.x.0 + layout.plot.size.width.0 - legend_w.0 - margin.0);
        let mut y = Px(layout.plot.origin.y.0 + margin.0);
        x = Px(x.0.max(layout.plot.origin.x.0));
        y = Px(y.0.max(layout.plot.origin.y.0));

        let rect = Rect::new(Point::new(x, y), Size::new(legend_w, legend_h));

        let mut rows: Vec<Rect> = Vec::with_capacity(self.legend_entries.len());
        let mut cursor_y = rect.origin.y.0 + pad.0;
        for (i, entry) in self.legend_entries.iter().enumerate() {
            let row_h = entry.text.metrics.size.height.0.max(swatch_h.0);
            rows.push(Rect::new(
                Point::new(rect.origin.x, Px(cursor_y)),
                Size::new(rect.size.width, Px(row_h)),
            ));
            cursor_y += row_h;
            if i + 1 < self.legend_entries.len() {
                cursor_y += row_gap.0;
            }
        }

        Some((rect, rows))
    }

    fn legend_swatch_column(rect: Rect) -> Rect {
        let pad = Px(8.0);
        let swatch_w = Px(14.0);
        Rect::new(
            Point::new(Px(rect.origin.x.0 + pad.0), rect.origin.y),
            Size::new(swatch_w, rect.size.height),
        )
    }

    fn hash_u64(mut state: u64, v: u64) -> u64 {
        state ^= v
            .wrapping_add(0x9e3779b97f4a7c15)
            .wrapping_add(state << 6)
            .wrapping_add(state >> 2);
        state
    }

    fn hash_f32_bits(state: u64, v: f32) -> u64 {
        Self::hash_u64(state, u64::from(v.to_bits()))
    }

    fn read_data_bounds<H: UiHost>(&self, app: &mut H) -> DataRect {
        let data_bounds = self
            .model
            .read(app, |_app, m| L::data_bounds(m))
            .unwrap_or(DataRect {
                x_min: 0.0,
                x_max: 1.0,
                y_min: 0.0,
                y_max: 1.0,
            });
        sanitize_data_rect(data_bounds)
    }

    fn text_style_key(style: &TextStyle) -> u64 {
        let mut state = 0u64;
        state = Self::hash_u64(state, hash_value(&style.font));
        state = Self::hash_u64(state, u64::from(style.weight.0));
        state = Self::hash_f32_bits(state, style.size.0);
        state = Self::hash_u64(
            state,
            u64::from(style.line_height.map(|v| v.0.to_bits()).unwrap_or(0)),
        );
        state = Self::hash_u64(
            state,
            u64::from(style.letter_spacing_em.map(|v| v.to_bits()).unwrap_or(0)),
        );
        state
    }

    fn prepare_text(
        &mut self,
        services: &mut dyn UiServices,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> PreparedText {
        let mut state = 0u64;
        for b in text.as_bytes() {
            state = Self::hash_u64(state, u64::from(*b));
        }
        state = Self::hash_u64(state, Self::text_style_key(style));
        state = Self::hash_u64(state, u64::from(constraints.scale_factor.to_bits()));
        state = Self::hash_u64(
            state,
            u64::from(constraints.max_width.map(|v| v.0.to_bits()).unwrap_or(0)),
        );
        state = Self::hash_u64(state, hash_value(&constraints.wrap));
        state = Self::hash_u64(state, hash_value(&constraints.overflow));

        let (blob, metrics) = services.text().prepare(text, style, constraints);
        PreparedText {
            blob,
            metrics,
            key: state,
        }
    }

    fn rebuild_axis_labels_if_needed<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        layout: PlotLayout,
        data_bounds: DataRect,
        theme_revision: u64,
        font_stack_key: u64,
    ) {
        let scale_bits = cx.scale_factor.to_bits();

        let mut key = 0u64;
        key = Self::hash_u64(key, u64::from(scale_bits));
        key = Self::hash_u64(key, theme_revision);
        key = Self::hash_u64(key, font_stack_key);
        key = Self::hash_f32_bits(key, layout.plot.size.width.0);
        key = Self::hash_f32_bits(key, layout.plot.size.height.0);
        key = Self::hash_f32_bits(key, data_bounds.x_min);
        key = Self::hash_f32_bits(key, data_bounds.x_max);
        key = Self::hash_f32_bits(key, data_bounds.y_min);
        key = Self::hash_f32_bits(key, data_bounds.y_max);
        key = Self::hash_u64(key, u64::from(self.style.axis_gap.0.to_bits()));
        key = Self::hash_u64(key, u64::from(self.style.tick_count as u32));

        if self.axis_label_key == Some(key) {
            return;
        }

        self.clear_axis_label_cache(cx.services);

        let font_size = cx
            .theme()
            .metric_by_key("font.size")
            .unwrap_or(cx.theme().metrics.font_size);
        let style = TextStyle {
            font: FontId::default(),
            size: Px((font_size.0 * 0.90).max(10.0)),
            weight: FontWeight::NORMAL,
            line_height: None,
            letter_spacing_em: None,
        };

        let x_ticks = linear_ticks(data_bounds.x_min, data_bounds.x_max, self.style.tick_count);
        let y_ticks = linear_ticks(data_bounds.y_min, data_bounds.y_max, self.style.tick_count);

        let fmt = |v: f32| -> String {
            let abs = v.abs();
            if abs < 1.0 {
                format!("{v:.3}")
            } else if abs < 10.0 {
                format!("{v:.2}")
            } else {
                format!("{v:.1}")
            }
        };

        for v in x_ticks {
            let constraints = TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: cx.scale_factor,
            };
            let prepared = self.prepare_text(cx.services, &fmt(v), &style, constraints);
            self.axis_labels_x.push(prepared);
        }

        for v in y_ticks {
            let constraints = TextConstraints {
                max_width: Some(layout.y_axis.size.width),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: cx.scale_factor,
            };
            let prepared = self.prepare_text(cx.services, &fmt(v), &style, constraints);
            self.axis_labels_y.push(prepared);
        }

        self.axis_label_key = Some(key);
    }

    fn rebuild_legend_if_needed<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        theme_revision: u64,
        font_stack_key: u64,
    ) {
        let series: Vec<SeriesMeta> = self
            .model
            .read(cx.app, |_app, m| L::series_meta(m))
            .unwrap_or_default();
        if let Some(hovered) = self.legend_hover
            && series.iter().all(|s| s.id != hovered)
        {
            self.legend_hover = None;
        }

        if series.len() <= 1 {
            if self.legend_key.is_some() {
                self.clear_legend_cache(cx.services);
            }
            return;
        }

        let font_size = cx
            .theme()
            .metric_by_key("font.size")
            .unwrap_or(cx.theme().metrics.font_size);
        let style = TextStyle {
            font: FontId::default(),
            size: Px((font_size.0 * 0.85).max(10.0)),
            weight: FontWeight::NORMAL,
            line_height: None,
            letter_spacing_em: None,
        };
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };

        let mut key = 0u64;
        key = Self::hash_u64(key, theme_revision);
        key = Self::hash_u64(key, font_stack_key);
        key = Self::hash_u64(key, u64::from(cx.scale_factor.to_bits()));
        key = Self::hash_u64(key, u64::from(series.len() as u32));
        key = Self::hash_u64(key, Self::text_style_key(&style));
        for s in &series {
            key = Self::hash_u64(key, s.id.0);
            for b in s.label.as_bytes() {
                key = Self::hash_u64(key, u64::from(*b));
            }
        }

        if self.legend_key == Some(key) {
            return;
        }

        self.clear_legend_cache(cx.services);

        self.legend_entries = Vec::with_capacity(series.len());
        for s in series {
            let text = s.label.to_string();
            let prepared = self.prepare_text(cx.services, &text, &style, constraints);
            self.legend_entries.push(LegendEntry {
                id: s.id,
                text: prepared,
            });
        }

        self.legend_key = Some(key);
    }
}

impl<H: UiHost, L: PlotLayer + 'static> Widget<H> for PlotCanvas<L> {
    fn event(&mut self, cx: &mut fret_ui::retained_bridge::EventCx<'_, H>, event: &Event) {
        match event {
            Event::KeyDown { key, modifiers, .. } => {
                let plain = !modifiers.shift
                    && !modifiers.ctrl
                    && !modifiers.alt
                    && !modifiers.alt_gr
                    && !modifiers.meta;
                if plain && *key == KeyCode::KeyR {
                    let _ = self.update_plot_state(cx.app, |s| {
                        s.view_is_auto = true;
                        s.view_bounds = None;
                        s.hidden_series.clear();
                        s.pinned_series = None;
                        s.query = None;
                    });
                    self.hover = None;
                    self.cursor_px = None;
                    self.pan_last_pos = None;
                    self.box_zoom_start = None;
                    self.box_zoom_current = None;
                    self.query_drag_start = None;
                    self.query_drag_current = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                } else if plain && *key == KeyCode::KeyA {
                    let _ = self.update_plot_state(cx.app, |s| {
                        s.hidden_series.clear();
                        s.pinned_series = None;
                    });
                    self.hover = None;
                    self.cursor_px = None;
                    self.legend_hover = None;
                    self.pan_last_pos = None;
                    self.box_zoom_start = None;
                    self.box_zoom_current = None;
                    self.query_drag_start = None;
                    self.query_drag_current = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                } else if plain && *key == KeyCode::KeyQ {
                    let query = self.read_plot_state(cx.app).query;
                    if query.is_some() {
                        let _ = self.update_plot_state(cx.app, |s| {
                            s.query = None;
                        });
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                } else if *key == KeyCode::Escape {
                    let state = self.read_plot_state(cx.app);
                    let has_active_drag = self.box_zoom_start.is_some()
                        || self.pan_last_pos.is_some()
                        || self.query_drag_start.is_some();

                    if has_active_drag {
                        self.pan_last_pos = None;
                        self.box_zoom_start = None;
                        self.box_zoom_current = None;
                        self.query_drag_start = None;
                        self.query_drag_current = None;
                        self.hover = None;
                        if cx.captured == Some(cx.node) {
                            cx.release_pointer_capture();
                        }
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    } else if state.pinned_series.is_some() {
                        let _ = self.update_plot_state(cx.app, |s| {
                            s.pinned_series = None;
                        });
                        self.legend_hover = None;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    } else if state.query.is_some() {
                        let _ = self.update_plot_state(cx.app, |s| {
                            s.query = None;
                        });
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                }
            }
            Event::Pointer(PointerEvent::Down {
                position,
                button,
                modifiers,
            }) => {
                let layout =
                    PlotLayout::from_bounds(cx.bounds, self.style.padding, self.style.axis_gap);
                if layout.plot.size.width.0 <= 0.0 || layout.plot.size.height.0 <= 0.0 {
                    return;
                }

                if *button == MouseButton::Left
                    && let Some((_legend, rows)) = self.legend_layout(layout)
                    && let Some(series_index) = rows
                        .iter()
                        .enumerate()
                        .find(|(_i, r)| contains_point(**r, *position))
                        .map(|(i, _r)| i)
                {
                    let row = rows[series_index];
                    let swatch = Self::legend_swatch_column(row);
                    let Some(entry) = self.legend_entries.get(series_index).cloned() else {
                        return;
                    };
                    let id = entry.id;
                    let state = self.read_plot_state(cx.app);
                    let mut next_hidden = state.hidden_series;
                    let mut next_pinned = state.pinned_series;

                    // Legend interaction policy:
                    // - Shift+Click: solo the series (or restore all if already solo)
                    // - Click swatch column: toggle visibility
                    // - Click label area: pin/unpin tooltip + emphasis to this series
                    if modifiers.shift {
                        let ids: Vec<SeriesId> = self.legend_entries.iter().map(|e| e.id).collect();
                        let visible_count =
                            ids.iter().filter(|sid| !next_hidden.contains(sid)).count();
                        let is_solo = visible_count == 1 && !next_hidden.contains(&id);
                        if is_solo {
                            next_hidden.clear();
                        } else {
                            next_hidden = ids.into_iter().filter(|sid| *sid != id).collect();
                        }
                        next_hidden.remove(&id);
                    } else if contains_point(swatch, *position) {
                        let total = self.legend_entries.len();
                        let hidden_count = self
                            .legend_entries
                            .iter()
                            .filter(|e| next_hidden.contains(&e.id))
                            .count();
                        let visible_count = total.saturating_sub(hidden_count);

                        let is_hidden = next_hidden.contains(&id);
                        if !is_hidden && visible_count <= 1 {
                            // Never hide the last visible series.
                        } else if is_hidden {
                            next_hidden.remove(&id);
                        } else {
                            next_hidden.insert(id);
                        }
                    } else if next_pinned == Some(id) {
                        next_pinned = None;
                    } else {
                        next_pinned = Some(id);
                        next_hidden.remove(&id);
                    }

                    let _ = self.update_plot_state(cx.app, |s| {
                        s.hidden_series = next_hidden;
                        s.pinned_series = next_pinned;
                    });

                    self.hover = None;
                    self.cursor_px = None;
                    self.legend_hover = Some(id);
                    self.pan_last_pos = None;
                    self.box_zoom_start = None;
                    self.box_zoom_current = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                let inside = contains_point(layout.plot, *position);

                if inside && *button == MouseButton::Left {
                    self.cursor_px = Some(local_from_absolute(layout.plot.origin, *position));
                    self.hover = None;
                    if modifiers.alt {
                        let local = local_from_absolute(layout.plot.origin, *position);
                        self.query_drag_start = Some(local);
                        self.query_drag_current = Some(local);
                        self.pan_last_pos = None;
                        self.box_zoom_start = None;
                        self.box_zoom_current = None;
                    } else if modifiers.shift {
                        let local = local_from_absolute(layout.plot.origin, *position);
                        self.box_zoom_start = Some(local);
                        self.box_zoom_current = Some(local);
                        self.pan_last_pos = None;
                        self.query_drag_start = None;
                        self.query_drag_current = None;
                    } else {
                        let _ = self.update_plot_state(cx.app, |s| {
                            s.view_is_auto = false;
                        });
                        self.pan_last_pos = Some(*position);
                        self.box_zoom_start = None;
                        self.box_zoom_current = None;
                        self.query_drag_start = None;
                        self.query_drag_current = None;
                    }
                    cx.request_focus(cx.node);
                    cx.capture_pointer(cx.node);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
            }
            Event::Pointer(PointerEvent::Up { button, .. }) => {
                if *button == MouseButton::Left {
                    if self.query_drag_start.is_some() {
                        if cx.captured == Some(cx.node) {
                            cx.release_pointer_capture();
                        }

                        let layout = PlotLayout::from_bounds(
                            cx.bounds,
                            self.style.padding,
                            self.style.axis_gap,
                        );
                        if layout.plot.size.width.0 > 0.0 && layout.plot.size.height.0 > 0.0 {
                            let start = self
                                .query_drag_start
                                .unwrap_or(Point::new(Px(0.0), Px(0.0)));
                            let end = self
                                .query_drag_current
                                .unwrap_or(Point::new(Px(0.0), Px(0.0)));

                            let w = (start.x.0 - end.x.0).abs();
                            let h = (start.y.0 - end.y.0).abs();

                            if w >= 4.0 && h >= 4.0 {
                                let state = self.read_plot_state(cx.app);
                                let view_bounds = self.current_view_bounds(cx.app, &state);
                                if let Some(next) = query_rect_from_plot_points_raw(
                                    view_bounds,
                                    layout.plot.size,
                                    start,
                                    end,
                                ) {
                                    let _ = self.update_plot_state(cx.app, |s| {
                                        s.query = Some(next);
                                    });
                                }
                            }
                        }

                        self.query_drag_start = None;
                        self.query_drag_current = None;
                        self.pan_last_pos = None;
                        self.hover = None;

                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    } else if self.box_zoom_start.is_some() {
                        if cx.captured == Some(cx.node) {
                            cx.release_pointer_capture();
                        }

                        let layout = PlotLayout::from_bounds(
                            cx.bounds,
                            self.style.padding,
                            self.style.axis_gap,
                        );
                        if layout.plot.size.width.0 > 0.0 && layout.plot.size.height.0 > 0.0 {
                            let start = self.box_zoom_start.unwrap_or(Point::new(Px(0.0), Px(0.0)));
                            let end = self
                                .box_zoom_current
                                .unwrap_or(Point::new(Px(0.0), Px(0.0)));

                            let w = (start.x.0 - end.x.0).abs();
                            let h = (start.y.0 - end.y.0).abs();

                            if w >= 4.0 && h >= 4.0 {
                                let state = self.read_plot_state(cx.app);
                                let view_bounds = self.current_view_bounds(cx.app, &state);
                                if let Some(mut next) = data_rect_from_plot_points(
                                    view_bounds,
                                    layout.plot.size,
                                    start,
                                    end,
                                ) {
                                    let data_bounds = self.read_data_bounds(cx.app);
                                    if self.style.clamp_to_data_bounds {
                                        next = clamp_view_to_data(
                                            next,
                                            data_bounds,
                                            self.style.overscroll_fraction,
                                        );
                                    }
                                    let _ = self.update_plot_state(cx.app, |s| {
                                        s.view_is_auto = false;
                                        s.view_bounds = Some(next);
                                    });
                                }
                            }
                        }

                        self.box_zoom_start = None;
                        self.box_zoom_current = None;
                        self.pan_last_pos = None;
                        self.hover = None;

                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    } else if self.pan_last_pos.take().is_some() {
                        if cx.captured == Some(cx.node) {
                            cx.release_pointer_capture();
                        }
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                }
            }
            Event::Pointer(PointerEvent::Wheel {
                position,
                delta,
                modifiers,
            }) => {
                let layout =
                    PlotLayout::from_bounds(cx.bounds, self.style.padding, self.style.axis_gap);
                if layout.plot.size.width.0 <= 0.0 || layout.plot.size.height.0 <= 0.0 {
                    return;
                }

                let inside = contains_point(layout.plot, *position);
                if !inside {
                    return;
                }
                if self.box_zoom_start.is_some() || self.query_drag_start.is_some() {
                    return;
                }

                let delta_y = delta.y.0;
                if !delta_y.is_finite() {
                    return;
                }

                let zoom = clamp_zoom_factors(2.0_f32.powf(delta_y * 0.0025));
                let (zoom_x, zoom_y) = if modifiers.shift {
                    (zoom, 1.0)
                } else if modifiers.ctrl {
                    (1.0, zoom)
                } else {
                    (zoom, zoom)
                };

                let state = self.read_plot_state(cx.app);
                let view_bounds = self.current_view_bounds(cx.app, &state);
                let local = local_from_absolute(layout.plot.origin, *position);
                let Some(next) =
                    zoom_view_at_px(view_bounds, layout.plot.size, local, zoom_x, zoom_y)
                else {
                    return;
                };
                let data_bounds = self.read_data_bounds(cx.app);
                let next = if self.style.clamp_to_data_bounds {
                    clamp_view_to_data(next, data_bounds, self.style.overscroll_fraction)
                } else {
                    next
                };

                let _ = self.update_plot_state(cx.app, |s| {
                    s.view_is_auto = false;
                    s.view_bounds = Some(next);
                });
                cx.request_focus(cx.node);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            Event::Pointer(PointerEvent::Move { position, .. }) => {
                let layout =
                    PlotLayout::from_bounds(cx.bounds, self.style.padding, self.style.axis_gap);
                if layout.plot.size.width.0 <= 0.0 || layout.plot.size.height.0 <= 0.0 {
                    return;
                }

                if self.box_zoom_start.is_none() && self.pan_last_pos.is_none() {
                    if let Some((legend, rows)) = self.legend_layout(layout)
                        && contains_point(legend, *position)
                    {
                        let cursor_changed = self.cursor_px.take().is_some();

                        let series_index = rows
                            .iter()
                            .enumerate()
                            .find(|(_i, r)| contains_point(**r, *position))
                            .map(|(i, _r)| i);

                        let hovered_id =
                            series_index.and_then(|i| self.legend_entries.get(i).map(|e| e.id));

                        if self.legend_hover != hovered_id {
                            self.legend_hover = hovered_id;
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }

                        if self.hover.is_some() {
                            self.hover = None;
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }

                        if cursor_changed {
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }

                        cx.stop_propagation();
                        return;
                    }

                    if self.legend_hover.take().is_some() {
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                }

                if self.query_drag_start.is_some() {
                    let local = local_from_absolute(layout.plot.origin, *position);
                    self.cursor_px = Some(local);
                    self.query_drag_current = Some(local);
                    self.hover = None;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if self.box_zoom_start.is_some() {
                    let local = local_from_absolute(layout.plot.origin, *position);
                    self.cursor_px = Some(local);
                    self.box_zoom_current = Some(local);
                    self.hover = None;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if let Some(last) = self.pan_last_pos {
                    self.cursor_px = None;
                    let dx_px = position.x.0 - last.x.0;
                    let dy_px = position.y.0 - last.y.0;

                    let state = self.read_plot_state(cx.app);
                    let view_bounds = self.current_view_bounds(cx.app, &state);
                    let Some(next) = pan_view_by_px(view_bounds, layout.plot.size, dx_px, dy_px)
                    else {
                        return;
                    };
                    let data_bounds = self.read_data_bounds(cx.app);
                    let next = if self.style.clamp_to_data_bounds {
                        clamp_view_to_data(next, data_bounds, self.style.overscroll_fraction)
                    } else {
                        next
                    };

                    let _ = self.update_plot_state(cx.app, |s| {
                        s.view_is_auto = false;
                        s.view_bounds = Some(next);
                    });
                    self.pan_last_pos = Some(*position);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                let inside = contains_point(layout.plot, *position);

                let prev_cursor = self.cursor_px;
                self.cursor_px = inside.then(|| local_from_absolute(layout.plot.origin, *position));
                if prev_cursor != self.cursor_px {
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }

                let next_hover = if inside {
                    let model_revision = self.model.revision(cx.app).unwrap_or(0);
                    let scale_factor = self.last_scale_factor;

                    let state = self.read_plot_state(cx.app);
                    let view_bounds = self.current_view_bounds(cx.app, &state);
                    let hidden = &state.hidden_series;
                    let pinned = state.pinned_series;

                    let local = local_from_absolute(layout.plot.origin, *position);

                    self.model
                        .read(cx.app, |_app, m| m.clone())
                        .ok()
                        .and_then(|model| {
                            self.layer.hit_test(
                                &model,
                                PlotHitTestArgs {
                                    model_revision,
                                    plot_size: layout.plot.size,
                                    view_bounds,
                                    scale_factor,
                                    local,
                                    style: self.style,
                                    hover_threshold: self.style.hover_threshold,
                                    hidden,
                                    pinned,
                                },
                            )
                        })
                } else {
                    None
                };

                if self.hover != next_hover {
                    self.hover = next_hover;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(&self.model, Invalidation::Paint);
        if let Some(state) = &self.plot_state_model {
            cx.observe_model(state, Invalidation::Paint);
        }
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(&self.model, Invalidation::Paint);
        if let Some(state) = &self.plot_state_model {
            cx.observe_model(state, Invalidation::Paint);
        }
        cx.observe_global::<TextFontStackKey>(Invalidation::Paint);
        self.last_scale_factor = cx.scale_factor;

        let theme = cx.theme().snapshot();
        let font_stack_key = cx
            .app
            .global::<TextFontStackKey>()
            .map(|k| k.0)
            .unwrap_or(0);
        let background = self
            .style
            .background
            .unwrap_or(theme.colors.panel_background);
        let border = self.style.border.unwrap_or(theme.colors.panel_border);

        let axis_color = self.style.axis_color.unwrap_or(theme.colors.panel_border);
        let grid_color = self.style.grid_color.unwrap_or(Color {
            a: 0.35,
            ..theme.colors.panel_border
        });
        let label_color = self.style.label_color.unwrap_or(theme.colors.text_muted);
        let crosshair_color = self.style.crosshair_color.unwrap_or(Color {
            a: 0.65,
            ..theme.colors.accent
        });
        let selection_border = crosshair_color;
        let selection_fill = Color {
            a: (crosshair_color.a * 0.18).clamp(0.06, 0.22),
            ..crosshair_color
        };
        let tooltip_background = self
            .style
            .tooltip_background
            .unwrap_or(theme.colors.menu_background);
        let tooltip_border = self
            .style
            .tooltip_border
            .unwrap_or(theme.colors.menu_border);
        let tooltip_text_color = self
            .style
            .tooltip_text_color
            .unwrap_or(theme.colors.text_primary);

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background,
            border: fret_core::Edges::all(self.style.border_width),
            border_color: border,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });

        let layout = PlotLayout::from_bounds(cx.bounds, self.style.padding, self.style.axis_gap);
        let state = self.read_plot_state(cx.app);
        let view_bounds = self.current_view_bounds(cx.app, &state);

        let cursor_data = self.cursor_px.and_then(|cursor_px| {
            if layout.plot.size.width.0 <= 0.0 || layout.plot.size.height.0 <= 0.0 {
                return None;
            }
            let transform = PlotTransform {
                viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size),
                data: view_bounds,
            };
            let data = transform.px_to_data(cursor_px);
            (data.x.is_finite() && data.y.is_finite()).then_some(data)
        });

        self.publish_plot_output(
            cx.app,
            PlotOutputSnapshot {
                view_bounds,
                cursor: cursor_data,
                hover: self.hover.map(|h| PlotHoverOutput {
                    series_id: h.series_id,
                    data: h.data,
                }),
                query: state.query,
            },
        );

        self.rebuild_axis_labels_if_needed(cx, layout, view_bounds, theme.revision, font_stack_key);
        self.rebuild_legend_if_needed(cx, theme.revision, font_stack_key);

        // Grid + series + hover are clipped to the plot area.
        cx.scene.push(SceneOp::PushClipRect { rect: layout.plot });

        if layout.plot.size.width.0 > 0.0 && layout.plot.size.height.0 > 0.0 {
            for (a, _) in GridLines::default().x_lines(layout.plot) {
                let x = Px(a.x.0.round());
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: Rect::new(
                        Point::new(x, layout.plot.origin.y),
                        Size::new(Px(1.0), layout.plot.size.height),
                    ),
                    background: grid_color,
                    border: fret_core::Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });
            }
            for (a, _) in GridLines::default().y_lines(layout.plot) {
                let y = Px(a.y.0.round());
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: Rect::new(
                        Point::new(layout.plot.origin.x, y),
                        Size::new(layout.plot.size.width, Px(1.0)),
                    ),
                    background: grid_color,
                    border: fret_core::Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });
            }

            let emphasized = self.style.emphasize_hovered_series;
            let dim_alpha = self.style.dimmed_series_alpha;
            let series_meta: Vec<SeriesMeta> = self
                .model
                .read(cx.app, |_app, m| L::series_meta(m))
                .unwrap_or_default();
            let pinned = state
                .pinned_series
                .filter(|id| series_meta.iter().any(|s| s.id == *id));
            let hidden = &state.hidden_series;

            let emphasized_series = pinned
                .or(self.hover.map(|h| h.series_id))
                .or(self.legend_hover);

            for (series_id, path, color) in
                self.rebuild_paths_if_needed(cx, layout.plot, view_bounds, hidden)
            {
                let color = if emphasized {
                    if let Some(emphasized) = emphasized_series
                        && emphasized != series_id
                    {
                        dim_color(color, dim_alpha)
                    } else {
                        color
                    }
                } else {
                    color
                };
                cx.scene.push(SceneOp::Path {
                    order: DrawOrder(2),
                    origin: layout.plot.origin,
                    path,
                    color,
                });
            }

            if let Some(cursor) = self.cursor_px {
                let x = Px((layout.plot.origin.x.0 + cursor.x.0).round());
                let y = Px((layout.plot.origin.y.0 + cursor.y.0).round());
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(3),
                    rect: Rect::new(
                        Point::new(x, layout.plot.origin.y),
                        Size::new(Px(1.0), layout.plot.size.height),
                    ),
                    background: crosshair_color,
                    border: fret_core::Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });

                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(3),
                    rect: Rect::new(
                        Point::new(layout.plot.origin.x, y),
                        Size::new(layout.plot.size.width, Px(1.0)),
                    ),
                    background: crosshair_color,
                    border: fret_core::Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });
            }

            if let Some(hover) = self.hover {
                let hx = Px((layout.plot.origin.x.0 + hover.plot_px.x.0).round());
                let hy = Px((layout.plot.origin.y.0 + hover.plot_px.y.0).round());

                let dot_size = Px(6.0);
                let dot_origin =
                    Point::new(Px(hx.0 - dot_size.0 * 0.5), Px(hy.0 - dot_size.0 * 0.5));
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(4),
                    rect: Rect::new(dot_origin, Size::new(dot_size, dot_size)),
                    background: crosshair_color,
                    border: fret_core::Edges::all(Px(1.0)),
                    border_color: tooltip_border,
                    corner_radii: fret_core::Corners::all(Px(dot_size.0 * 0.5)),
                });
            }

            if let Some(query) = state.query {
                let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size);
                let transform = PlotTransform {
                    viewport: local_viewport,
                    data: view_bounds,
                };

                let a = transform.data_to_px(DataPoint {
                    x: query.x_min,
                    y: query.y_min,
                });
                let b = transform.data_to_px(DataPoint {
                    x: query.x_max,
                    y: query.y_max,
                });

                let x0 = a.x.0.min(b.x.0).clamp(0.0, layout.plot.size.width.0);
                let x1 = a.x.0.max(b.x.0).clamp(0.0, layout.plot.size.width.0);
                let y0 = a.y.0.min(b.y.0).clamp(0.0, layout.plot.size.height.0);
                let y1 = a.y.0.max(b.y.0).clamp(0.0, layout.plot.size.height.0);
                let w = x1 - x0;
                let h = y1 - y0;
                if w >= 1.0 && h >= 1.0 {
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(5),
                        rect: Rect::new(
                            Point::new(
                                Px(layout.plot.origin.x.0 + x0),
                                Px(layout.plot.origin.y.0 + y0),
                            ),
                            Size::new(Px(w), Px(h)),
                        ),
                        background: selection_fill,
                        border: fret_core::Edges::all(Px(1.0)),
                        border_color: selection_border,
                        corner_radii: fret_core::Corners::all(Px(0.0)),
                    });
                }
            }

            if let (Some(start), Some(end)) = (self.query_drag_start, self.query_drag_current) {
                let x0 = start.x.0.min(end.x.0).clamp(0.0, layout.plot.size.width.0);
                let x1 = start.x.0.max(end.x.0).clamp(0.0, layout.plot.size.width.0);
                let y0 = start.y.0.min(end.y.0).clamp(0.0, layout.plot.size.height.0);
                let y1 = start.y.0.max(end.y.0).clamp(0.0, layout.plot.size.height.0);
                let w = x1 - x0;
                let h = y1 - y0;
                if w >= 1.0 && h >= 1.0 {
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(5),
                        rect: Rect::new(
                            Point::new(
                                Px(layout.plot.origin.x.0 + x0),
                                Px(layout.plot.origin.y.0 + y0),
                            ),
                            Size::new(Px(w), Px(h)),
                        ),
                        background: selection_fill,
                        border: fret_core::Edges::all(Px(1.0)),
                        border_color: selection_border,
                        corner_radii: fret_core::Corners::all(Px(0.0)),
                    });
                }
            }

            if let (Some(start), Some(end)) = (self.box_zoom_start, self.box_zoom_current) {
                let x0 = start.x.0.min(end.x.0).clamp(0.0, layout.plot.size.width.0);
                let x1 = start.x.0.max(end.x.0).clamp(0.0, layout.plot.size.width.0);
                let y0 = start.y.0.min(end.y.0).clamp(0.0, layout.plot.size.height.0);
                let y1 = start.y.0.max(end.y.0).clamp(0.0, layout.plot.size.height.0);
                let w = x1 - x0;
                let h = y1 - y0;
                if w >= 1.0 && h >= 1.0 {
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(5),
                        rect: Rect::new(
                            Point::new(
                                Px(layout.plot.origin.x.0 + x0),
                                Px(layout.plot.origin.y.0 + y0),
                            ),
                            Size::new(Px(w), Px(h)),
                        ),
                        background: selection_fill,
                        border: fret_core::Edges::all(Px(1.0)),
                        border_color: selection_border,
                        corner_radii: fret_core::Corners::all(Px(0.0)),
                    });
                }
            }
        }

        cx.scene.push(SceneOp::PopClip);

        // Legend (P0: shown when there is more than one series; can be moved to overlays later).
        if let Some((rect, rows)) = self.legend_layout(layout) {
            let series_overrides: Vec<Option<Color>> = self
                .model
                .read(cx.app, |_app, m| {
                    L::series_meta(m)
                        .into_iter()
                        .map(|s| s.stroke_color)
                        .collect()
                })
                .unwrap_or_default();
            let series_count = self.legend_entries.len();

            let pad = Px(8.0);
            let gap = Px(8.0);
            let swatch_w = Px(14.0);
            let swatch_h = Px(self.style.stroke_width.0.clamp(2.0, 6.0));
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(6),
                rect,
                background: tooltip_background,
                border: fret_core::Edges::all(Px(1.0)),
                border_color: tooltip_border,
                corner_radii: fret_core::Corners::all(Px(6.0)),
            });

            for (i, entry) in self.legend_entries.iter().enumerate() {
                let row = rows.get(i).copied().unwrap_or(rect);
                let row_h = row.size.height;

                let hovered_row = self.legend_hover == Some(entry.id);
                let pinned_row = state.pinned_series == Some(entry.id);
                if hovered_row || pinned_row {
                    let a = if pinned_row { 0.16 } else { 0.10 };
                    let highlight = Color {
                        a,
                        ..crosshair_color
                    };
                    let inset_x = Px(2.0);
                    let highlight_rect = Rect::new(
                        Point::new(Px(row.origin.x.0 + inset_x.0), row.origin.y),
                        Size::new(
                            Px((row.size.width.0 - inset_x.0 * 2.0).max(0.0)),
                            row.size.height,
                        ),
                    );
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(6),
                        rect: highlight_rect,
                        background: highlight,
                        border: fret_core::Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: fret_core::Corners::all(Px(4.0)),
                    });
                }

                let override_color = series_overrides.get(i).copied().flatten();
                let color = resolve_series_color(i, self.style, series_count, override_color);

                let visible = !state.hidden_series.contains(&entry.id);
                let swatch_color = if visible {
                    color
                } else {
                    Color {
                        a: (color.a * 0.20).clamp(0.05, 0.35),
                        ..color
                    }
                };
                let text_color = if visible {
                    tooltip_text_color
                } else {
                    Color {
                        a: (tooltip_text_color.a * 0.55).clamp(0.25, 0.85),
                        ..tooltip_text_color
                    }
                };

                let row_mid = row.origin.y.0 + row_h.0 * 0.5;
                let swatch_x = Px(row.origin.x.0 + pad.0);
                let swatch_y = Px(row_mid - swatch_h.0 * 0.5);
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(7),
                    rect: Rect::new(
                        Point::new(swatch_x, swatch_y),
                        Size::new(swatch_w, swatch_h),
                    ),
                    background: swatch_color,
                    border: fret_core::Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });

                let text_x = Px(swatch_x.0 + swatch_w.0 + gap.0);
                let text_top = row.origin.y.0 + (row_h.0 - entry.text.metrics.size.height.0) * 0.5;
                let origin = Point::new(text_x, Px(text_top + entry.text.metrics.baseline.0));
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(7),
                    origin,
                    text: entry.text.blob,
                    color: text_color,
                });
            }
        }

        // Axes.
        if layout.plot.size.width.0 > 0.0 && layout.plot.size.height.0 > 0.0 {
            // Y axis line.
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(10),
                rect: Rect::new(
                    layout.plot.origin,
                    Size::new(Px(1.0), layout.plot.size.height),
                ),
                background: axis_color,
                border: fret_core::Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });

            // X axis line.
            let y = Px(layout.plot.origin.y.0 + layout.plot.size.height.0 - 1.0);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(10),
                rect: Rect::new(
                    Point::new(layout.plot.origin.x, y),
                    Size::new(layout.plot.size.width, Px(1.0)),
                ),
                background: axis_color,
                border: fret_core::Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        }

        // Axis labels (P0: evenly spaced ticks).
        let x_ticks = linear_ticks(view_bounds.x_min, view_bounds.x_max, self.style.tick_count);
        let y_ticks = linear_ticks(view_bounds.y_min, view_bounds.y_max, self.style.tick_count);

        let x_den = view_bounds.x_max - view_bounds.x_min;
        let y_den = view_bounds.y_max - view_bounds.y_min;

        for (i, label) in self.axis_labels_x.iter().enumerate() {
            let Some(v) = x_ticks.get(i).copied() else {
                continue;
            };
            if x_den == 0.0 || !x_den.is_finite() {
                continue;
            }
            let t = (v - view_bounds.x_min) / x_den;
            if !t.is_finite() {
                continue;
            }
            let x = layout.plot.origin.x.0 + layout.plot.size.width.0 * t;
            let x = Px(x.round());

            let top = layout.x_axis.origin.y.0 + 2.0;
            let origin = Point::new(
                Px(x.0 - (label.metrics.size.width.0 * 0.5)),
                Px(top + label.metrics.baseline.0),
            );

            cx.scene.push(SceneOp::Text {
                order: DrawOrder(11),
                origin,
                text: label.blob,
                color: label_color,
            });
        }

        for (i, label) in self.axis_labels_y.iter().enumerate() {
            let Some(v) = y_ticks.get(i).copied() else {
                continue;
            };
            if y_den == 0.0 || !y_den.is_finite() {
                continue;
            }
            let t = (v - view_bounds.y_min) / y_den;
            if !t.is_finite() {
                continue;
            }
            let y = layout.plot.origin.y.0 + layout.plot.size.height.0 * (1.0 - t);
            let y = Px(y.round());

            let origin_x = layout.y_axis.origin.x.0 + layout.y_axis.size.width.0
                - label.metrics.size.width.0
                - 4.0;
            let top = y.0 - (label.metrics.size.height.0 * 0.5);
            let origin = Point::new(
                Px(origin_x.max(layout.y_axis.origin.x.0)),
                Px(top + label.metrics.baseline.0),
            );

            cx.scene.push(SceneOp::Text {
                order: DrawOrder(11),
                origin,
                text: label.blob,
                color: label_color,
            });
        }

        // Tooltip (P0: drawn in the same scene; can be moved to overlays later).
        if let Some(hover) = self.hover {
            let (series_count, series_label) = self
                .model
                .read(cx.app, |_app, m| {
                    let series_count = L::series_meta(m).len();
                    let series_label = L::series_label(m, hover.series_id);
                    (series_count, series_label)
                })
                .unwrap_or((0, None));
            let font_size = cx
                .theme()
                .metric_by_key("font.size")
                .unwrap_or(cx.theme().metrics.font_size);
            let style = TextStyle {
                font: FontId::default(),
                size: Px((font_size.0 * 0.90).max(10.0)),
                weight: FontWeight::NORMAL,
                line_height: None,
                letter_spacing_em: None,
            };
            let text = if series_count > 1 {
                if let Some(label) = series_label {
                    format!("{label}  x={:.3}  y={:.3}", hover.data.x, hover.data.y)
                } else {
                    format!(
                        "s={}  x={:.3}  y={:.3}",
                        hover.series_id.0, hover.data.x, hover.data.y
                    )
                }
            } else {
                format!("x={:.3}  y={:.3}", hover.data.x, hover.data.y)
            };
            let constraints = TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: cx.scale_factor,
            };

            let mut key = 0u64;
            key = Self::hash_u64(key, theme.revision);
            key = Self::hash_u64(key, font_stack_key);
            key = Self::hash_u64(key, u64::from(cx.scale_factor.to_bits()));
            for b in text.as_bytes() {
                key = Self::hash_u64(key, u64::from(*b));
            }
            key = Self::hash_u64(key, Self::text_style_key(&style));

            let needs = self.tooltip_text.as_ref().is_none_or(|t| t.key != key);
            if needs {
                if let Some(prev) = self.tooltip_text.take() {
                    cx.services.text().release(prev.blob);
                }
                let prepared = self.prepare_text(cx.services, &text, &style, constraints);
                self.tooltip_text = Some(PreparedText {
                    blob: prepared.blob,
                    metrics: prepared.metrics,
                    key,
                });
            }

            if let Some(tt) = self.tooltip_text {
                let anchor = Point::new(
                    Px(layout.plot.origin.x.0 + hover.plot_px.x.0),
                    Px(layout.plot.origin.y.0 + hover.plot_px.y.0),
                );
                let pad = Px(6.0);
                let gap = Px(10.0);
                let w = Px(tt.metrics.size.width.0 + pad.0 * 2.0);
                let h = Px(tt.metrics.size.height.0 + pad.0 * 2.0);

                let mut x = Px(anchor.x.0 + gap.0);
                let mut y = Px(anchor.y.0 + gap.0);
                if x.0 + w.0 > cx.bounds.origin.x.0 + cx.bounds.size.width.0 {
                    x = Px(anchor.x.0 - gap.0 - w.0);
                }
                if y.0 + h.0 > cx.bounds.origin.y.0 + cx.bounds.size.height.0 {
                    y = Px(anchor.y.0 - gap.0 - h.0);
                }
                x = Px(x.0.max(cx.bounds.origin.x.0));
                y = Px(y.0.max(cx.bounds.origin.y.0));

                let rect = Rect::new(Point::new(x, y), Size::new(w, h));
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(20),
                    rect,
                    background: tooltip_background,
                    border: fret_core::Edges::all(Px(1.0)),
                    border_color: tooltip_border,
                    corner_radii: fret_core::Corners::all(Px(6.0)),
                });

                let origin = Point::new(
                    Px(rect.origin.x.0 + pad.0),
                    Px(rect.origin.y.0 + pad.0 + tt.metrics.baseline.0),
                );
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(21),
                    origin,
                    text: tt.blob,
                    color: tooltip_text_color,
                });
            }
        }
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Viewport);
        cx.set_label("Plot");
    }

    fn cleanup_resources(&mut self, services: &mut dyn UiServices) {
        self.layer.cleanup_resources(services);
        self.clear_axis_label_cache(services);
        self.clear_legend_cache(services);
        if let Some(t) = self.tooltip_text.take() {
            services.text().release(t.blob);
        }
    }
}

impl PlotLayer for LinePlotLayer {
    type Model = LinePlotModel;

    fn data_bounds(model: &Self::Model) -> DataRect {
        model.data_bounds
    }

    fn series_meta(model: &Self::Model) -> Vec<SeriesMeta> {
        model
            .series
            .iter()
            .map(|s| SeriesMeta {
                id: s.id,
                label: s.label.clone(),
                stroke_color: s.stroke_color,
            })
            .collect()
    }

    fn series_label(model: &Self::Model, series_id: SeriesId) -> Option<String> {
        model
            .series
            .iter()
            .find(|s| s.id == series_id)
            .map(|s| s.label.to_string())
    }

    fn paint_paths<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        model: &Self::Model,
        args: PlotPaintArgs<'_>,
    ) -> Vec<(SeriesId, PathId, Color)> {
        let PlotPaintArgs {
            model_revision,
            plot,
            view_bounds,
            style,
            hidden,
        } = args;

        let scale_factor_bits = cx.scale_factor.to_bits();
        let viewport_w_bits = plot.size.width.0.to_bits();
        let viewport_h_bits = plot.size.height.0.to_bits();
        let view_key = data_rect_key(view_bounds);

        let series = &model.series;
        let series_count = series.len();

        if series_count == 0 {
            for cached in self.cached_paths.drain(..) {
                if let Some(id) = cached.id {
                    cx.services.path().release(id);
                }
            }
            return Vec::new();
        }

        let cached_ok = self.cached_paths.len() == series_count
            && self.cached_paths.iter().enumerate().all(|(i, c)| {
                series.get(i).is_some_and(|s| s.id == c.series_id)
                    && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
                    && c.stroke_width == style.stroke_width
                    && c.view_key == view_key
            });

        if cached_ok {
            let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
            for (i, s) in series.iter().enumerate() {
                if hidden.contains(&s.id) {
                    continue;
                }
                let Some(id) = self.cached_paths.get(i).and_then(|c| c.id) else {
                    continue;
                };
                let style = series_style(s, i, style, series_count);
                out.push((s.id, id, style.stroke_color));
            }
            return out;
        }

        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.id {
                cx.services.path().release(id);
            }
        }

        let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size);
        let transform = PlotTransform {
            viewport: local_viewport,
            data: view_bounds,
        };

        let path_style = PathStyle::Stroke(fret_core::StrokeStyle {
            width: style.stroke_width,
        });
        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };

        let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
        self.cached_paths = Vec::with_capacity(series_count);

        for (series_index, s) in series.iter().enumerate() {
            let series_id = s.id;
            if hidden.contains(&series_id) {
                self.cached_paths.push(CachedPath {
                    id: None,
                    series_id,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width: style.stroke_width,
                    view_key,
                    samples: Vec::new(),
                });
                continue;
            }

            let (commands, samples) =
                decimate_polyline(transform, &*s.data, cx.scale_factor, series_id);
            let id = if commands.is_empty() {
                None
            } else {
                let (id, _metrics) = cx
                    .services
                    .path()
                    .prepare(&commands, path_style, constraints);
                Some(id)
            };

            self.cached_paths.push(CachedPath {
                id,
                series_id,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width: style.stroke_width,
                view_key,
                samples,
            });

            if let Some(id) = id {
                let style = series_style(s, series_index, style, series_count);
                out.push((series_id, id, style.stroke_color));
            }
        }

        out
    }

    fn hit_test(&mut self, model: &Self::Model, args: PlotHitTestArgs<'_>) -> Option<PlotHover> {
        let series: Vec<(SeriesId, &dyn SeriesData)> =
            model.series.iter().map(|s| (s.id, &*s.data)).collect();
        hit_test_series_data(&self.cached_paths, &series, args)
    }

    fn cleanup_resources(&mut self, services: &mut dyn UiServices) {
        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.id {
                services.path().release(id);
            }
        }
    }
}

impl PlotLayer for ScatterPlotLayer {
    type Model = ScatterPlotModel;

    fn data_bounds(model: &Self::Model) -> DataRect {
        model.data_bounds
    }

    fn series_meta(model: &Self::Model) -> Vec<SeriesMeta> {
        model
            .series
            .iter()
            .map(|s| SeriesMeta {
                id: s.id,
                label: s.label.clone(),
                stroke_color: s.stroke_color,
            })
            .collect()
    }

    fn series_label(model: &Self::Model, series_id: SeriesId) -> Option<String> {
        model
            .series
            .iter()
            .find(|s| s.id == series_id)
            .map(|s| s.label.to_string())
    }

    fn paint_paths<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        model: &Self::Model,
        args: PlotPaintArgs<'_>,
    ) -> Vec<(SeriesId, PathId, Color)> {
        let PlotPaintArgs {
            model_revision,
            plot,
            view_bounds,
            style,
            hidden,
        } = args;

        let scale_factor_bits = cx.scale_factor.to_bits();
        let viewport_w_bits = plot.size.width.0.to_bits();
        let viewport_h_bits = plot.size.height.0.to_bits();
        let view_key = data_rect_key(view_bounds);

        let series = &model.series;
        let series_count = series.len();

        if series_count == 0 {
            for cached in self.cached_paths.drain(..) {
                if let Some(id) = cached.id {
                    cx.services.path().release(id);
                }
            }
            return Vec::new();
        }

        let cached_ok = self.cached_paths.len() == series_count
            && self.cached_paths.iter().enumerate().all(|(i, c)| {
                series.get(i).is_some_and(|s| s.id == c.series_id)
                    && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
                    && c.stroke_width == style.stroke_width
                    && c.view_key == view_key
            });

        if cached_ok {
            let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
            for (i, s) in series.iter().enumerate() {
                if hidden.contains(&s.id) {
                    continue;
                }
                let Some(id) = self.cached_paths.get(i).and_then(|c| c.id) else {
                    continue;
                };
                let color = resolve_series_color(i, style, series_count, s.stroke_color);
                out.push((s.id, id, color));
            }
            return out;
        }

        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.id {
                cx.services.path().release(id);
            }
        }

        let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size);
        let transform = PlotTransform {
            viewport: local_viewport,
            data: view_bounds,
        };

        let marker_radius = Px((style.stroke_width.0 * 3.0).clamp(2.0, 6.0));
        let path_style = PathStyle::Stroke(fret_core::StrokeStyle {
            width: style.stroke_width,
        });
        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };

        let mut out: Vec<(SeriesId, PathId, Color)> = Vec::with_capacity(series_count);
        self.cached_paths = Vec::with_capacity(series_count);

        for (series_index, s) in series.iter().enumerate() {
            let series_id = s.id;
            if hidden.contains(&series_id) {
                self.cached_paths.push(CachedPath {
                    id: None,
                    series_id,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width: style.stroke_width,
                    view_key,
                    samples: Vec::new(),
                });
                continue;
            }

            let samples = decimate_samples(transform, &*s.data, cx.scale_factor, series_id);
            let commands = scatter_marker_commands(&samples, marker_radius);
            let id = if commands.is_empty() {
                None
            } else {
                let (id, _metrics) = cx
                    .services
                    .path()
                    .prepare(&commands, path_style, constraints);
                Some(id)
            };

            self.cached_paths.push(CachedPath {
                id,
                series_id,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width: style.stroke_width,
                view_key,
                samples,
            });

            if let Some(id) = id {
                let color = resolve_series_color(series_index, style, series_count, s.stroke_color);
                out.push((series_id, id, color));
            }
        }

        out
    }

    fn hit_test(&mut self, model: &Self::Model, args: PlotHitTestArgs<'_>) -> Option<PlotHover> {
        let series: Vec<(SeriesId, &dyn SeriesData)> =
            model.series.iter().map(|s| (s.id, &*s.data)).collect();
        hit_test_series_data(&self.cached_paths, &series, args)
    }

    fn cleanup_resources(&mut self, services: &mut dyn UiServices) {
        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.id {
                services.path().release(id);
            }
        }
    }
}

fn hash_value<T: Hash>(v: &T) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    v.hash(&mut h);
    h.finish()
}

fn hit_test_series_data(
    cached_paths: &[CachedPath],
    series: &[(SeriesId, &dyn SeriesData)],
    args: PlotHitTestArgs<'_>,
) -> Option<PlotHover> {
    let PlotHitTestArgs {
        model_revision,
        plot_size,
        view_bounds,
        scale_factor,
        local,
        style,
        hover_threshold,
        hidden,
        pinned,
    } = args;

    let threshold = hover_threshold.0.max(0.0);
    let threshold2 = threshold * threshold;

    let scale_factor_bits = scale_factor.to_bits();
    let viewport_w_bits = plot_size.width.0.to_bits();
    let viewport_h_bits = plot_size.height.0.to_bits();
    let view_key = data_rect_key(view_bounds);

    let series_count = series.len();
    if series_count == 0 {
        return None;
    }

    let cached_ok = cached_paths.len() == series_count
        && cached_paths.iter().enumerate().all(|(i, c)| {
            series.get(i).is_some_and(|(id, _data)| *id == c.series_id)
                && c.model_revision == model_revision
                && c.scale_factor_bits == scale_factor_bits
                && c.viewport_w_bits == viewport_w_bits
                && c.viewport_h_bits == viewport_h_bits
                && c.stroke_width == style.stroke_width
                && c.view_key == view_key
        });

    let mut best: Option<(SamplePoint, f32)> = None;
    let mut consider_sample = |s: SamplePoint| {
        let dx = s.plot_px.x.0 - local.x.0;
        let dy = s.plot_px.y.0 - local.y.0;
        let d2 = dx * dx + dy * dy;
        if !d2.is_finite() {
            return;
        }
        if best.is_none_or(|b| d2 < b.1) {
            best = Some((s, d2));
        }
    };

    if cached_ok {
        for cached in cached_paths {
            if hidden.contains(&cached.series_id) {
                continue;
            }
            if let Some(pinned) = pinned
                && cached.series_id != pinned
            {
                continue;
            }
            for s in cached.samples.iter().copied() {
                consider_sample(s);
            }
        }
    } else {
        let transform = PlotTransform {
            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot_size),
            data: view_bounds,
        };

        for (series_id, data) in series.iter().copied() {
            if hidden.contains(&series_id) {
                continue;
            }
            if let Some(pinned) = pinned
                && pinned != series_id
            {
                continue;
            }
            for sample in decimate_samples(transform, data, scale_factor, series_id) {
                consider_sample(sample);
            }
        }
    }

    best.and_then(|(s, d2)| {
        (d2 <= threshold2).then_some(PlotHover {
            series_id: s.series_id,
            index: s.index,
            data: s.data,
            plot_px: s.plot_px,
        })
    })
}

fn compute_data_bounds_from_series_data<T>(
    series: &[T],
    data: impl Fn(&T) -> &Series,
) -> Option<DataRect> {
    let mut out: Option<DataRect> = None;

    for s in series {
        let data = data(s);
        let bounds = if let Some(hint) = data.bounds_hint() {
            Some(hint)
        } else if let Some(slice) = data.as_slice() {
            DataRect::from_points(slice.iter().copied())
        } else {
            DataRect::from_points((0..data.len()).filter_map(|i| data.get(i)))
        };

        let Some(bounds) = bounds else {
            continue;
        };

        out = Some(out.map_or(bounds, |acc| acc.union(bounds)));
    }

    out
}
