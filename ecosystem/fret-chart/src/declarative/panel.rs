use fret_core::time::{Duration, Instant};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use delinea::engine::model::{ChartPatch, PatchMode};
use delinea::engine::window::DataWindow;
use delinea::marks::{MarkKind, MarkPayloadRef, MarkTree};
use delinea::tooltip::TooltipOutput;
use delinea::{Action, ChartEngine, WorkBudget};
use fret_canvas::ui::{
    CanvasToolDownResult, CanvasToolEntry, CanvasToolHandlers, CanvasToolId, CanvasToolRouterProps,
    OnCanvasToolPinch, OnCanvasToolPointerDown, OnCanvasToolPointerMove, OnCanvasToolPointerUp,
    OnCanvasToolWheel, PanZoomCanvasPaintCx, PanZoomCanvasSurfacePanelProps,
    canvas_tool_router_panel,
};
use fret_core::{
    Color, Corners, DrawOrder, Edges, KeyCode, MouseButton, PathCommand, PathStyle, Point, Px,
    Rect, SemanticsRole, Size, StrokeStyle,
};
use fret_runtime::Model;
use fret_ui::action::OnKeyDown;
use fret_ui::canvas::CanvasPainter;
use fret_ui::element::{
    AnyElement, CanvasProps, FocusScopeProps, Length, ManagedSurfaceProps, PointerRegionProps,
    SemanticsProps,
};
use fret_ui::{ElementContext, ElementContextAccess, UiHost};
use std::collections::BTreeSet;

use crate::a11y::ChartA11yIndex;
use crate::input_map::{ChartInputMap, ModifierKey};
use crate::linking::{AxisPointerLinkAnchor, BrushSelectionLink2D, ChartLinkRouter, LinkAxisKey};
use crate::output::{
    ChartCanvasOutput, chart_canvas_output_link_events_batch,
    chart_canvas_output_snapshot_for_engine, update_chart_canvas_output,
};
use crate::retained::ChartStyle;
use crate::{DefaultTooltipFormatter, TooltipFormatter, TooltipTextLine};

use super::legend_overlay::{LegendOverlayState, LegendSeriesEntry, legend_overlay_tool};
use super::tooltip_overlay::{AxisPointerLabelOverlay, TooltipOverlayState, tooltip_overlay_tool};

#[derive(Debug, Default)]
struct NullTextMeasurer;

impl delinea::text::TextMeasurer for NullTextMeasurer {
    fn measure(
        &mut self,
        _text: delinea::ids::StringId,
        _style: delinea::text::TextStyleId,
    ) -> delinea::text::TextMetrics {
        delinea::text::TextMetrics::default()
    }
}

#[derive(Debug, Clone, Copy)]
struct ChartPanDrag {
    start_pos: Point,
    x_axis: delinea::AxisId,
    y_axis: delinea::AxisId,
    start_x: DataWindow,
    start_y: DataWindow,
}

fn default_chart_input_map_safe() -> ChartInputMap {
    let mut map = ChartInputMap::default();
    map.wheel_zoom_mod = Some(ModifierKey::Ctrl);
    map
}

fn primary_axes(engine: &ChartEngine) -> Option<(delinea::AxisId, delinea::AxisId)> {
    let model = engine.model();
    for id in &model.series_order {
        let s = model.series.get(id)?;
        if s.visible {
            return Some((s.x_axis, s.y_axis));
        }
    }
    None
}

fn fallback_window() -> DataWindow {
    DataWindow { min: 0.0, max: 1.0 }
}

fn window_for_axis_x(engine: &ChartEngine, axis: delinea::AxisId) -> DataWindow {
    engine
        .output()
        .axis_windows
        .get(&axis)
        .copied()
        .unwrap_or_else(fallback_window)
}

fn window_for_axis_y(engine: &ChartEngine, axis: delinea::AxisId) -> DataWindow {
    engine
        .output()
        .axis_windows
        .get(&axis)
        .copied()
        .unwrap_or_else(fallback_window)
}

fn paint_color(style: ChartStyle, paint: delinea::PaintId) -> Color {
    let palette = &style.series_palette;
    palette[(paint.0 as usize) % palette.len()]
}

fn series_color(style: ChartStyle, series: delinea::SeriesId) -> Color {
    let palette = &style.series_palette;
    palette[(series.0 as usize) % palette.len()]
}

#[track_caller]
fn ensure_engine_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<ChartEngine>>,
    spec: delinea::ChartSpec,
) -> Model<ChartEngine> {
    if let Some(model) = controlled {
        return model;
    }

    let mut spec = spec;
    spec.axis_pointer.get_or_insert_with(Default::default);
    cx.local_model(|| ChartEngine::new(spec).expect("chart spec should be valid"))
}

#[derive(Debug, Clone)]
struct MarksCache {
    marks_rev: delinea::ids::Revision,
    output_rev: delinea::ids::Revision,
    marks: Arc<MarkTree>,
    axis_pointer: Option<AxisPointerPaintData>,
    hover_point_px: Option<Point>,
}

