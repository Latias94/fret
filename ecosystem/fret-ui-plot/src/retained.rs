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
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::cartesian::{DataPoint, DataRect, PlotTransform};
use crate::plot::axis::linear_ticks;
use crate::plot::grid::GridLines;
use crate::plot::view::{
    clamp_view_to_data, clamp_zoom_factors, data_rect_from_plot_points, data_rect_key,
    expand_data_bounds, local_from_absolute, pan_view_by_px, sanitize_data_rect, zoom_view_at_px,
};
use crate::series::{Series, SeriesData};

#[derive(Debug, Clone)]
pub struct LineSeries {
    pub data: Series,
    pub label: Option<Arc<str>>,
    pub stroke_color: Option<Color>,
    pub visible: bool,
}

impl LineSeries {
    pub fn new(data: Series) -> Self {
        Self {
            data,
            label: None,
            stroke_color: None,
            visible: true,
        }
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.stroke_color = Some(color);
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

impl From<Series> for LineSeries {
    fn from(value: Series) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone)]
pub struct LinePlotModel {
    pub data_bounds: DataRect,
    pub series: Vec<LineSeries>,
}

impl LinePlotModel {
    pub fn from_series(series: Vec<LineSeries>) -> Self {
        let bounds = compute_data_bounds_from_series(&series).unwrap_or(DataRect {
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
        }
    }
}

#[derive(Debug)]
struct CachedPath {
    id: Option<PathId>,
    series_index: usize,
    model_revision: u64,
    scale_factor_bits: u32,
    viewport_w_bits: u32,
    viewport_h_bits: u32,
    stroke_width: Px,
    view_key: u64,
    samples: Vec<SamplePoint>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SamplePoint {
    series_index: usize,
    index: usize,
    data: DataPoint,
    /// Point in plot-local logical pixels (origin at plot rect origin).
    plot_px: Point,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct HoverState {
    series_index: usize,
    index: usize,
    data: DataPoint,
    plot_px: Point,
}

#[derive(Debug, Clone, Copy)]
struct PreparedText {
    blob: TextBlobId,
    metrics: TextMetrics,
    key: u64,
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
pub struct LinePlotCanvas {
    model: Model<LinePlotModel>,
    style: LinePlotStyle,
    cached_paths: Vec<CachedPath>,
    hover: Option<HoverState>,
    last_scale_factor: f32,
    view_bounds: Option<DataRect>,
    view_is_auto: bool,
    pan_last_pos: Option<Point>,
    box_zoom_start: Option<Point>,
    box_zoom_current: Option<Point>,
    axis_label_key: Option<u64>,
    axis_labels_x: Vec<PreparedText>,
    axis_labels_y: Vec<PreparedText>,
    legend_key: Option<u64>,
    legend_labels: Vec<PreparedText>,
    tooltip_text: Option<PreparedText>,
}

impl LinePlotCanvas {
    pub fn new(model: Model<LinePlotModel>) -> Self {
        Self {
            model,
            style: LinePlotStyle::default(),
            cached_paths: Vec::new(),
            hover: None,
            last_scale_factor: 1.0,
            view_bounds: None,
            view_is_auto: true,
            pan_last_pos: None,
            box_zoom_start: None,
            box_zoom_current: None,
            axis_label_key: None,
            axis_labels_x: Vec::new(),
            axis_labels_y: Vec::new(),
            legend_key: None,
            legend_labels: Vec::new(),
            tooltip_text: None,
        }
    }

    pub fn style(mut self, style: LinePlotStyle) -> Self {
        self.style = style;
        self
    }

    pub fn create_node<H: UiHost>(ui: &mut fret_ui::UiTree<H>, canvas: Self) -> fret_core::NodeId {
        ui.create_node_retained(canvas)
    }

    fn rebuild_paths_if_needed<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        plot: Rect,
        view_bounds: DataRect,
    ) -> Vec<(PathId, Color)> {
        let model_revision = self.model.revision(cx.app).unwrap_or(0);
        let scale_factor_bits = cx.scale_factor.to_bits();
        let viewport_w_bits = plot.size.width.0.to_bits();
        let viewport_h_bits = plot.size.height.0.to_bits();
        let view_key = data_rect_key(view_bounds);

        let series: Vec<LineSeries> = self
            .model
            .read(cx.app, |_app, m| m.series.clone())
            .unwrap_or_default();
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
                c.series_index == i
                    && c.model_revision == model_revision
                    && c.scale_factor_bits == scale_factor_bits
                    && c.viewport_w_bits == viewport_w_bits
                    && c.viewport_h_bits == viewport_h_bits
                    && c.stroke_width == self.style.stroke_width
                    && c.view_key == view_key
            });

        if cached_ok {
            let mut out: Vec<(PathId, Color)> = Vec::with_capacity(series_count);
            for (i, s) in series.iter().enumerate() {
                if !s.visible {
                    continue;
                }
                let Some(id) = self.cached_paths.get(i).and_then(|c| c.id) else {
                    continue;
                };
                let style = series_style(s, i, self.style, series_count);
                out.push((id, style.stroke_color));
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
            width: self.style.stroke_width,
        });
        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };

        let mut out: Vec<(PathId, Color)> = Vec::with_capacity(series_count);
        self.cached_paths = Vec::with_capacity(series_count);

        for (series_index, s) in series.into_iter().enumerate() {
            let (commands, samples) =
                decimate_polyline(transform, &*s.data, cx.scale_factor, series_index);
            if !s.visible {
                self.cached_paths.push(CachedPath {
                    id: None,
                    series_index,
                    model_revision,
                    scale_factor_bits,
                    viewport_w_bits,
                    viewport_h_bits,
                    stroke_width: self.style.stroke_width,
                    view_key,
                    samples: Vec::new(),
                });
                continue;
            }
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
                series_index,
                model_revision,
                scale_factor_bits,
                viewport_w_bits,
                viewport_h_bits,
                stroke_width: self.style.stroke_width,
                view_key,
                samples,
            });

            if let Some(id) = id {
                let style = series_style(&s, series_index, self.style, series_count);
                out.push((id, style.stroke_color));
            }
        }

        out
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
        for t in self.legend_labels.drain(..) {
            services.text().release(t.blob);
        }
        self.legend_key = None;
    }

    fn legend_layout(&self, layout: PlotLayout) -> Option<(Rect, Vec<Rect>)> {
        if self.legend_labels.len() <= 1 {
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
        for (i, label) in self.legend_labels.iter().enumerate() {
            if i > 0 {
                total_h += row_gap.0;
            }
            max_label_w = max_label_w.max(label.metrics.size.width.0);
            total_h += label.metrics.size.height.0.max(swatch_h.0);
        }

        let legend_w = Px(pad.0 * 2.0 + swatch_w.0 + gap.0 + max_label_w);
        let legend_h = Px(pad.0 * 2.0 + total_h);

        let mut x = Px(layout.plot.origin.x.0 + layout.plot.size.width.0 - legend_w.0 - margin.0);
        let mut y = Px(layout.plot.origin.y.0 + margin.0);
        x = Px(x.0.max(layout.plot.origin.x.0));
        y = Px(y.0.max(layout.plot.origin.y.0));

        let rect = Rect::new(Point::new(x, y), Size::new(legend_w, legend_h));

        let mut rows: Vec<Rect> = Vec::with_capacity(self.legend_labels.len());
        let mut cursor_y = rect.origin.y.0 + pad.0;
        for (i, label) in self.legend_labels.iter().enumerate() {
            let row_h = label.metrics.size.height.0.max(swatch_h.0);
            rows.push(Rect::new(
                Point::new(rect.origin.x, Px(cursor_y)),
                Size::new(rect.size.width, Px(row_h)),
            ));
            cursor_y += row_h;
            if i + 1 < self.legend_labels.len() {
                cursor_y += row_gap.0;
            }
        }

        Some((rect, rows))
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

    fn ensure_view_bounds<H: UiHost>(&mut self, app: &mut H) -> DataRect {
        if self.view_is_auto {
            let data_bounds = self.read_data_bounds(app);
            let view = if self.style.clamp_to_data_bounds {
                expand_data_bounds(data_bounds, self.style.overscroll_fraction)
            } else {
                data_bounds
            };
            self.view_bounds = Some(view);
            return view;
        }

        if let Some(view) = self.view_bounds {
            let view = sanitize_data_rect(view);
            self.view_bounds = Some(view);
            return view;
        }

        let data_bounds = self.read_data_bounds(app);
        self.view_bounds = Some(data_bounds);
        data_bounds
    }

    fn read_data_bounds<H: UiHost>(&self, app: &mut H) -> DataRect {
        let data_bounds = self
            .model
            .read(app, |_app, m| m.data_bounds)
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
        let labels: Vec<Option<Arc<str>>> = self
            .model
            .read(cx.app, |_app, m| {
                m.series.iter().map(|s| s.label.clone()).collect()
            })
            .unwrap_or_default();

        if labels.len() <= 1 {
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
        key = Self::hash_u64(key, u64::from(labels.len() as u32));
        key = Self::hash_u64(key, Self::text_style_key(&style));
        for (i, label) in labels.iter().enumerate() {
            key = Self::hash_u64(key, u64::from(i as u32));
            if let Some(label) = label.as_deref() {
                for b in label.as_bytes() {
                    key = Self::hash_u64(key, u64::from(*b));
                }
            }
        }

        if self.legend_key == Some(key) {
            return;
        }

        self.clear_legend_cache(cx.services);

        self.legend_labels = Vec::with_capacity(labels.len());
        for (i, label) in labels.iter().enumerate() {
            let text = label
                .as_deref()
                .map(str::to_owned)
                .unwrap_or_else(|| format!("Series {}", i + 1));
            let prepared = self.prepare_text(cx.services, &text, &style, constraints);
            self.legend_labels.push(prepared);
        }

        self.legend_key = Some(key);
    }
}

impl<H: UiHost> Widget<H> for LinePlotCanvas {
    fn event(&mut self, cx: &mut fret_ui::retained_bridge::EventCx<'_, H>, event: &Event) {
        match event {
            Event::KeyDown { key, modifiers, .. } => {
                let plain = !modifiers.shift
                    && !modifiers.ctrl
                    && !modifiers.alt
                    && !modifiers.alt_gr
                    && !modifiers.meta;
                if plain && *key == KeyCode::KeyR {
                    self.view_is_auto = true;
                    self.view_bounds = None;
                    self.hover = None;
                    self.pan_last_pos = None;
                    self.box_zoom_start = None;
                    self.box_zoom_current = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
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
                    let _ = self.model.update(cx.app, |m, _cx| {
                        if modifiers.shift {
                            let is_solo = m.series.iter().enumerate().all(|(i, s)| {
                                (i == series_index && s.visible)
                                    || (i != series_index && !s.visible)
                            });
                            if is_solo {
                                for s in &mut m.series {
                                    s.visible = true;
                                }
                            } else {
                                for (i, s) in m.series.iter_mut().enumerate() {
                                    s.visible = i == series_index;
                                }
                            }
                            return;
                        }

                        let visible_count = m.series.iter().filter(|s| s.visible).count();
                        let Some(clicked) = m.series.get_mut(series_index) else {
                            return;
                        };
                        if clicked.visible && visible_count <= 1 {
                            return;
                        }
                        clicked.visible = !clicked.visible;
                    });

                    self.hover = None;
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
                    self.hover = None;
                    if modifiers.shift {
                        let local = local_from_absolute(layout.plot.origin, *position);
                        self.box_zoom_start = Some(local);
                        self.box_zoom_current = Some(local);
                        self.pan_last_pos = None;
                    } else {
                        self.view_is_auto = false;
                        self.pan_last_pos = Some(*position);
                        self.box_zoom_start = None;
                        self.box_zoom_current = None;
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
                    if self.box_zoom_start.is_some() {
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
                                let view_bounds = self.ensure_view_bounds(cx.app);
                                let view_bounds = sanitize_data_rect(view_bounds);
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
                                    self.view_is_auto = false;
                                    self.view_bounds = Some(next);
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
                if self.box_zoom_start.is_some() {
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

                let view_bounds = self.ensure_view_bounds(cx.app);
                let view_bounds = sanitize_data_rect(view_bounds);
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

                self.view_is_auto = false;
                self.view_bounds = Some(next);
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

                if self.box_zoom_start.is_some() {
                    self.box_zoom_current =
                        Some(local_from_absolute(layout.plot.origin, *position));
                    self.hover = None;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if let Some(last) = self.pan_last_pos {
                    let dx_px = position.x.0 - last.x.0;
                    let dy_px = position.y.0 - last.y.0;

                    let view_bounds = self.ensure_view_bounds(cx.app);
                    let view_bounds = sanitize_data_rect(view_bounds);
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

                    self.view_bounds = Some(next);
                    self.view_is_auto = false;
                    self.pan_last_pos = Some(*position);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                let inside = position.x.0 >= layout.plot.origin.x.0
                    && position.y.0 >= layout.plot.origin.y.0
                    && position.x.0 <= layout.plot.origin.x.0 + layout.plot.size.width.0
                    && position.y.0 <= layout.plot.origin.y.0 + layout.plot.size.height.0;

                let next_hover = if inside {
                    let model_revision = self.model.revision(cx.app).unwrap_or(0);
                    let scale_factor = self.last_scale_factor;
                    let scale_factor_bits = scale_factor.to_bits();
                    let viewport_w_bits = layout.plot.size.width.0.to_bits();
                    let viewport_h_bits = layout.plot.size.height.0.to_bits();

                    let view_bounds = self.ensure_view_bounds(cx.app);
                    let view_bounds = sanitize_data_rect(view_bounds);
                    let view_key = data_rect_key(view_bounds);

                    let local = local_from_absolute(layout.plot.origin, *position);

                    let threshold = self.style.hover_threshold.0.max(0.0);
                    let threshold2 = threshold * threshold;

                    let series_count = self
                        .model
                        .read(cx.app, |_app, m| m.series.len())
                        .unwrap_or(0);

                    let cached_ok = self.cached_paths.len() == series_count
                        && self.cached_paths.iter().enumerate().all(|(i, c)| {
                            c.series_index == i
                                && c.model_revision == model_revision
                                && c.scale_factor_bits == scale_factor_bits
                                && c.viewport_w_bits == viewport_w_bits
                                && c.viewport_h_bits == viewport_h_bits
                                && c.stroke_width == self.style.stroke_width
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
                        for cached in &self.cached_paths {
                            for s in cached.samples.iter().copied() {
                                consider_sample(s);
                            }
                        }
                    } else {
                        let series: Vec<LineSeries> = self
                            .model
                            .read(cx.app, |_app, m| m.series.clone())
                            .unwrap_or_default();

                        let transform = PlotTransform {
                            viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size),
                            data: view_bounds,
                        };

                        for (series_index, s) in series.into_iter().enumerate() {
                            if !s.visible {
                                continue;
                            }
                            for sample in
                                decimate_samples(transform, &*s.data, scale_factor, series_index)
                            {
                                consider_sample(sample);
                            }
                        }
                    }

                    best.and_then(|(s, d2)| {
                        (d2 <= threshold2).then_some(HoverState {
                            series_index: s.series_index,
                            index: s.index,
                            data: s.data,
                            plot_px: s.plot_px,
                        })
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
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(&self.model, Invalidation::Paint);
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

        let data_bounds = self.read_data_bounds(cx.app);

        let view_bounds = if self.view_is_auto {
            self.ensure_view_bounds(cx.app)
        } else {
            let view = self.view_bounds.unwrap_or(data_bounds);
            let view = sanitize_data_rect(view);
            self.view_bounds = Some(view);
            view
        };

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

            for (path, color) in self.rebuild_paths_if_needed(cx, layout.plot, view_bounds) {
                cx.scene.push(SceneOp::Path {
                    order: DrawOrder(2),
                    origin: layout.plot.origin,
                    path,
                    color,
                });
            }

            if let Some(hover) = self.hover {
                let x = Px((layout.plot.origin.x.0 + hover.plot_px.x.0).round());
                let y = Px((layout.plot.origin.y.0 + hover.plot_px.y.0).round());

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

                let dot_size = Px(6.0);
                let dot_origin = Point::new(Px(x.0 - dot_size.0 * 0.5), Px(y.0 - dot_size.0 * 0.5));
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(4),
                    rect: Rect::new(dot_origin, Size::new(dot_size, dot_size)),
                    background: crosshair_color,
                    border: fret_core::Edges::all(Px(1.0)),
                    border_color: tooltip_border,
                    corner_radii: fret_core::Corners::all(Px(dot_size.0 * 0.5)),
                });
            }
        }

        cx.scene.push(SceneOp::PopClip);

        // Legend (P0: shown when there is more than one series; can be moved to overlays later).
        if let Some((rect, rows)) = self.legend_layout(layout) {
            let (series_overrides, series_visible): (Vec<Option<Color>>, Vec<bool>) = self
                .model
                .read(cx.app, |_app, m| {
                    (
                        m.series.iter().map(|s| s.stroke_color).collect(),
                        m.series.iter().map(|s| s.visible).collect(),
                    )
                })
                .unwrap_or((Vec::new(), Vec::new()));
            let series_count = self.legend_labels.len();

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

            for (i, label) in self.legend_labels.iter().enumerate() {
                let row = rows.get(i).copied().unwrap_or(rect);
                let row_h = row.size.height;

                let override_color = series_overrides.get(i).copied().flatten();
                let color = resolve_series_color(i, self.style, series_count, override_color);

                let visible = series_visible.get(i).copied().unwrap_or(true);
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
                let text_top = row.origin.y.0 + (row_h.0 - label.metrics.size.height.0) * 0.5;
                let origin = Point::new(text_x, Px(text_top + label.metrics.baseline.0));
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(7),
                    origin,
                    text: label.blob,
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
                    let series_count = m.series.len();
                    let series_label = m
                        .series
                        .get(hover.series_index)
                        .and_then(|s| s.label.as_deref())
                        .map(str::to_owned);
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
                        hover.series_index, hover.data.x, hover.data.y
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
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Viewport);
        cx.set_label("Plot");
    }

    fn cleanup_resources(&mut self, services: &mut dyn UiServices) {
        for cached in self.cached_paths.drain(..) {
            if let Some(id) = cached.id {
                services.path().release(id);
            }
        }
        self.clear_axis_label_cache(services);
        self.clear_legend_cache(services);
        if let Some(t) = self.tooltip_text.take() {
            services.text().release(t.blob);
        }
    }
}

fn hash_value<T: Hash>(v: &T) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    v.hash(&mut h);
    h.finish()
}

fn compute_data_bounds_from_series(series: &[LineSeries]) -> Option<DataRect> {
    let mut x_min: Option<f32> = None;
    let mut x_max: Option<f32> = None;
    let mut y_min: Option<f32> = None;
    let mut y_max: Option<f32> = None;

    let mut consider = |p: DataPoint| {
        if !p.x.is_finite() || !p.y.is_finite() {
            return;
        }
        x_min = Some(x_min.map_or(p.x, |v| v.min(p.x)));
        x_max = Some(x_max.map_or(p.x, |v| v.max(p.x)));
        y_min = Some(y_min.map_or(p.y, |v| v.min(p.y)));
        y_max = Some(y_max.map_or(p.y, |v| v.max(p.y)));
    };

    for s in series {
        if let Some(slice) = s.data.as_slice() {
            for p in slice.iter().copied() {
                consider(p);
            }
        } else {
            for idx in 0..s.data.len() {
                let Some(p) = s.data.get(idx) else {
                    continue;
                };
                consider(p);
            }
        }
    }

    Some(DataRect {
        x_min: x_min?,
        x_max: x_max?,
        y_min: y_min?,
        y_max: y_max?,
    })
}

fn decimate_samples(
    transform: PlotTransform,
    points: &dyn SeriesData,
    scale_factor: f32,
    series_index: usize,
) -> Vec<SamplePoint> {
    let (_commands, samples) = decimate_polyline(transform, points, scale_factor, series_index);
    samples
}

/// Produces a decimated polyline suitable for large datasets.
///
/// Strategy: bucket by device-pixel X (plot-local), then emit min/max Y points per bucket to
/// preserve spikes while bounding the output size to O(plot_width_px).
fn decimate_polyline(
    transform: PlotTransform,
    points: &dyn SeriesData,
    scale_factor: f32,
    series_index: usize,
) -> (Vec<fret_core::PathCommand>, Vec<SamplePoint>) {
    let mut commands: Vec<fret_core::PathCommand> = Vec::new();
    let mut samples: Vec<SamplePoint> = Vec::new();

    let mut segment: Vec<SamplePoint> = Vec::new();

    let mut flush_segment = |segment: &mut Vec<SamplePoint>| {
        if segment.is_empty() {
            return;
        }

        if segment.len() == 1 {
            let p = segment[0];
            commands.push(fret_core::PathCommand::MoveTo(p.plot_px));
            samples.push(p);
            segment.clear();
            return;
        }

        let first = segment[0];
        let last = *segment.last().expect("non-empty segment");

        commands.push(fret_core::PathCommand::MoveTo(first.plot_px));
        samples.push(first);

        let mut last_emitted_idx = first.index;
        let mut last_emitted_point = first.plot_px;

        let bucket_of = |x: Px| -> i32 {
            let x = x.0 * scale_factor.max(1.0);
            if !x.is_finite() { 0 } else { x.floor() as i32 }
        };

        let mut current_bucket: Option<i32> = None;
        let mut min: Option<SamplePoint> = None;
        let mut max: Option<SamplePoint> = None;

        let mut flush_bucket = |min: Option<SamplePoint>, max: Option<SamplePoint>| {
            let (Some(min), Some(max)) = (min, max) else {
                return;
            };

            let mut a = min;
            let mut b = max;
            if a.index > b.index {
                std::mem::swap(&mut a, &mut b);
            }

            for p in [a, b] {
                if p.index <= last_emitted_idx {
                    continue;
                }
                if p.plot_px == last_emitted_point {
                    last_emitted_idx = p.index;
                    continue;
                }
                commands.push(fret_core::PathCommand::LineTo(p.plot_px));
                samples.push(p);
                last_emitted_idx = p.index;
                last_emitted_point = p.plot_px;
            }
        };

        // Exclude endpoints from bucketing (they are emitted explicitly).
        for p in segment
            .iter()
            .copied()
            .skip(1)
            .take(segment.len().saturating_sub(2))
        {
            let b = bucket_of(p.plot_px.x);
            if current_bucket != Some(b) {
                flush_bucket(min.take(), max.take());
                current_bucket = Some(b);
                min = Some(p);
                max = Some(p);
                continue;
            }

            if let Some(m) = min
                && p.plot_px.y.0.is_finite()
                && m.plot_px.y.0.is_finite()
                && p.plot_px.y.0 < m.plot_px.y.0
            {
                min = Some(p);
            }
            if let Some(m) = max
                && p.plot_px.y.0.is_finite()
                && m.plot_px.y.0.is_finite()
                && p.plot_px.y.0 > m.plot_px.y.0
            {
                max = Some(p);
            }
        }

        flush_bucket(min.take(), max.take());

        if last.index > last_emitted_idx && last.plot_px != last_emitted_point {
            commands.push(fret_core::PathCommand::LineTo(last.plot_px));
            samples.push(last);
        } else if last.index > last_emitted_idx && last.plot_px == last_emitted_point {
            // Keep sample indices monotonic for hover even if the point collapses.
            samples.push(last);
        }

        segment.clear();
    };

    if let Some(slice) = points.as_slice() {
        for (idx, p) in slice.iter().copied().enumerate() {
            if !p.x.is_finite() || !p.y.is_finite() {
                flush_segment(&mut segment);
                continue;
            }
            let px = transform.data_to_px(p);
            if !px.x.0.is_finite() || !px.y.0.is_finite() {
                flush_segment(&mut segment);
                continue;
            }
            segment.push(SamplePoint {
                series_index,
                index: idx,
                data: p,
                plot_px: px,
            });
        }
    } else {
        for idx in 0..points.len() {
            let Some(p) = points.get(idx) else {
                flush_segment(&mut segment);
                continue;
            };
            if !p.x.is_finite() || !p.y.is_finite() {
                flush_segment(&mut segment);
                continue;
            }
            let px = transform.data_to_px(p);
            if !px.x.0.is_finite() || !px.y.0.is_finite() {
                flush_segment(&mut segment);
                continue;
            }
            segment.push(SamplePoint {
                series_index,
                index: idx,
                data: p,
                plot_px: px,
            });
        }
    }

    flush_segment(&mut segment);

    (commands, samples)
}