impl Default for MarksCache {
    fn default() -> Self {
        Self {
            marks_rev: delinea::ids::Revision::default(),
            output_rev: delinea::ids::Revision::default(),
            marks: Arc::new(MarkTree::default()),
            axis_pointer: None,
            hover_point_px: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct AxisPointerPaintData {
    crosshair_px: Point,
    shadow_rect_px: Option<Rect>,
    draw_x: bool,
    draw_y: bool,
}

#[derive(Debug, Default, Clone)]
struct ChartA11yState {
    index: ChartA11yIndex,
    last_key: Option<(delinea::SeriesId, u32)>,
    marks_rev: delinea::ids::Revision,
    series_rank_by_id: BTreeMap<delinea::SeriesId, usize>,
}

#[derive(Debug, Default, Clone)]
struct ChartA11ySemanticsState {
    pos_in_set: Option<u32>,
    set_size: Option<u32>,
    value: Option<Arc<str>>,
}

fn a11y_tooltip_value(engine: &ChartEngine, formatter: &dyn TooltipFormatter) -> Option<Arc<str>> {
    let output = engine.output();
    let axis_pointer = output.axis_pointer.as_ref()?;

    let mut parts: Vec<String> = Vec::new();
    match &axis_pointer.tooltip {
        TooltipOutput::Item(item) => {
            let x_window = output
                .axis_windows
                .get(&item.x_axis)
                .copied()
                .unwrap_or_default();
            let x_label = engine
                .model()
                .axes
                .get(&item.x_axis)
                .and_then(|axis| axis.name.as_deref())
                .unwrap_or("X");
            let x_value = delinea::engine::axis::format_value_for(
                engine.model(),
                item.x_axis,
                x_window,
                item.x_value,
            );
            parts.push(format!("{x_label}: {x_value}"));
        }
        TooltipOutput::Axis(axis) => {
            let axis_window = output
                .axis_windows
                .get(&axis.axis)
                .copied()
                .unwrap_or_default();
            let axis_label = engine
                .model()
                .axes
                .get(&axis.axis)
                .and_then(|axis| axis.name.as_deref())
                .unwrap_or("Axis");
            let axis_value = delinea::engine::axis::format_value_for(
                engine.model(),
                axis.axis,
                axis_window,
                axis.axis_value,
            );
            parts.push(format!("{axis_label}: {axis_value}"));
        }
    }

    let lines = formatter.format_axis_pointer(engine, &output.axis_windows, axis_pointer);
    for line in lines {
        parts.push(if let Some((left, right)) = line.columns {
            format!("{left}: {right}")
        } else {
            line.text
        });
    }

    (!parts.is_empty()).then(|| Arc::from(parts.join(" | ")))
}

fn series_row_count(engine: &mut ChartEngine, series: delinea::SeriesId) -> Option<u32> {
    let dataset = {
        let model = engine.model();
        let series = model.series.get(&series)?;
        model.root_dataset_id(series.dataset)
    };

    engine
        .datasets_mut()
        .dataset(dataset)
        .and_then(|table| u32::try_from(table.row_count()).ok())
}

fn current_a11y_key_from_engine(
    engine: &mut ChartEngine,
    a11y_state: &ChartA11yState,
) -> Option<(delinea::SeriesId, u32)> {
    let first_from_marks = a11y_state
        .index
        .series_by_index
        .iter()
        .next()
        .and_then(|(data_index, series)| Some((*series.first()?, *data_index)));
    let engine_hit = engine
        .output()
        .axis_pointer
        .as_ref()
        .and_then(|o| o.hit)
        .map(|hit| (hit.series, hit.data_index));
    let fallback_first = engine
        .model()
        .series_order
        .clone()
        .into_iter()
        .find(|s| series_row_count(engine, *s).is_some_and(|n| n > 0))
        .map(|s| (s, 0));

    a11y_state
        .last_key
        .or(engine_hit)
        .or(first_from_marks)
        .or(fallback_first)
}

fn a11y_semantics_for_engine(
    engine: &mut ChartEngine,
    a11y_state: &ChartA11yState,
    formatter: &dyn TooltipFormatter,
) -> ChartA11ySemanticsState {
    let mut semantics = ChartA11ySemanticsState {
        value: a11y_tooltip_value(engine, formatter),
        ..Default::default()
    };
    let Some((series, data_index)) = current_a11y_key_from_engine(engine, a11y_state) else {
        return semantics;
    };

    if let Some(indices) = a11y_state.index.indices_by_series.get(&series) {
        semantics.set_size = u32::try_from(indices.len()).ok().filter(|n| *n > 0);
        semantics.pos_in_set = indices
            .binary_search(&data_index)
            .ok()
            .and_then(|pos| u32::try_from(pos + 1).ok());
        return semantics;
    }

    if let Some(set_size) = series_row_count(engine, series).filter(|n| *n > 0) {
        semantics.set_size = Some(set_size);
        semantics.pos_in_set = Some(data_index.min(set_size.saturating_sub(1)) + 1);
    }

    semantics
}

fn px_at_data(window: DataWindow, value: f64, origin_px: f32, span_px: f32) -> f32 {
    let mut window = window;
    window.clamp_non_degenerate();
    let span = window.span();
    if !span.is_finite() || span <= 0.0 || !span_px.is_finite() || span_px <= 0.0 {
        return origin_px;
    }
    let t = ((value - window.min) / span).clamp(0.0, 1.0) as f32;
    origin_px + t * span_px
}

fn y_local_for_data_value(window: DataWindow, value: f64, plot_height_px: f32) -> f32 {
    let mut window = window;
    window.clamp_non_degenerate();
    let span = window.span();
    if !span.is_finite() || span <= 0.0 || !value.is_finite() {
        return plot_height_px;
    }
    let t = ((value - window.min) / span).clamp(0.0, 1.0) as f32;
    plot_height_px * (1.0 - t)
}

fn point_for_series_data_index(
    engine: &mut ChartEngine,
    bounds: Rect,
    series: delinea::SeriesId,
    data_index: u32,
) -> Option<Point> {
    let plot_w = bounds.size.width.0;
    let plot_h = bounds.size.height.0;
    if plot_w <= 0.0 || plot_h <= 0.0 {
        return None;
    }

    let (x_axis, y_axis, x_value, y_value) = {
        let (dataset, x_axis, y_axis, x_col, y_col) = {
            let model = engine.model();
            let series = model.series.get(&series)?;
            let dataset = model.root_dataset_id(series.dataset);
            let dataset_model = model.datasets.get(&series.dataset)?;
            let x_col = *dataset_model.fields.get(&series.encode.x)?;
            let y_col = *dataset_model.fields.get(&series.encode.y)?;
            (dataset, series.x_axis, series.y_axis, x_col, y_col)
        };

        let table = engine.datasets_mut().dataset(dataset)?;
        let idx = usize::try_from(data_index).ok()?;
        let x_value = table.column_f64(x_col)?.get(idx).copied()?;
        let y_value = table.column_f64(y_col)?.get(idx).copied()?;
        (x_axis, y_axis, x_value, y_value)
    };

    let x_window = window_for_axis_x(engine, x_axis);
    let y_window = window_for_axis_y(engine, y_axis);
    let x_local = px_at_data(x_window, x_value, 0.0, plot_w);
    let y_local = y_local_for_data_value(y_window, y_value, plot_h);
    Some(Point::new(
        Px(bounds.origin.x.0 + x_local),
        Px(bounds.origin.y.0 + y_local),
    ))
}

fn point_for_axis_pointer_anchor(
    engine: &ChartEngine,
    router: &ChartLinkRouter,
    axis: delinea::AxisId,
    value: f64,
) -> Option<Point> {
    if !value.is_finite() {
        return None;
    }

    let output = engine.output();
    let axis_model = engine.model().axes.get(&axis)?;
    let plot = output
        .plot_viewports_by_grid
        .get(&axis_model.grid)
        .copied()
        .or(output.viewport)
        .or_else(|| output.plot_viewports_by_grid.values().next().copied())?;
    let axis_window = output.axis_windows.get(&axis).copied()?;

    let px = match router.axis_key(axis)?.kind {
        delinea::AxisKind::X => {
            let x = delinea::engine::axis::x_px_at_data_in_rect(axis_window, value, plot);
            let y = plot.origin.y.0 + 0.5 * plot.size.height.0;
            Point::new(Px(x), Px(y))
        }
        delinea::AxisKind::Y => {
            let x = plot.origin.x.0 + 0.5 * plot.size.width.0;
            let y = delinea::engine::axis::y_px_at_data_in_rect(axis_window, value, plot);
            Point::new(Px(x), Px(y))
        }
    };

    (px.x.0.is_finite() && px.y.0.is_finite()).then_some(px)
}

fn navigate_a11y_index(
    a11y_state: &ChartA11yState,
    current: (delinea::SeriesId, u32),
    key: KeyCode,
) -> Option<(delinea::SeriesId, u32)> {
    let (series, data_index) = current;
    match key {
        KeyCode::ArrowLeft => a11y_state
            .index
            .indices_by_series
            .get(&series)
            .and_then(|indices| match indices.binary_search(&data_index) {
                Ok(pos) | Err(pos) => pos.checked_sub(1).and_then(|i| indices.get(i).copied()),
            })
            .map(|next_index| (series, next_index)),
        KeyCode::ArrowRight => a11y_state
            .index
            .indices_by_series
            .get(&series)
            .and_then(|indices| match indices.binary_search(&data_index) {
                Ok(pos) => indices.get(pos + 1).copied(),
                Err(pos) => indices.get(pos).copied(),
            })
            .map(|next_index| (series, next_index)),
        KeyCode::ArrowUp | KeyCode::ArrowDown => a11y_state
            .index
            .series_by_index
            .get(&data_index)
            .and_then(|series_ids| {
                let pos = series_ids.iter().position(|s| *s == series).unwrap_or(0);
                let next_pos = match key {
                    KeyCode::ArrowUp => pos.checked_sub(1),
                    KeyCode::ArrowDown => (pos + 1 < series_ids.len()).then_some(pos + 1),
                    _ => None,
                }?;
                series_ids.get(next_pos).copied().map(|s| (s, data_index))
            }),
        _ => None,
    }
}

fn navigate_a11y_fallback(
    engine: &mut ChartEngine,
    current: Option<(delinea::SeriesId, u32)>,
    key: KeyCode,
) -> Option<(delinea::SeriesId, u32)> {
    let series_order = engine.model().series_order.clone();
    if series_order.is_empty() {
        return None;
    }

    let mut current_series = current.map(|(s, _)| s);
    let mut current_index = current.map(|(_, i)| i).unwrap_or(0);
    if current_series
        .and_then(|s| series_row_count(engine, s).filter(|n| *n > 0))
        .is_none()
    {
        current_series = series_order
            .iter()
            .copied()
            .find(|s| series_row_count(engine, *s).is_some_and(|n| n > 0));
    }

    let current_series = current_series?;
    let current_row_count = series_row_count(engine, current_series).filter(|n| *n > 0)?;
    current_index = current_index.min(current_row_count.saturating_sub(1));

    match key {
        KeyCode::ArrowLeft => Some((current_series, current_index.saturating_sub(1))),
        KeyCode::ArrowRight => Some((
            current_series,
            (current_index + 1).min(current_row_count.saturating_sub(1)),
        )),
        KeyCode::ArrowUp | KeyCode::ArrowDown => {
            let pos = series_order
                .iter()
                .position(|s| *s == current_series)
                .unwrap_or(0) as i32;
            let step = if key == KeyCode::ArrowUp { -1 } else { 1 };
            let mut next_pos = pos + step;
            let mut next_series = current_series;
            while next_pos >= 0 && (next_pos as usize) < series_order.len() {
                let candidate = series_order[next_pos as usize];
                if series_row_count(engine, candidate).is_some_and(|n| n > 0) {
                    next_series = candidate;
                    break;
                }
                next_pos += step;
            }

            let next_row_count = series_row_count(engine, next_series).unwrap_or(0);
            if next_row_count == 0 {
                return None;
            }
            Some((
                next_series,
                current_index.min(next_row_count.saturating_sub(1)),
            ))
        }
        _ => None,
    }
}

fn handle_a11y_navigation(
    engine: &mut ChartEngine,
    a11y_state: &mut ChartA11yState,
    bounds: Rect,
    key: KeyCode,
) -> bool {
    if !matches!(
        key,
        KeyCode::ArrowLeft | KeyCode::ArrowRight | KeyCode::ArrowUp | KeyCode::ArrowDown
    ) {
        return false;
    }

    let current = current_a11y_key_from_engine(engine, a11y_state);
    let next = current
        .and_then(|current| navigate_a11y_index(a11y_state, current, key))
        .or_else(|| navigate_a11y_fallback(engine, current, key));
    let Some((next_series, next_index)) = next else {
        return false;
    };

    let point = a11y_state
        .index
        .point(next_series, next_index)
        .or_else(|| point_for_series_data_index(engine, bounds, next_series, next_index));
    let Some(point) = point else {
        return false;
    };

    engine.apply_action(Action::HoverAt { point });
    a11y_state.last_key = Some((next_series, next_index));
    true
}

#[derive(Debug, Default, Clone)]
struct ChartCanvasPanelOutputState {
    output: ChartCanvasOutput,
}

#[derive(Debug, Default, Clone)]
struct ChartCanvasPanelLinkedState {
    domain_windows_model_revision: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartCanvasPanelMode {
    Full,
    GridView(delinea::GridId),
    Overlay,
}

impl Default for ChartCanvasPanelMode {
    fn default() -> Self {
        Self::Full
    }
}

impl ChartCanvasPanelMode {
    fn grid(self) -> Option<delinea::GridId> {
        match self {
            Self::GridView(grid) => Some(grid),
            Self::Full | Self::Overlay => None,
        }
    }

    fn renders_marks(self) -> bool {
        !matches!(self, Self::Overlay)
    }

    fn renders_overlays(self) -> bool {
        matches!(self, Self::Full | Self::Overlay)
    }

    fn default_test_id(self) -> Option<Arc<str>> {
        match self {
            Self::Full => None,
            Self::GridView(grid) => Some(Arc::from(format!("fret-chart-grid-{}", grid.0))),
            Self::Overlay => Some(Arc::from("fret-chart-overlay")),
        }
    }
}

#[derive(Clone)]
pub struct ChartCanvasPanelProps {
    pub pointer_region: PointerRegionProps,
    pub canvas: CanvasProps,

    /// When `None`, an internal engine model is created once from `spec`.
    pub engine: Option<Model<ChartEngine>>,
    pub spec: delinea::ChartSpec,
    pub output_model: Option<Model<ChartCanvasOutput>>,

    /// Enables the focusable chart semantics layer with arrow-key point navigation.
    pub accessibility_layer: bool,
    pub test_id: Option<Arc<str>>,

    /// Optional formatter hook for axis-trigger tooltips (ADR 0209).
    ///
    /// When `None`, `DefaultTooltipFormatter` is used.
    pub tooltip_formatter: Option<Arc<dyn TooltipFormatter>>,

    /// Chart interaction mapping (ImPlot-aligned). Defaults to a "safe" wheel mapping
    /// (zoom requires Ctrl), because charts are often embedded inside scroll containers.
    pub input_map: ChartInputMap,
    pub link_axis_map: BTreeMap<delinea::AxisId, LinkAxisKey>,
    pub linked_brush_model: Option<Model<Option<BrushSelectionLink2D>>>,
    pub linked_axis_pointer_model: Option<Model<Option<AxisPointerLinkAnchor>>>,
    pub linked_domain_windows_model: Option<Model<BTreeMap<LinkAxisKey, Option<DataWindow>>>>,
    pub mode: ChartCanvasPanelMode,

    pub style: ChartStyle,
}

impl ChartCanvasPanelProps {
    pub fn new(spec: delinea::ChartSpec) -> Self {
        Self {
            pointer_region: PointerRegionProps::default(),
            canvas: CanvasProps::default(),
            engine: None,
            spec,
            output_model: None,
            accessibility_layer: false,
            test_id: None,
            tooltip_formatter: None,
            input_map: default_chart_input_map_safe(),
            link_axis_map: BTreeMap::new(),
            linked_brush_model: None,
            linked_axis_pointer_model: None,
            linked_domain_windows_model: None,
            mode: ChartCanvasPanelMode::Full,
            style: ChartStyle::default(),
        }
    }

    pub fn output_model(mut self, output: Model<ChartCanvasOutput>) -> Self {
        self.output_model = Some(output);
        self
    }

    pub fn link_axis_map(mut self, map: BTreeMap<delinea::AxisId, LinkAxisKey>) -> Self {
        self.link_axis_map = map;
        self
    }

    pub fn linked_brush(mut self, brush: Model<Option<BrushSelectionLink2D>>) -> Self {
        self.linked_brush_model = Some(brush);
        self
    }

    pub fn linked_axis_pointer(
        mut self,
        axis_pointer: Model<Option<AxisPointerLinkAnchor>>,
    ) -> Self {
        self.linked_axis_pointer_model = Some(axis_pointer);
        self
    }

    pub fn linked_domain_windows(
        mut self,
        windows: Model<BTreeMap<LinkAxisKey, Option<DataWindow>>>,
    ) -> Self {
        self.linked_domain_windows_model = Some(windows);
        self
    }

    pub fn input_map(mut self, map: ChartInputMap) -> Self {
        self.input_map = map;
        self
    }

    pub fn accessibility_layer(mut self, enabled: bool) -> Self {
        self.accessibility_layer = enabled;
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn grid_view(mut self, grid: delinea::GridId) -> Self {
        self.mode = ChartCanvasPanelMode::GridView(grid);
        self
    }

    pub fn overlay_only(mut self) -> Self {
        self.mode = ChartCanvasPanelMode::Overlay;
        self
    }
}

fn series_is_in_panel_grid(
    mode: ChartCanvasPanelMode,
    model: &delinea::engine::model::ChartModel,
    series: delinea::SeriesId,
) -> bool {
    let Some(grid) = mode.grid() else {
        return true;
    };
    let Some(series) = model.series.get(&series) else {
        return false;
    };
    model
        .axes
        .get(&series.x_axis)
        .is_some_and(|axis| axis.grid == grid)
}

#[track_caller]
pub fn chart_canvas_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    mut props: ChartCanvasPanelProps,
) -> AnyElement {
    props.pointer_region.layout.size.width = Length::Fill;
    props.pointer_region.layout.size.height = Length::Fill;
    props.canvas.layout.size.width = Length::Fill;
    props.canvas.layout.size.height = Length::Fill;

    let engine = ensure_engine_model(cx, props.engine.clone(), props.spec.clone());

    // Tool-local drag model.
    let pan_drag: Model<Option<ChartPanDrag>> = cx.local_model(|| None::<ChartPanDrag>);
    let a11y_state_model: Model<ChartA11yState> = cx.local_model(ChartA11yState::default);

    let legend_state: Arc<Mutex<LegendOverlayState>> = cx.slot_state(
        || Arc::new(Mutex::new(LegendOverlayState::default())),
        |st| st.clone(),
    );
    let tooltip_state: Arc<Mutex<TooltipOverlayState>> = cx.slot_state(
        || Arc::new(Mutex::new(TooltipOverlayState::default())),
        |st| st.clone(),
    );

    let default_tooltip_formatter: Arc<dyn TooltipFormatter> = cx.slot_state(
        || Arc::new(DefaultTooltipFormatter) as Arc<dyn TooltipFormatter>,
        |st| st.clone(),
    );
    let tooltip_formatter: Arc<dyn TooltipFormatter> = props
        .tooltip_formatter
        .clone()
        .unwrap_or(default_tooltip_formatter);

    // Step the engine during declarative render and cache the current marks snapshot.
    let bounds = cx.bounds;
    let mode = props.mode;
    let panel_grid = mode.grid();
    let mut unfinished = false;

    let marks_cache_slot = cx.slot_id();
    let (prev_marks_rev, prev_output_rev) =
        cx.state_for(marks_cache_slot, MarksCache::default, |cache| {
            (cache.marks_rev, cache.output_rev)
        });
    let output_state_slot = cx.slot_id();
    let a11y_semantics_state_slot = cx.slot_id();
    let linked_state_slot = cx.slot_id();

    let mut marks_rev = prev_marks_rev;
    let mut output_rev = prev_output_rev;
    let mut output_marks: Option<Arc<MarkTree>> = None;

    let mut legend_series: Vec<LegendSeriesEntry> = Vec::new();
    let mut series_rank_by_id: BTreeMap<delinea::SeriesId, usize> = BTreeMap::default();
    let mut series_visible_in_panel: BTreeSet<delinea::SeriesId> = BTreeSet::new();
    let mut axis_pointer_output: Option<delinea::engine::AxisPointerOutput> = None;
    let mut axis_pointer_labels: Vec<AxisPointerLabelOverlay> = Vec::new();
    let mut tooltip_lines: Vec<TooltipTextLine> = Vec::new();

    let mut axis_pointer: Option<AxisPointerPaintData> = None;
    let mut hover_point_px: Option<Point> = None;

    let output_model = props.output_model.clone();
    if let Some(output_model) = output_model.as_ref() {
        cx.observe_model(output_model, fret_ui::Invalidation::Paint);
    }
    let output_state_before = output_model.as_ref().map(|_| {
        cx.state_for(
            output_state_slot,
            ChartCanvasPanelOutputState::default,
            |state| state.output.clone(),
        )
    });
    let mut next_published_output: Option<ChartCanvasOutput> = None;

    let tooltip_formatter_c = tooltip_formatter.clone();
    let explicit_link_axis_map = props.link_axis_map.clone();
    let linked_brush_model = props.linked_brush_model.clone();
    let linked_axis_pointer_model = props.linked_axis_pointer_model.clone();
    let linked_domain_windows_model = props.linked_domain_windows_model.clone();

    if let Some(model) = linked_brush_model.as_ref() {
        cx.observe_model(model, fret_ui::Invalidation::Paint);
    }
    if let Some(model) = linked_axis_pointer_model.as_ref() {
        cx.observe_model(model, fret_ui::Invalidation::Paint);
    }
    if let Some(model) = linked_domain_windows_model.as_ref() {
        cx.observe_model(model, fret_ui::Invalidation::Paint);
    }

    let linked_domain_windows_revision = linked_domain_windows_model
        .as_ref()
        .and_then(|model| model.revision(cx.app));
    let linked_domain_windows_should_sync = linked_domain_windows_model.is_some()
        && linked_domain_windows_revision.is_some_and(|rev| {
            cx.state_for(
                linked_state_slot,
                ChartCanvasPanelLinkedState::default,
                |state| {
                    if state.domain_windows_model_revision == Some(rev) {
                        false
                    } else {
                        state.domain_windows_model_revision = Some(rev);
                        true
                    }
                },
            )
        });
    let linked_brush = linked_brush_model
        .as_ref()
        .and_then(|model| model.read(cx.app, |_app, selection| *selection).ok());
    let linked_axis_pointer = linked_axis_pointer_model
        .as_ref()
        .and_then(|model| model.read(cx.app, |_app, anchor| anchor.clone()).ok());
    let linked_domain_windows = if linked_domain_windows_should_sync {
        linked_domain_windows_model
            .as_ref()
            .and_then(|model| model.read(cx.app, |_app, windows| windows.clone()).ok())
    } else {
        None
    };
    let mut linked_inputs_changed = false;

    let _ = engine.update(cx.app, |engine, _cx| {
        match mode {
            ChartCanvasPanelMode::Full => {
                if engine.model().viewport != Some(bounds) {
                    let _ = engine.apply_patch(
                        ChartPatch {
                            viewport: Some(Some(bounds)),
                            ..ChartPatch::default()
                        },
                        PatchMode::Merge,
                    );
                }
            }
            ChartCanvasPanelMode::GridView(grid) => {
                if engine.model().plot_viewports_by_grid.get(&grid).copied() != Some(bounds) {
                    let mut patch = ChartPatch::default();
                    patch.plot_viewports_by_grid.insert(grid, Some(bounds));
                    let _ = engine.apply_patch(patch, PatchMode::Merge);
                }
            }
            ChartCanvasPanelMode::Overlay => {}
        }

        let mut router = ChartLinkRouter::from_model(engine.model());
        if !explicit_link_axis_map.is_empty() {
            let explicit = explicit_link_axis_map
                .iter()
                .filter_map(|(axis, key)| {
                    engine
                        .model()
                        .axes
                        .contains_key(axis)
                        .then_some((*axis, *key))
                })
                .collect();
            router = router.with_explicit_axis_map(explicit);
        }

        if let Some(selection) = linked_brush {
            let current = engine.state().brush_selection_2d.and_then(|sel| {
                let x_axis = router.axis_key(sel.x_axis)?;
                let y_axis = router.axis_key(sel.y_axis)?;
                Some(BrushSelectionLink2D {
                    x_axis,
                    y_axis,
                    x: sel.x,
                    y: sel.y,
                })
            });
            if selection != current {
                match selection {
                    Some(sel) => {
                        if let (Some(x_axis), Some(y_axis)) = (
                            router.axis_for_key(sel.x_axis),
                            router.axis_for_key(sel.y_axis),
                        ) {
                            engine.apply_action(Action::SetBrushSelection2D {
                                x_axis,
                                y_axis,
                                x: sel.x,
                                y: sel.y,
                            });
                            linked_inputs_changed = true;
                        }
                    }
                    None => {
                        engine.apply_action(Action::ClearBrushSelection);
                        linked_inputs_changed = true;
                    }
                }
            }
        }

        if let Some(windows) = linked_domain_windows {
            for (key, window) in windows {
                let Some(axis) = router.axis_for_key(key) else {
                    continue;
                };

                match key.kind {
                    delinea::AxisKind::X => {
                        let current = engine.state().data_zoom_x.get(&axis).and_then(|s| s.window);
                        if current != window {
                            engine.apply_action(Action::SetDataWindowX { axis, window });
                            linked_inputs_changed = true;
                        }
                    }
                    delinea::AxisKind::Y => {
                        let current = engine.state().data_window_y.get(&axis).copied();
                        if current != window {
                            engine.apply_action(Action::SetDataWindowY { axis, window });
                            linked_inputs_changed = true;
                        }
                    }
                }
            }
        }

        if let Some(anchor) = linked_axis_pointer {
            let current = engine.output().axis_pointer.as_ref().and_then(|o| {
                let axis = router.axis_key(o.axis)?;
                o.axis_value.is_finite().then_some(AxisPointerLinkAnchor {
                    axis,
                    value: o.axis_value,
                })
            });

            if anchor != current {
                match anchor {
                    Some(anchor) => {
                        if let Some(axis) = router.axis_for_key(anchor.axis)
                            && let Some(point) =
                                point_for_axis_pointer_anchor(engine, &router, axis, anchor.value)
                        {
                            engine.apply_action(Action::HoverAt { point });
                            linked_inputs_changed = true;
                        }
                    }
                    None => {
                        engine.apply_action(Action::HoverAt {
                            point: Point::new(Px(1.0e9), Px(1.0e9)),
                        });
                        linked_inputs_changed = true;
                    }
                }
            }
        }

        let mut measurer = NullTextMeasurer;
        let start = Instant::now();
        let mut steps_ran = 0u32;
        let mut still_unfinished = true;
        while still_unfinished && steps_ran < 8 && start.elapsed() < Duration::from_millis(4) {
            let budget = WorkBudget::new(262_144, 0, 32);
            let step = engine.step(&mut measurer, budget);
            match step {
                Ok(step) => {
                    still_unfinished = step.unfinished;
                }
                Err(_) => {
                    still_unfinished = false;
                }
            }
            steps_ran = steps_ran.saturating_add(1);
        }

        unfinished = still_unfinished;

        let output = engine.output();
        output_rev = output.revision;
        marks_rev = output.marks.revision;

        if marks_rev != prev_marks_rev {
            output_marks = Some(Arc::new(output.marks.clone()));
        }

        let model = engine.model();
        series_visible_in_panel.clear();
        for series_id in &model.series_order {
            if series_is_in_panel_grid(mode, model, *series_id) {
                series_visible_in_panel.insert(*series_id);
            }
        }
        series_rank_by_id.clear();
        legend_series = model
            .series_in_order()
            .filter(|s| series_is_in_panel_grid(mode, model, s.id))
            .enumerate()
            .map(|(order, s)| LegendSeriesEntry {
                id: s.id,
                order,
                label: s
                    .name
                    .clone()
                    .unwrap_or_else(|| format!("Series {}", s.id.0))
                    .into(),
                visible: s.visible,
            })
            .collect();
        for s in &legend_series {
            series_rank_by_id.insert(s.id, s.order);
        }

        axis_pointer_output = output.axis_pointer.clone();
        axis_pointer_labels.clear();
        tooltip_lines.clear();
        if let Some(axis_pointer) = axis_pointer_output.as_ref() {
            tooltip_lines =
                tooltip_formatter_c.format_axis_pointer(engine, &output.axis_windows, axis_pointer);

            if let Some(pointer_model) = model.axis_pointer.as_ref()
                && pointer_model.label.show
            {
                let default_tooltip_spec = delinea::TooltipSpecV1::default();
                let tooltip_spec = model.tooltip.as_ref().unwrap_or(&default_tooltip_spec);
                let template = pointer_model.label.template.as_str();

                let mut push_label_for_axis =
                    |axis_id: delinea::AxisId, axis_kind: delinea::AxisKind, axis_value: f64| {
                        let axis_window = output
                            .axis_windows
                            .get(&axis_id)
                            .copied()
                            .unwrap_or_default();
                        let axis_name = model
                            .axes
                            .get(&axis_id)
                            .and_then(|a| a.name.as_deref())
                            .unwrap_or("");
                        let value_text = if axis_value.is_finite() {
                            delinea::engine::axis::format_value_for(
                                model,
                                axis_id,
                                axis_window,
                                axis_value,
                            )
                        } else {
                            tooltip_spec.missing_value.clone()
                        };
                        let label_text = if template == "{value}" {
                            value_text
                        } else {
                            template
                                .replace("{value}", &value_text)
                                .replace("{axis_name}", axis_name)
                        };
                        axis_pointer_labels.push(AxisPointerLabelOverlay {
                            axis_kind,
                            text: label_text.into(),
                        });
                    };

                match &axis_pointer.tooltip {
                    delinea::TooltipOutput::Axis(axis) => {
                        push_label_for_axis(axis.axis, axis.axis_kind, axis.axis_value);
                    }
                    delinea::TooltipOutput::Item(item) => {
                        push_label_for_axis(item.x_axis, delinea::AxisKind::X, item.x_value);
                        push_label_for_axis(item.y_axis, delinea::AxisKind::Y, item.y_value);
                    }
                }
            }
        }

        if output_rev != prev_output_rev {
            hover_point_px = output.hover.map(|hit| hit.point_px);

            if let Some(axis_pointer_out) = output.axis_pointer.as_ref() {
                let (draw_x, draw_y) = match &axis_pointer_out.tooltip {
                    TooltipOutput::Axis(axis) => match axis.axis_kind {
                        delinea::AxisKind::X => (true, false),
                        delinea::AxisKind::Y => (false, true),
                    },
                    TooltipOutput::Item(_) => (true, true),
                };

                axis_pointer = Some(AxisPointerPaintData {
                    crosshair_px: axis_pointer_out.crosshair_px,
                    shadow_rect_px: axis_pointer_out.shadow_rect_px,
                    draw_x,
                    draw_y,
                });
            } else {
                axis_pointer = None;
            }
        }

        if let Some(output_model) = output_model.as_ref() {
            let _ = output_model;
            let mut router = ChartLinkRouter::from_model(engine.model());
            if !explicit_link_axis_map.is_empty() {
                let explicit = explicit_link_axis_map
                    .iter()
                    .filter_map(|(axis, key)| {
                        engine
                            .model()
                            .axes
                            .contains_key(axis)
                            .then_some((*axis, *key))
                    })
                    .collect();
                router = router.with_explicit_axis_map(explicit);
            }

            let drained_link_events = engine.drain_link_events();
            let mut next_output = output_state_before.clone().unwrap_or_default();
            let (link_events_revision, link_events) =
                chart_canvas_output_link_events_batch(&next_output, drained_link_events);
            let snapshot = chart_canvas_output_snapshot_for_engine(
                engine,
                &router,
                link_events,
                &*tooltip_formatter_c,
            );

            if update_chart_canvas_output(&mut next_output, snapshot, link_events_revision) {
                next_published_output = Some(next_output);
            }
        }
    });

    if linked_inputs_changed {
        cx.request_animation_frame();
    }

    if let Some(next_output) = next_published_output {
        cx.state_for(
            output_state_slot,
            ChartCanvasPanelOutputState::default,
            |state| {
                state.output = next_output.clone();
            },
        );
        if let Some(output_model) = output_model.as_ref() {
            let _ = output_model.update(cx.app, |output, _cx| {
                *output = next_output;
            });
        }
    }

    if let Ok(mut st) = legend_state.lock() {
        st.sync_series(legend_series);
    }
    if let Ok(mut st) = tooltip_state.lock() {
        st.axis_pointer = axis_pointer_output;
        st.axis_pointer_labels = std::mem::take(&mut axis_pointer_labels);
        st.lines = tooltip_lines;
        st.series_rank_by_id = series_rank_by_id.clone();
    }

    if props.accessibility_layer {
        let marks_for_a11y = output_marks.clone();
        let _ = a11y_state_model.update(cx.app, |state, _cx| {
            if state.marks_rev != marks_rev
                && let Some(marks) = marks_for_a11y.as_ref()
            {
                state.index.rebuild(marks, &series_rank_by_id);
                state.marks_rev = marks_rev;
                state.series_rank_by_id = series_rank_by_id.clone();
            } else if state.series_rank_by_id != series_rank_by_id {
                state.series_rank_by_id = series_rank_by_id.clone();
            }
        });

        let a11y_state = a11y_state_model
            .read(cx.app, |_app, state| state.clone())
            .unwrap_or_default();
        let formatter_for_semantics = tooltip_formatter.clone();
        let next_semantics = engine
            .update(cx.app, |engine, _cx| {
                a11y_semantics_for_engine(engine, &a11y_state, &*formatter_for_semantics)
            })
            .unwrap_or_default();
        cx.state_for(
            a11y_semantics_state_slot,
            ChartA11ySemanticsState::default,
            |state| {
                *state = next_semantics;
            },
        );
    }

    let (cache, axis_pointer, hover_point_px) =
        cx.state_for(marks_cache_slot, MarksCache::default, |cache| {
            if cache.marks_rev != marks_rev
                && let Some(marks) = output_marks.clone()
            {
                cache.marks_rev = marks_rev;
                cache.marks = marks;
            }

            if cache.output_rev != output_rev {
                cache.output_rev = output_rev;
                cache.axis_pointer = axis_pointer;
                cache.hover_point_px = hover_point_px;
            }

            (
                cache.marks.clone(),
                cache.axis_pointer,
                cache.hover_point_px,
            )
        });

    let style = props.style;
    let engine_c = engine.clone();
    let input_map = props.input_map;

    let pan_drag_down = pan_drag.clone();
    let on_pan_down: OnCanvasToolPointerDown = Arc::new(move |host, _action_cx, tool_cx, down| {
        if !input_map.pan.matches(down.button, down.modifiers) {
            return CanvasToolDownResult::unhandled();
        }
        if !tool_cx.bounds.contains(down.position) {
            return CanvasToolDownResult::unhandled();
        }

        let Some((x_axis, y_axis)) = host
            .models_mut()
            .read(&engine_c, primary_axes)
            .ok()
            .flatten()
        else {
            return CanvasToolDownResult::unhandled();
        };

        let (start_x, start_y) = host
            .models_mut()
            .read(&engine_c, |engine| {
                (
                    window_for_axis_x(engine, x_axis),
                    window_for_axis_y(engine, y_axis),
                )
            })
            .ok()
            .unwrap_or((fallback_window(), fallback_window()));

        let _ = host.models_mut().update(&pan_drag_down, |st| {
            *st = Some(ChartPanDrag {
                start_pos: down.position,
                x_axis,
                y_axis,
                start_x,
                start_y,
            });
        });

        CanvasToolDownResult::activate_and_capture()
    });

    let pan_drag_move = pan_drag.clone();
    let engine_c = engine.clone();
    let on_pan_move: OnCanvasToolPointerMove = Arc::new(move |host, action_cx, tool_cx, mv| {
        let Some(drag) = host
            .models_mut()
            .read(&pan_drag_move, |st| *st)
            .ok()
            .flatten()
        else {
            return false;
        };

        let width = tool_cx.bounds.size.width.0;
        let height = tool_cx.bounds.size.height.0;
        if width <= 0.0 || height <= 0.0 {
            return false;
        }

        let dx = mv.position.x.0 - drag.start_pos.x.0;
        let dy = mv.position.y.0 - drag.start_pos.y.0;

        let _ = host.models_mut().update(&engine_c, |engine| {
            engine.apply_action(Action::PanDataWindowXFromBase {
                axis: drag.x_axis,
                base: drag.start_x,
                delta_px: dx,
                viewport_span_px: width,
            });
            engine.apply_action(Action::PanDataWindowYFromBase {
                axis: drag.y_axis,
                base: drag.start_y,
                delta_px: -dy,
                viewport_span_px: height,
            });
        });

        host.request_redraw(action_cx.window);
        true
    });

    let pan_drag_up = pan_drag.clone();
    let on_pan_up: OnCanvasToolPointerUp = Arc::new(move |host, _action_cx, _tool_cx, _up| {
        let _ = host.models_mut().update(&pan_drag_up, |st| *st = None);
        true
    });

    let engine_c = engine.clone();
    let on_hover_move: OnCanvasToolPointerMove = Arc::new(move |host, action_cx, _tool_cx, mv| {
        let _ = host.models_mut().update(&engine_c, |engine| {
            engine.apply_action(Action::HoverAt { point: mv.position });
        });
        host.request_redraw(action_cx.window);
        true
    });

    let engine_c = engine.clone();
    let input_map_c = input_map;
    let on_wheel_zoom: OnCanvasToolWheel = Arc::new(move |host, action_cx, tool_cx, wheel| {
        let delta_y = wheel.delta.y.0;
        if !delta_y.is_finite() {
            return false;
        }

        if let Some(required) = input_map_c.wheel_zoom_mod
            && !required.is_pressed(wheel.modifiers)
        {
            return false;
        }

        let width = tool_cx.bounds.size.width.0;
        let height = tool_cx.bounds.size.height.0;
        if width <= 0.0 || height <= 0.0 {
            return false;
        }

        let Some((x_axis, y_axis)) = host
            .models_mut()
            .read(&engine_c, primary_axes)
            .ok()
            .flatten()
        else {
            return false;
        };

        let (base_x, base_y) = host
            .models_mut()
            .read(&engine_c, |engine| {
                (
                    window_for_axis_x(engine, x_axis),
                    window_for_axis_y(engine, y_axis),
                )
            })
            .ok()
            .unwrap_or((fallback_window(), fallback_window()));

        // Match ImPlot's default feel: zoom factor ~= 2^(delta_y * 0.0025)
        let log2_scale = delta_y * 0.0025;

        let local_x = (wheel.position.x.0 - tool_cx.bounds.origin.x.0).clamp(0.0, width);
        let local_y = (wheel.position.y.0 - tool_cx.bounds.origin.y.0).clamp(0.0, height);
        let center_x = local_x;
        let center_y_from_bottom = height - local_y;

        let _ = host.models_mut().update(&engine_c, |engine| {
            engine.apply_action(Action::ZoomDataWindowXFromBase {
                axis: x_axis,
                base: base_x,
                center_px: center_x,
                log2_scale,
                viewport_span_px: width,
            });
            engine.apply_action(Action::ZoomDataWindowYFromBase {
                axis: y_axis,
                base: base_y,
                center_px: center_y_from_bottom,
                log2_scale,
                viewport_span_px: height,
            });
        });

        host.request_redraw(action_cx.window);
        true
    });

    let engine_c = engine.clone();
    let on_pinch_zoom: OnCanvasToolPinch = Arc::new(move |host, action_cx, tool_cx, pinch| {
        if !pinch.delta.is_finite() {
            return false;
        }

        let width = tool_cx.bounds.size.width.0;
        let height = tool_cx.bounds.size.height.0;
        if width <= 0.0 || height <= 0.0 {
            return false;
        }

        let Some((x_axis, y_axis)) = host
            .models_mut()
            .read(&engine_c, primary_axes)
            .ok()
            .flatten()
        else {
            return false;
        };

        let (base_x, base_y) = host
            .models_mut()
            .read(&engine_c, |engine| {
                (
                    window_for_axis_x(engine, x_axis),
                    window_for_axis_y(engine, y_axis),
                )
            })
            .ok()
            .unwrap_or((fallback_window(), fallback_window()));

        // Match `fret-ui-kit`'s pinch mapping: factor = 1 + delta.
        let delta = pinch.delta.clamp(-0.95, 10.0);
        let factor = (1.0 + delta).max(0.01);
        let log2_scale = factor.log2();
        if !log2_scale.is_finite() || log2_scale.abs() <= 1.0e-9 {
            return false;
        }

        let local_x = (pinch.position.x.0 - tool_cx.bounds.origin.x.0).clamp(0.0, width);
        let local_y = (pinch.position.y.0 - tool_cx.bounds.origin.y.0).clamp(0.0, height);
        let center_x = local_x;
        let center_y_from_bottom = height - local_y;

        let _ = host.models_mut().update(&engine_c, |engine| {
            engine.apply_action(Action::ZoomDataWindowXFromBase {
                axis: x_axis,
                base: base_x,
                center_px: center_x,
                log2_scale,
                viewport_span_px: width,
            });
            engine.apply_action(Action::ZoomDataWindowYFromBase {
                axis: y_axis,
                base: base_y,
                center_px: center_y_from_bottom,
                log2_scale,
                viewport_span_px: height,
            });
        });

        host.request_redraw(action_cx.window);
        true
    });

    let mut tools = Vec::new();
    if mode.renders_overlays() {
        tools.push(legend_overlay_tool(
            engine.clone(),
            legend_state.clone(),
            style,
        ));
        tools.push(tooltip_overlay_tool(tooltip_state.clone(), style));
    }
    if mode.renders_marks() {
        tools.push(CanvasToolEntry {
            id: CanvasToolId::new(1),
            priority: 100,
            handlers: CanvasToolHandlers {
                on_pointer_down: Some(on_pan_down),
                on_pointer_move: Some(on_pan_move),
                on_pointer_up: Some(on_pan_up),
                ..Default::default()
            },
        });
        tools.push(CanvasToolEntry {
            id: CanvasToolId::new(2),
            priority: 50,
            handlers: CanvasToolHandlers {
                on_wheel: Some(on_wheel_zoom),
                ..Default::default()
            },
        });
        tools.push(CanvasToolEntry {
            id: CanvasToolId::new(4),
            priority: 50,
            handlers: CanvasToolHandlers {
                on_pinch: Some(on_pinch_zoom),
                ..Default::default()
            },
        });
        tools.push(CanvasToolEntry {
            id: CanvasToolId::new(3),
            priority: -10,
            handlers: CanvasToolHandlers {
                on_pointer_move: Some(on_hover_move),
                ..Default::default()
            },
        });
    }

    let mut pan_zoom = PanZoomCanvasSurfacePanelProps::default();
    pan_zoom.pointer_region = props.pointer_region;
    pan_zoom.canvas = props.canvas;

    // Disable built-in infinite-canvas pan/zoom: chart interactions are routed via tools.
    pan_zoom.pan_button = MouseButton::Other(999);
    pan_zoom.min_zoom = 1.0;
    pan_zoom.max_zoom = 1.0;

    let router_props = CanvasToolRouterProps {
        pan_zoom,
        active_tool: None,
    };

    let marks = cache;
    let series_visible_in_panel = Arc::new(series_visible_in_panel);
    let paint = move |painter: &mut CanvasPainter<'_>, paint_cx: PanZoomCanvasPaintCx| {
        if unfinished {
            painter.request_animation_frame();
        }

        let bounds = painter.bounds();

        // Basic background.
        if mode.renders_marks()
            && let Some(background) = style.background
        {
            painter.scene().push(fret_core::SceneOp::Quad {
                order: DrawOrder(style.draw_order.0.saturating_sub(1)),
                rect: bounds,
                background: fret_core::Paint::Solid(background).into(),
                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT.into(),

                corner_radii: Corners::all(Px(0.0)),
            });
        }

        let viewport = bounds;
        painter.with_clip_rect(viewport, |painter| {
            let marks = &*marks;
            let arena = &marks.arena;

            if mode.renders_marks() {
                for node in &marks.nodes {
                    if let Some(series_id) = node.source_series
                        && panel_grid.is_some()
                        && !series_visible_in_panel.contains(&series_id)
                    {
                        continue;
                    }

                    match (node.kind, &node.payload) {
                        (MarkKind::Polyline, MarkPayloadRef::Polyline(poly)) => {
                            let start = poly.points.start;
                            let end = poly.points.end;
                            if end <= start || end > arena.points.len() {
                                continue;
                            }

                            let mut commands: Vec<PathCommand> =
                                Vec::with_capacity((end - start).saturating_add(1));
                            for (i, p) in arena.points[start..end].iter().enumerate() {
                                if i == 0 {
                                    commands.push(PathCommand::MoveTo(*p));
                                } else {
                                    commands.push(PathCommand::LineTo(*p));
                                }
                            }
                            if commands.len() < 2 {
                                continue;
                            }

                            let stroke_width = poly
                                .stroke
                                .as_ref()
                                .map(|(_, s)| s.width)
                                .unwrap_or(style.stroke_width);
                            let stroke_color = if let Some((paint, _)) = &poly.stroke {
                                paint_color(style, *paint)
                            } else if let Some(series) = node.source_series {
                                series_color(style, series)
                            } else {
                                style.stroke_color
                            };

                            let key = node.id.0;
                            painter.path(
                                key,
                                DrawOrder(style.draw_order.0.saturating_add(node.order.0)),
                                Point::new(Px(0.0), Px(0.0)),
                                &commands,
                                PathStyle::Stroke(StrokeStyle {
                                    width: stroke_width,
                                }),
                                stroke_color,
                                paint_cx.raster_scale_factor,
                            );
                        }
                        (MarkKind::Rect, MarkPayloadRef::Rect(rects)) => {
                            let start = rects.rects.start;
                            let end = rects.rects.end;
                            if end <= start || end > arena.rects.len() {
                                continue;
                            }

                            let stroke_width = rects
                                .stroke
                                .as_ref()
                                .map(|(_, s)| s.width)
                                .filter(|w| w.0.is_finite() && w.0 > 0.0)
                                .unwrap_or(Px(0.0));

                            for rect in &arena.rects[start..end] {
                                let mut background = Color::TRANSPARENT;
                                if let Some(paint) = rects.fill {
                                    background = paint_color(style, paint);
                                } else if let Some(series) = node.source_series {
                                    background = series_color(style, series);
                                }
                                background.a *= rects.opacity_mul.unwrap_or(1.0);

                                let border_color = if stroke_width.0 > 0.0 {
                                    background
                                } else {
                                    Color::TRANSPARENT
                                };

                                painter.scene().push(fret_core::SceneOp::Quad {
                                    order: DrawOrder(
                                        style.draw_order.0.saturating_add(node.order.0),
                                    ),
                                    rect: *rect,
                                    background: fret_core::Paint::Solid(background).into(),
                                    border: Edges::all(stroke_width),
                                    border_paint: fret_core::Paint::Solid(border_color).into(),
                                    corner_radii: Corners::all(Px(0.0)),
                                });
                            }
                        }
                        (MarkKind::Points, MarkPayloadRef::Points(points)) => {
                            let start = points.points.start;
                            let end = points.points.end;
                            if end <= start || end > arena.points.len() {
                                continue;
                            }

                            let base_point_r = style.scatter_point_radius.0.max(1.0);
                            let stroke_width = points
                                .stroke
                                .as_ref()
                                .map(|(_, s)| s.width)
                                .filter(|w| w.0.is_finite() && w.0 > 0.0)
                                .unwrap_or(Px(0.0));

                            for p in &arena.points[start..end] {
                                let radius_mul = points
                                    .radius_mul
                                    .filter(|v| v.is_finite() && *v > 0.0)
                                    .unwrap_or(1.0);
                                let point_r = (base_point_r * radius_mul).max(1.0);

                                let mut fill = style.stroke_color;
                                if let Some(paint) = points.fill {
                                    fill = paint_color(style, paint);
                                    fill.a *= style.scatter_fill_alpha;
                                } else if let Some(series) = node.source_series {
                                    fill = series_color(style, series);
                                    fill.a *= style.scatter_fill_alpha;
                                }
                                fill.a *= points.opacity_mul.unwrap_or(1.0);

                                let border_color = if stroke_width.0 > 0.0 {
                                    fill
                                } else {
                                    Color::TRANSPARENT
                                };

                                painter.scene().push(fret_core::SceneOp::Quad {
                                    order: DrawOrder(
                                        style.draw_order.0.saturating_add(node.order.0),
                                    ),
                                    rect: Rect::new(
                                        Point::new(Px(p.x.0 - point_r), Px(p.y.0 - point_r)),
                                        Size::new(Px(2.0 * point_r), Px(2.0 * point_r)),
                                    ),
                                    background: fret_core::Paint::Solid(fill).into(),

                                    border: Edges::all(stroke_width),
                                    border_paint: fret_core::Paint::Solid(border_color).into(),
                                    corner_radii: Corners::all(Px(point_r)),
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }

            if !mode.renders_overlays() {
                return;
            }

            let overlay_order = DrawOrder(style.draw_order.0.saturating_add(10_000));
            let shadow_order = DrawOrder(style.draw_order.0.saturating_add(9_900));

            if let Some(axis_pointer) = axis_pointer {
                if let Some(rect) = axis_pointer.shadow_rect_px {
                    let color = Color {
                        a: 0.08,
                        ..style.selection_fill
                    };
                    painter.scene().push(fret_core::SceneOp::Quad {
                        order: shadow_order,
                        rect,
                        background: fret_core::Paint::Solid(color).into(),

                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT.into(),

                        corner_radii: Corners::all(Px(0.0)),
                    });
                } else {
                    let plot = bounds;
                    let crosshair_w = style.crosshair_width.0.max(1.0);
                    let x = axis_pointer
                        .crosshair_px
                        .x
                        .0
                        .clamp(plot.origin.x.0, plot.origin.x.0 + plot.size.width.0);
                    let y = axis_pointer
                        .crosshair_px
                        .y
                        .0
                        .clamp(plot.origin.y.0, plot.origin.y.0 + plot.size.height.0);

                    if axis_pointer.draw_x {
                        painter.scene().push(fret_core::SceneOp::Quad {
                            order: overlay_order,
                            rect: Rect::new(
                                Point::new(Px(x - 0.5 * crosshair_w), plot.origin.y),
                                Size::new(Px(crosshair_w), plot.size.height),
                            ),
                            background: fret_core::Paint::Solid(style.crosshair_color).into(),

                            border: Edges::all(Px(0.0)),
                            border_paint: fret_core::Paint::TRANSPARENT.into(),

                            corner_radii: Corners::all(Px(0.0)),
                        });
                    }
                    if axis_pointer.draw_y {
                        painter.scene().push(fret_core::SceneOp::Quad {
                            order: overlay_order,
                            rect: Rect::new(
                                Point::new(plot.origin.x, Px(y - 0.5 * crosshair_w)),
                                Size::new(plot.size.width, Px(crosshair_w)),
                            ),
                            background: fret_core::Paint::Solid(style.crosshair_color).into(),

                            border: Edges::all(Px(0.0)),
                            border_paint: fret_core::Paint::TRANSPARENT.into(),

                            corner_radii: Corners::all(Px(0.0)),
                        });
                    }
                }
            }

            if let Some(point) = hover_point_px {
                let size = style.hover_point_size.0.max(1.0);
                let r = 0.5 * size;
                painter.scene().push(fret_core::SceneOp::Quad {
                    order: overlay_order,
                    rect: Rect::new(
                        Point::new(Px(point.x.0 - r), Px(point.y.0 - r)),
                        Size::new(Px(size), Px(size)),
                    ),
                    background: fret_core::Paint::Solid(style.hover_point_color).into(),

                    border: Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT.into(),

                    corner_radii: Corners::all(Px(r)),
                });
            }
        });
    };

    let engine_k = engine.clone();
    let engine_a11y_k = engine.clone();
    let a11y_state_k = a11y_state_model.clone();
    let legend_state_k = legend_state.clone();
    let accessibility_layer = props.accessibility_layer;
    let bounds_for_a11y = bounds;
    let on_key_down: OnKeyDown = Arc::new(move |host, action_cx, down| {
        if accessibility_layer
            && !down.repeat
            && !down.modifiers.shift
            && !down.modifiers.ctrl
            && !down.modifiers.alt
            && !down.modifiers.alt_gr
            && !down.modifiers.meta
        {
            let mut a11y_state = host
                .models_mut()
                .read(&a11y_state_k, |state| state.clone())
                .ok()
                .unwrap_or_default();
            let handled = host
                .models_mut()
                .update(&engine_a11y_k, |engine| {
                    handle_a11y_navigation(engine, &mut a11y_state, bounds_for_a11y, down.key)
                })
                .ok()
                .unwrap_or(false);
            if handled {
                let _ = host.models_mut().update(&a11y_state_k, |state| {
                    *state = a11y_state;
                });
                host.request_redraw(action_cx.window);
                return true;
            }
        }

        let modifiers = down.modifiers;
        let legend_mods_ok =
            modifiers.ctrl && !modifiers.alt && !modifiers.alt_gr && !modifiers.meta;
        if !legend_mods_ok {
            return false;
        }

        let in_legend = legend_state_k
            .lock()
            .ok()
            .is_some_and(|st| st.is_pointer_in_panel());
        if !in_legend {
            return false;
        }

        let changed = host
            .models_mut()
            .update(&engine_k, |engine| {
                let model = engine.model();
                let updates = match down.key {
                    KeyCode::KeyA if modifiers.shift => {
                        crate::legend_logic::legend_select_none_updates(model)
                    }
                    KeyCode::KeyA => crate::legend_logic::legend_select_all_updates(model),
                    KeyCode::KeyI if !modifiers.shift => {
                        crate::legend_logic::legend_invert_updates(model)
                    }
                    _ => return false,
                };
                if updates.is_empty() {
                    return false;
                }
                engine.apply_action(Action::SetSeriesVisibility { updates });
                true
            })
            .ok()
            .unwrap_or(false);

        if !changed {
            return false;
        }
        if let Ok(mut st) = legend_state_k.lock() {
            st.anchor = None;
        }
        host.request_redraw(action_cx.window);
        true
    });

    let focus_props = FocusScopeProps::default();
    let on_key_down_focus = on_key_down.clone();
    let inner = cx.focus_scope_with_id(focus_props, move |cx, focus_id| {
        cx.key_add_on_key_down_for(focus_id, on_key_down_focus.clone());
        vec![canvas_tool_router_panel(cx, router_props, tools, paint)]
    });
    let inner = if matches!(props.mode, ChartCanvasPanelMode::Overlay) {
        let legend_state_for_hit_test = legend_state.clone();
        let mut surface_props = ManagedSurfaceProps::default();
        surface_props.layout.size.width = Length::Fill;
        surface_props.layout.size.height = Length::Fill;
        cx.managed_surface(
            surface_props,
            move |cx| {
                let bounds = cx.bounds();
                for child in cx.children().to_vec() {
                    let _ = cx.layout_child(child, bounds);
                }
                let hit_rects = legend_state_for_hit_test
                    .lock()
                    .ok()
                    .and_then(|state| state.panel_rect())
                    .into_iter();
                cx.set_hit_test_rects(hit_rects);
            },
            move |cx| {
                for child in cx.children().to_vec() {
                    if let Some(bounds) = cx.child_bounds(child) {
                        cx.paint_child(child, bounds);
                    }
                }
            },
            |_cx| vec![inner],
        )
    } else {
        inner
    };
    let test_id = props
        .test_id
        .clone()
        .or_else(|| props.mode.default_test_id());

    if props.accessibility_layer {
        let semantics_state = cx.state_for(
            a11y_semantics_state_slot,
            ChartA11ySemanticsState::default,
            |state| state.clone(),
        );
        let mut semantics = SemanticsProps {
            role: SemanticsRole::Viewport,
            label: Some(Arc::from("Chart")),
            test_id: test_id.clone(),
            pos_in_set: semantics_state.pos_in_set,
            set_size: semantics_state.set_size,
            value: semantics_state.value,
            focusable: true,
            ..Default::default()
        };
        semantics.layout.size.width = Length::Fill;
        semantics.layout.size.height = Length::Fill;
        cx.semantics_with_id(semantics, move |cx, semantics_id| {
            cx.key_add_on_key_down_for(semantics_id, on_key_down);
            vec![inner]
        })
    } else if let Some(test_id) = test_id {
        inner.test_id(test_id)
    } else {
        inner
    }
}

/// Capability-first adapter for [`chart_canvas_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn chart_canvas_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: ChartCanvasPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    chart_canvas_panel(cx.elements(), props)
}

#[cfg(test)]
mod tests {
    use super::*;
    use delinea::data::{Column, DataTable};
    use delinea::ids::{AxisId, ChartId, DatasetId, FieldId, GridId, SeriesId};
    use delinea::{
        AxisKind, AxisScale, ChartSpec, DatasetSpec, FieldSpec, GridSpec, SeriesEncode, SeriesKind,
        SeriesSpec,
    };
    use fret_app::App;
    use fret_core::{
        AppWindowId, Event, Modifiers, PathConstraints, PathId, PathMetrics, Scene, SceneOp,
        TextBlobId, TextConstraints, TextMetrics,
    };
    use fret_runtime::{FrameId, Model};
    use fret_ui::declarative::render_root;
    use fret_ui::tree::UiTree;

    fn chart_spec() -> (ChartSpec, DatasetId, Vec<f64>, Vec<f64>) {
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
        let x_field = FieldId::new(1);
        let y_field = FieldId::new(2);
        let series_id = SeriesId::new(1);

        let spec = ChartSpec {
            id: ChartId::new(1),
            viewport: None,
            datasets: vec![DatasetSpec {
                id: dataset_id,
                fields: vec![
                    FieldSpec {
                        id: x_field,
                        column: 0,
                    },
                    FieldSpec {
                        id: y_field,
                        column: 1,
                    },
                ],
                ..Default::default()
            }],
            grids: vec![GridSpec { id: grid_id }],
            axes: vec![
                delinea::AxisSpec {
                    id: x_axis,
                    name: Some("Month".to_string()),
                    kind: AxisKind::X,
                    grid: grid_id,
                    position: None,
                    scale: AxisScale::Category(delinea::CategoryAxisScale {
                        categories: vec![
                            "Jan".to_string(),
                            "Feb".to_string(),
                            "Mar".to_string(),
                            "Apr".to_string(),
                        ],
                    }),
                    range: None,
                },
                delinea::AxisSpec {
                    id: y_axis,
                    name: Some("Visitors".to_string()),
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: None,
                    scale: Default::default(),
                    range: None,
                },
            ],
            data_zoom_x: vec![],
            data_zoom_y: vec![],
            tooltip: None,
            axis_pointer: Some(delinea::AxisPointerSpec::default()),
            visual_maps: vec![],
            series: vec![SeriesSpec {
                id: series_id,
                name: Some("Desktop".to_string()),
                kind: SeriesKind::Bar,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_field,
                    y2: None,
                },
                x_axis,
                y_axis,
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            }],
        };

        (
            spec,
            dataset_id,
            vec![0.0, 1.0, 2.0, 3.0],
            vec![186.0, 305.0, 237.0, 73.0],
        )
    }

    fn seed_dataset(engine: &mut ChartEngine, dataset_id: DatasetId, x: Vec<f64>, y: Vec<f64>) {
        let mut table = DataTable::default();
        table.push_column(Column::F64(x));
        table.push_column(Column::F64(y));
        engine.datasets_mut().insert(dataset_id, table);
    }

    fn line_scatter_chart_spec() -> (ChartSpec, DatasetId, Vec<f64>, Vec<f64>, Vec<f64>) {
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
        let x_field = FieldId::new(1);
        let y_line_field = FieldId::new(2);
        let y_scatter_field = FieldId::new(3);
        let line_series = SeriesId::new(1);
        let scatter_series = SeriesId::new(2);

        let spec = ChartSpec {
            id: ChartId::new(1),
            viewport: None,
            datasets: vec![DatasetSpec {
                id: dataset_id,
                fields: vec![
                    FieldSpec {
                        id: x_field,
                        column: 0,
                    },
                    FieldSpec {
                        id: y_line_field,
                        column: 1,
                    },
                    FieldSpec {
                        id: y_scatter_field,
                        column: 2,
                    },
                ],
                ..Default::default()
            }],
            grids: vec![GridSpec { id: grid_id }],
            axes: vec![
                delinea::AxisSpec {
                    id: x_axis,
                    name: Some("X".to_string()),
                    kind: AxisKind::X,
                    grid: grid_id,
                    position: None,
                    scale: Default::default(),
                    range: None,
                },
                delinea::AxisSpec {
                    id: y_axis,
                    name: Some("Y".to_string()),
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: None,
                    scale: Default::default(),
                    range: None,
                },
            ],
            data_zoom_x: vec![],
            data_zoom_y: vec![],
            tooltip: None,
            axis_pointer: None,
            visual_maps: vec![],
            series: vec![
                SeriesSpec {
                    id: line_series,
                    name: Some("Line".to_string()),
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_line_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
                SeriesSpec {
                    id: scatter_series,
                    name: Some("Scatter".to_string()),
                    kind: SeriesKind::Scatter,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_scatter_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
            ],
        };

        (
            spec,
            dataset_id,
            vec![0.0, 1.0, 2.0, 3.0, 4.0],
            vec![0.0, 2.0, 1.0, 3.0, 2.0],
            vec![1.0, 1.5, 0.5, 2.5, 1.25],
        )
    }

    fn multi_axis_spec() -> ChartSpec {
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_left = AxisId::new(2);
        let y_right = AxisId::new(3);
        let x_field = FieldId::new(1);
        let y_field = FieldId::new(2);

        ChartSpec {
            id: ChartId::new(1),
            viewport: None,
            datasets: vec![DatasetSpec {
                id: dataset_id,
                fields: vec![
                    FieldSpec {
                        id: x_field,
                        column: 0,
                    },
                    FieldSpec {
                        id: y_field,
                        column: 1,
                    },
                ],
                ..Default::default()
            }],
            grids: vec![GridSpec { id: grid_id }],
            axes: vec![
                delinea::AxisSpec {
                    id: x_axis,
                    name: None,
                    kind: AxisKind::X,
                    grid: grid_id,
                    position: None,
                    scale: AxisScale::default(),
                    range: None,
                },
                delinea::AxisSpec {
                    id: y_left,
                    name: None,
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: None,
                    scale: AxisScale::default(),
                    range: None,
                },
                delinea::AxisSpec {
                    id: y_right,
                    name: None,
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: None,
                    scale: AxisScale::default(),
                    range: None,
                },
            ],
            data_zoom_x: vec![],
            data_zoom_y: vec![],
            tooltip: None,
            axis_pointer: None,
            visual_maps: vec![],
            series: vec![
                SeriesSpec {
                    id: SeriesId::new(1),
                    name: None,
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis: y_left,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
                SeriesSpec {
                    id: SeriesId::new(2),
                    name: None,
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis: y_right,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
            ],
        }
    }

    fn multi_grid_spec() -> (ChartSpec, DatasetId) {
        let dataset_id = DatasetId::new(1);
        let grid_1 = GridId::new(1);
        let grid_2 = GridId::new(2);
        let x_axis_1 = AxisId::new(1);
        let y_axis_1 = AxisId::new(2);
        let x_axis_2 = AxisId::new(3);
        let y_axis_2 = AxisId::new(4);
        let x_field = FieldId::new(1);
        let y_grid_1 = FieldId::new(2);
        let y_grid_2 = FieldId::new(3);

        let spec = ChartSpec {
            id: ChartId::new(1),
            viewport: None,
            datasets: vec![DatasetSpec {
                id: dataset_id,
                fields: vec![
                    FieldSpec {
                        id: x_field,
                        column: 0,
                    },
                    FieldSpec {
                        id: y_grid_1,
                        column: 1,
                    },
                    FieldSpec {
                        id: y_grid_2,
                        column: 2,
                    },
                ],
                ..Default::default()
            }],
            grids: vec![GridSpec { id: grid_1 }, GridSpec { id: grid_2 }],
            axes: vec![
                delinea::AxisSpec {
                    id: x_axis_1,
                    name: Some("X1".to_string()),
                    kind: AxisKind::X,
                    grid: grid_1,
                    position: None,
                    scale: AxisScale::Category(delinea::CategoryAxisScale {
                        categories: vec![
                            "A".to_string(),
                            "B".to_string(),
                            "C".to_string(),
                            "D".to_string(),
                        ],
                    }),
                    range: None,
                },
                delinea::AxisSpec {
                    id: y_axis_1,
                    name: Some("Y1".to_string()),
                    kind: AxisKind::Y,
                    grid: grid_1,
                    position: None,
                    scale: Default::default(),
                    range: None,
                },
                delinea::AxisSpec {
                    id: x_axis_2,
                    name: Some("X2".to_string()),
                    kind: AxisKind::X,
                    grid: grid_2,
                    position: None,
                    scale: AxisScale::Category(delinea::CategoryAxisScale {
                        categories: vec![
                            "A".to_string(),
                            "B".to_string(),
                            "C".to_string(),
                            "D".to_string(),
                        ],
                    }),
                    range: None,
                },
                delinea::AxisSpec {
                    id: y_axis_2,
                    name: Some("Y2".to_string()),
                    kind: AxisKind::Y,
                    grid: grid_2,
                    position: None,
                    scale: Default::default(),
                    range: None,
                },
            ],
            data_zoom_x: vec![],
            data_zoom_y: vec![],
            tooltip: None,
            axis_pointer: Some(delinea::AxisPointerSpec::default()),
            visual_maps: vec![],
            series: vec![
                SeriesSpec {
                    id: SeriesId::new(1),
                    name: Some("Grid 1".to_string()),
                    kind: SeriesKind::Bar,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_grid_1,
                        y2: None,
                    },
                    x_axis: x_axis_1,
                    y_axis: y_axis_1,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
                SeriesSpec {
                    id: SeriesId::new(2),
                    name: Some("Grid 2".to_string()),
                    kind: SeriesKind::Bar,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_grid_2,
                        y2: None,
                    },
                    x_axis: x_axis_2,
                    y_axis: y_axis_2,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
            ],
        };
        (spec, dataset_id)
    }

    fn seed_line_scatter_dataset(
        engine: &mut ChartEngine,
        dataset_id: DatasetId,
        x: Vec<f64>,
        y_line: Vec<f64>,
        y_scatter: Vec<f64>,
    ) {
        let mut table = DataTable::default();
        table.push_column(Column::F64(x));
        table.push_column(Column::F64(y_line));
        table.push_column(Column::F64(y_scatter));
        engine.datasets_mut().insert(dataset_id, table);
    }

    fn seed_multi_grid_dataset(engine: &mut ChartEngine, dataset_id: DatasetId) {
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![0.0, 1.0, 2.0, 3.0]));
        table.push_column(Column::F64(vec![1.0, 2.0, 3.0, 4.0]));
        table.push_column(Column::F64(vec![10.0, 20.0, 30.0, 40.0]));
        engine.datasets_mut().insert(dataset_id, table);
    }

    fn pump_chart_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut FakeServices,
        bounds: Rect,
    ) {
        ui.layout_all(app, services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(app, services, bounds, &mut scene, 1.0);
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[derive(Default)]
    struct FakeServices;

    impl fret_core::TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl fret_core::PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl fret_core::SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            true
        }
    }

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Ok(fret_core::MaterialId::default())
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

    #[test]
    fn chart_canvas_panel_paints_seeded_chart_marks_on_declarative_path() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices;
        let (spec, dataset_id, x, y) = chart_spec();

        let engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("chart spec should be valid"));
        app.models_mut()
            .update(&engine, |engine| seed_dataset(engine, dataset_id, x, y))
            .expect("chart engine model should exist");

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "chart-declarative-canvas-panel",
            |cx| {
                vec![chart_canvas_panel(
                    cx,
                    ChartCanvasPanelProps {
                        engine: Some(engine.clone()),
                        spec: spec.clone(),
                        ..ChartCanvasPanelProps::new(spec.clone())
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let root_bounds = ui.debug_node_bounds(root).expect("root should be laid out");
        assert!(root_bounds.size.width.0 > 0.0);
        assert!(root_bounds.size.height.0 > 0.0);

        let mut stack = ui.debug_node_children(root);
        let mut non_zero_descendants = 0usize;
        while let Some(node) = stack.pop() {
            if let Some(node_bounds) = ui.debug_node_bounds(node)
                && node_bounds.size.width.0 > 0.0
                && node_bounds.size.height.0 > 0.0
            {
                non_zero_descendants = non_zero_descendants.saturating_add(1);
            }
            stack.extend(ui.debug_node_children(node));
        }
        assert!(
            non_zero_descendants > 0,
            "declarative chart panel should lay out a non-zero child surface"
        );

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let has_rect_marks = app
            .models()
            .read(&engine, |engine| {
                engine.output().marks.nodes.iter().any(|node| {
                    matches!(
                        (node.kind, &node.payload),
                        (MarkKind::Rect, MarkPayloadRef::Rect(rects))
                            if rects.rects.end > rects.rects.start
                    )
                })
            })
            .expect("chart engine model should exist");
        assert!(
            has_rect_marks,
            "seeded bar chart should produce rect marks before declarative paint"
        );

        let chart_mark_quads = scene
            .ops()
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    SceneOp::Quad { rect, order, .. }
                        if order.0 >= ChartStyle::default().draw_order.0
                            && rect.size.width.0 > 0.0
                            && rect.size.height.0 > 0.0
                            && *rect != bounds
                )
            })
            .count();
        assert!(
            chart_mark_quads > 0,
            "declarative chart canvas should paint non-zero chart quads"
        );

        let viewport = app
            .models()
            .read(&engine, |engine| engine.model().viewport)
            .expect("chart engine model should exist");
        assert_eq!(viewport, Some(bounds));

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn chart_canvas_panel_publishes_output_model_on_declarative_path() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices;
        let (spec, dataset_id, x, y) = chart_spec();
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
        let x_key = LinkAxisKey {
            kind: AxisKind::X,
            dataset: DatasetId::new(1),
            field: FieldId::new(1),
        };
        let y_key = LinkAxisKey {
            kind: AxisKind::Y,
            dataset: DatasetId::new(1),
            field: FieldId::new(2),
        };
        let x_window = delinea::engine::window::DataWindow { min: 0.0, max: 2.0 };
        let y_window = delinea::engine::window::DataWindow {
            min: 40.0,
            max: 320.0,
        };
        let brush_y = delinea::engine::window::DataWindow {
            min: 50.0,
            max: 260.0,
        };

        let engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("chart spec should be valid"));
        app.models_mut()
            .update(&engine, |engine| {
                seed_dataset(engine, dataset_id, x, y);
                engine.apply_action(Action::SetLinkGroup {
                    group: Some(delinea::LinkGroupId::new(1)),
                });
                engine.apply_action(Action::SetDataWindowX {
                    axis: x_axis,
                    window: Some(x_window),
                });
                engine.apply_action(Action::SetDataWindowY {
                    axis: y_axis,
                    window: Some(y_window),
                });
                engine.apply_action(Action::SetBrushSelection2D {
                    x_axis,
                    y_axis,
                    x: x_window,
                    y: brush_y,
                });
            })
            .expect("chart engine model should exist");
        let output: Model<ChartCanvasOutput> =
            app.models_mut().insert(ChartCanvasOutput::default());

        let mut render_frame = |ui: &mut UiTree<App>, app: &mut App| {
            let output = output.clone();
            let engine = engine.clone();
            let spec = spec.clone();
            let root = render_root(
                ui,
                app,
                &mut services,
                window,
                bounds,
                "chart-declarative-output-panel",
                |cx| {
                    let mut props = ChartCanvasPanelProps::new(spec.clone())
                        .output_model(output)
                        .link_axis_map(BTreeMap::from([(x_axis, x_key), (y_axis, y_key)]));
                    props.engine = Some(engine);
                    vec![chart_canvas_panel(cx, props)]
                },
            );
            ui.set_root(root);
            ui.layout_all(app, &mut services, bounds, 1.0);
            let mut scene = Scene::default();
            ui.paint_all(app, &mut services, bounds, &mut scene, 1.0);
        };

        render_frame(&mut ui, &mut app);
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        render_frame(&mut ui, &mut app);

        let published = output
            .read(&mut app, |_app, output| output.clone())
            .expect("output model should be readable");
        assert!(
            published.revision > 0,
            "declarative chart output publication should advance the output revision"
        );
        assert_eq!(
            published
                .snapshot
                .domain_windows_by_key
                .get(&x_key)
                .copied(),
            Some(Some(x_window)),
            "declarative chart should publish X domain windows in LinkAxisKey space"
        );
        assert_eq!(
            published
                .snapshot
                .domain_windows_by_key
                .get(&y_key)
                .copied(),
            Some(Some(y_window)),
            "declarative chart should publish Y domain windows in LinkAxisKey space"
        );
        assert_eq!(
            published.snapshot.brush_selection_2d.map(|brush| brush.x),
            Some(x_window),
            "declarative chart should publish brush selections"
        );
        assert!(
            published.snapshot.link_events.iter().any(|event| matches!(
                event,
                delinea::LinkEvent::DomainWindowChanged {
                    axis,
                    window: Some(window)
                } if *axis == x_axis && *window == x_window
            )),
            "declarative chart should preserve drained domain-window link events"
        );
        assert!(
            published.snapshot.link_events.iter().any(|event| matches!(
                event,
                delinea::LinkEvent::BrushSelectionChanged { selection: Some(_) }
            )),
            "declarative chart should preserve drained brush link events"
        );
        assert!(
            published.link_events_revision > 0,
            "declarative chart should advance the link events revision when publishing link events"
        );
    }

    #[test]
    fn explicit_y_domain_window_propagates_to_second_declarative_chart_output_model() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(400.0)),
        );
        let mut services = FakeServices;

        let source_output: Model<ChartCanvasOutput> =
            app.models_mut().insert(ChartCanvasOutput::default());
        let target_output: Model<ChartCanvasOutput> =
            app.models_mut().insert(ChartCanvasOutput::default());
        let shared_domain_windows = app
            .models_mut()
            .insert(BTreeMap::<LinkAxisKey, Option<DataWindow>>::default());
        let shared_brush = app.models_mut().insert(None::<BrushSelectionLink2D>);
        let shared_axis_pointer = app.models_mut().insert(None::<AxisPointerLinkAnchor>);

        let y_axis = AxisId::new(3);
        let y_key = LinkAxisKey {
            kind: AxisKind::Y,
            dataset: DatasetId::new(1),
            field: FieldId::new(2),
        };
        let source_window = DataWindow {
            min: -0.25,
            max: 0.75,
        };
        let target_initial_window = DataWindow {
            min: -5.0,
            max: 5.0,
        };
        let explicit = BTreeMap::from([(y_axis, y_key)]);
        let spec = multi_axis_spec();

        let mut source_engine =
            ChartEngine::new(spec.clone()).expect("source spec should be valid");
        source_engine.apply_action(Action::SetDataWindowY {
            axis: y_axis,
            window: Some(source_window),
        });
        let source_router = ChartLinkRouter::from_model(source_engine.model())
            .with_explicit_axis_map(explicit.clone());
        let source_snapshot = chart_canvas_output_snapshot_for_engine(
            &source_engine,
            &source_router,
            Vec::new(),
            &DefaultTooltipFormatter,
        );
        app.models_mut()
            .update(&source_output, |output| {
                let link_events_revision = output.link_events_revision.saturating_add(1);
                assert!(update_chart_canvas_output(
                    output,
                    source_snapshot,
                    link_events_revision
                ));
            })
            .expect("source output model should be writable");

        let target_engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("target spec should be valid"));
        app.models_mut()
            .update(&target_engine, |engine| {
                engine.apply_action(Action::SetDataWindowY {
                    axis: y_axis,
                    window: Some(target_initial_window),
                });
            })
            .expect("target engine model should be writable");
        let target_router =
            ChartLinkRouter::from_spec(&spec).with_explicit_axis_map(explicit.clone());

        let mut linked = crate::linking::LinkedChartGroup::new(
            crate::linking::ChartLinkPolicy {
                brush: false,
                axis_pointer: false,
                domain_windows: true,
            },
            shared_brush,
            shared_axis_pointer,
            shared_domain_windows.clone(),
        );
        linked
            .push(crate::linking::LinkedChartMember {
                router: source_router,
                output: source_output.clone(),
            })
            .push(crate::linking::LinkedChartMember {
                router: target_router,
                output: target_output.clone(),
            });

        assert!(
            linked.tick(&mut app),
            "linked group should copy the source domain window into shared state"
        );
        let shared = shared_domain_windows
            .read(&mut app, |_app, windows| windows.clone())
            .expect("shared domain windows model should be readable");
        assert_eq!(
            shared.get(&y_key).copied(),
            Some(Some(source_window)),
            "shared linked-domain state should contain the source explicit Y window"
        );

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "chart-linked-target-panel",
            |cx| {
                let mut props = ChartCanvasPanelProps::new(spec.clone())
                    .output_model(target_output.clone())
                    .link_axis_map(explicit.clone())
                    .linked_domain_windows(shared_domain_windows.clone())
                    .test_id("linked-target-chart");
                props.engine = Some(target_engine.clone());
                vec![chart_canvas_panel(cx, props)]
            },
        );
        ui.set_root(root);
        pump_chart_frame(&mut ui, &mut app, &mut services, bounds);
        pump_chart_frame(&mut ui, &mut app, &mut services, bounds);

        let target_published = target_output
            .read(&mut app, |_app, state| state.clone())
            .expect("target output model should be readable");
        assert_eq!(
            target_published
                .snapshot
                .domain_windows_by_key
                .get(&y_key)
                .copied(),
            Some(Some(source_window)),
            "target declarative chart should apply the shared explicit Y window and publish it back"
        );
        assert_ne!(
            target_published
                .snapshot
                .domain_windows_by_key
                .get(&y_key)
                .copied(),
            Some(Some(target_initial_window)),
            "target declarative chart output should not remain at its initial local Y window"
        );
    }

    #[test]
    fn chart_canvas_panel_grid_view_publishes_grid_viewport_without_global_viewport() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(12.0), Px(18.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices;
        let (spec, dataset_id, x, y) = chart_spec();
        let grid = GridId::new(1);

        let engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("chart spec should be valid"));
        app.models_mut()
            .update(&engine, |engine| seed_dataset(engine, dataset_id, x, y))
            .expect("chart engine model should exist");

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "chart-declarative-grid-view-panel",
            |cx| {
                let mut props = ChartCanvasPanelProps::new(spec.clone()).grid_view(grid);
                props.engine = Some(engine.clone());
                vec![chart_canvas_panel(cx, props)]
            },
        );
        ui.set_root(root);
        pump_chart_frame(&mut ui, &mut app, &mut services, bounds);

        let (global_viewport, grid_viewport) = app
            .models()
            .read(&engine, |engine| {
                (
                    engine.model().viewport,
                    engine.model().plot_viewports_by_grid.get(&grid).copied(),
                )
            })
            .expect("chart engine model should exist");
        assert_eq!(
            grid_viewport,
            Some(bounds),
            "declarative grid view should publish its panel bounds as the grid plot viewport"
        );
        assert_eq!(
            global_viewport, None,
            "declarative grid view should not overwrite the shared engine's global viewport"
        );
    }

    #[test]
    fn chart_canvas_panel_grid_view_paints_only_series_for_that_grid() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(180.0)),
        );
        let mut services = FakeServices;
        let (spec, dataset_id) = multi_grid_spec();
        let grid_1 = GridId::new(1);
        let grid_2 = GridId::new(2);

        let engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("chart spec should be valid"));
        app.models_mut()
            .update(&engine, |engine| {
                seed_multi_grid_dataset(engine, dataset_id)
            })
            .expect("chart engine model should exist");

        let render_grids = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            let root = render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "chart-declarative-grid-view-filter-panel",
                |cx| {
                    let mut column = fret_ui::element::ColumnProps::default();
                    column.layout.size.width = Length::Fill;
                    column.layout.size.height = Length::Fill;

                    vec![cx.column(column, |cx| {
                        let mut grid_1_props =
                            ChartCanvasPanelProps::new(spec.clone()).grid_view(grid_1);
                        grid_1_props.engine = Some(engine.clone());
                        grid_1_props.pointer_region.layout.flex.grow = 1.0;
                        grid_1_props.pointer_region.layout.flex.basis = Length::Px(Px(0.0));

                        let mut grid_2_props =
                            ChartCanvasPanelProps::new(spec.clone()).grid_view(grid_2);
                        grid_2_props.engine = Some(engine.clone());
                        grid_2_props.pointer_region.layout.flex.grow = 1.0;
                        grid_2_props.pointer_region.layout.flex.basis = Length::Px(Px(0.0));

                        vec![
                            chart_canvas_panel(cx, grid_1_props),
                            chart_canvas_panel(cx, grid_2_props),
                        ]
                    })]
                },
            );
            ui.set_root(root);
            ui.layout_all(app, services, bounds, 1.0);
            let mut scene = Scene::default();
            ui.paint_all(app, services, bounds, &mut scene, 1.0);
            app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
            scene
        };

        let _ = render_grids(&mut ui, &mut app, &mut services);
        let scene = render_grids(&mut ui, &mut app, &mut services);
        let grid_1_bounds = Rect::new(
            bounds.origin,
            Size::new(bounds.size.width, Px(bounds.size.height.0 / 2.0)),
        );
        let grid_2_bounds = Rect::new(
            Point::new(
                bounds.origin.x,
                Px(bounds.origin.y.0 + bounds.size.height.0 / 2.0),
            ),
            Size::new(bounds.size.width, Px(bounds.size.height.0 / 2.0)),
        );

        let chart_mark_quads = |panel_bounds: Rect| {
            scene
                .ops()
                .iter()
                .filter_map(|op| match op {
                    SceneOp::Quad { rect, order, .. }
                        if order.0 >= ChartStyle::default().draw_order.0
                            && rect.size.width.0 > 0.0
                            && rect.size.height.0 > 0.0
                            && *rect != panel_bounds
                            && panel_bounds.contains(rect.origin) =>
                    {
                        Some(*rect)
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        let grid_1_quads = chart_mark_quads(grid_1_bounds);
        let grid_2_quads = chart_mark_quads(grid_2_bounds);
        assert!(
            !grid_1_quads.is_empty(),
            "first declarative grid view should paint marks for grid 1"
        );
        assert!(
            !grid_2_quads.is_empty(),
            "second declarative grid view should paint marks for grid 2"
        );
        assert_ne!(
            grid_1_quads, grid_2_quads,
            "per-grid declarative panels should not paint the same mark set from the shared engine"
        );
    }

    #[test]
    fn chart_canvas_panel_overlay_hit_test_falls_through_outside_legend_panel() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(180.0)),
        );
        let mut services = FakeServices;
        let (spec, dataset_id, x, y) = chart_spec();

        let engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("chart spec should be valid"));
        app.models_mut()
            .update(&engine, |engine| seed_dataset(engine, dataset_id, x, y))
            .expect("chart engine model should exist");

        let render_overlay = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            let root = render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "chart-declarative-overlay-hit-test-panel",
                |cx| {
                    let mut stack = fret_ui::element::StackProps::default();
                    stack.layout.size.width = Length::Fill;
                    stack.layout.size.height = Length::Fill;

                    vec![cx.stack_props(stack, |cx| {
                        let mut underlay = PointerRegionProps::default();
                        underlay.layout.size.width = Length::Fill;
                        underlay.layout.size.height = Length::Fill;
                        let underlay = cx
                            .pointer_region(underlay, |_cx| Vec::new())
                            .test_id("chart-overlay-underlay");

                        let mut overlay_props = ChartCanvasPanelProps::new(spec.clone())
                            .overlay_only()
                            .test_id("chart-overlay-only");
                        overlay_props.engine = Some(engine.clone());
                        let overlay = chart_canvas_panel(cx, overlay_props);

                        vec![underlay, overlay]
                    })]
                },
            );
            ui.set_root(root);
            ui.layout_all(app, services, bounds, 1.0);
            let mut scene = Scene::default();
            ui.paint_all(app, services, bounds, &mut scene, 1.0);
            app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
            root
        };

        let _ = render_overlay(&mut ui, &mut app, &mut services);
        let root = render_overlay(&mut ui, &mut app, &mut services);

        let stack = ui.children(root)[0];
        let underlay = ui.children(stack)[0];
        let overlay = ui.children(stack)[1];
        let legend_hit = ui.debug_hit_test(Point::new(Px(330.0), Px(28.0))).hit;

        assert_eq!(
            ui.debug_hit_test(Point::new(Px(8.0), Px(8.0))).hit,
            Some(underlay),
            "overlay-only chart panel should fall through outside the legend panel"
        );
        assert!(
            legend_hit.is_some_and(|node| ui.is_descendant_via_children(overlay, node)),
            "overlay-only chart panel should remain hit-testable over the legend panel"
        );
    }

    #[test]
    fn chart_canvas_panel_keyboard_navigation_publishes_tooltip_lines_on_declarative_path() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(293.5), Px(296.5)),
            Size::new(Px(560.0), Px(208.0)),
        );
        let mut services = FakeServices;
        let (spec, dataset_id, x, y) = chart_spec();

        let engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("chart spec should be valid"));
        app.models_mut()
            .update(&engine, |engine| seed_dataset(engine, dataset_id, x, y))
            .expect("chart engine model should exist");
        let output: Model<ChartCanvasOutput> =
            app.models_mut().insert(ChartCanvasOutput::default());

        let render_frame = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            let engine = engine.clone();
            let output = output.clone();
            let spec = spec.clone();
            let root = render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "chart-declarative-a11y-panel",
                |cx| {
                    let mut props = ChartCanvasPanelProps::new(spec.clone())
                        .output_model(output)
                        .accessibility_layer(true)
                        .test_id("chart-keyboard-canvas");
                    props.engine = Some(engine);
                    props.input_map = crate::input_map::ChartInputMap::default();
                    vec![chart_canvas_panel(cx, props)]
                },
            );
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);
            let mut scene = Scene::default();
            ui.paint_all(app, services, bounds, &mut scene, 1.0);
        };

        render_frame(&mut ui, &mut app, &mut services);
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        render_frame(&mut ui, &mut app, &mut services);

        let before = ui
            .semantics_snapshot()
            .expect("expected semantics snapshot before keyboard navigation");
        let before_node = before
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some("chart-keyboard-canvas"))
            .expect("expected chart semantics node before keyboard navigation");
        assert_eq!(
            before_node.pos_in_set,
            Some(1),
            "expected initial chart semantics collection position to point at the first item"
        );

        ui.set_focus(Some(before_node.id));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        render_frame(&mut ui, &mut app, &mut services);
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        render_frame(&mut ui, &mut app, &mut services);

        let after = ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after keyboard navigation");
        let after_node = after
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some("chart-keyboard-canvas"))
            .expect("expected chart semantics node after keyboard navigation");
        let published = output
            .read(&mut app, |_app, state| state.clone())
            .expect("expected output model to be readable");

        assert_eq!(
            after_node.pos_in_set,
            Some(2),
            "expected keyboard accessibility navigation to update chart semantics collection position"
        );
        assert!(
            published.revision > 0,
            "expected keyboard accessibility navigation to advance the shared output model revision; after_pos_in_set={:?} after_value={:?} tooltip_lines={}",
            after_node.pos_in_set,
            after_node.value,
            published.snapshot.tooltip_lines.len()
        );
        assert!(
            !published.snapshot.tooltip_lines.is_empty(),
            "expected keyboard accessibility navigation to publish tooltip lines"
        );
    }

    #[test]
    fn chart_canvas_panel_paints_line_and_scatter_marks_on_declarative_path() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices;
        let (spec, dataset_id, x, y_line, y_scatter) = line_scatter_chart_spec();

        let engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("chart spec should be valid"));
        app.models_mut()
            .update(&engine, |engine| {
                seed_line_scatter_dataset(engine, dataset_id, x, y_line, y_scatter)
            })
            .expect("chart engine model should exist");

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "chart-declarative-line-scatter-panel",
            |cx| {
                vec![chart_canvas_panel(
                    cx,
                    ChartCanvasPanelProps {
                        engine: Some(engine.clone()),
                        spec: spec.clone(),
                        ..ChartCanvasPanelProps::new(spec.clone())
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let (has_polyline_marks, has_point_marks) = app
            .models()
            .read(&engine, |engine| {
                let has_polyline = engine.output().marks.nodes.iter().any(|node| {
                    matches!(
                        (node.kind, &node.payload),
                        (MarkKind::Polyline, MarkPayloadRef::Polyline(poly))
                            if poly.points.end > poly.points.start
                    )
                });
                let has_points = engine.output().marks.nodes.iter().any(|node| {
                    matches!(
                        (node.kind, &node.payload),
                        (MarkKind::Points, MarkPayloadRef::Points(points))
                            if points.points.end > points.points.start
                    )
                });
                (has_polyline, has_points)
            })
            .expect("chart engine model should exist");
        assert!(
            has_polyline_marks,
            "seeded line series should produce polyline marks before declarative paint"
        );
        assert!(
            has_point_marks,
            "seeded scatter series should produce point marks before declarative paint"
        );

        assert!(
            scene
                .ops()
                .iter()
                .any(|op| matches!(op, SceneOp::Path { .. })),
            "declarative chart canvas should paint line marks as scene paths"
        );
        assert!(
            scene.ops().iter().any(|op| {
                matches!(
                    op,
                    SceneOp::Quad { rect, order, .. }
                        if order.0 >= ChartStyle::default().draw_order.0
                            && rect.size.width.0 > 0.0
                            && rect.size.height.0 > 0.0
                            && *rect != bounds
                )
            }),
            "declarative chart canvas should paint scatter marks as non-zero quads"
        );

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }
}
