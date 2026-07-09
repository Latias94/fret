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

use crate::ChartStyle;
use crate::a11y::ChartA11yIndex;
use crate::input_map::{ChartInputMap, ModifierKey};
use crate::linking::{AxisPointerLinkAnchor, BrushSelectionLink2D, ChartLinkRouter, LinkAxisKey};
use crate::output::{
    ChartCanvasOutput, chart_canvas_output_link_events_batch,
    chart_canvas_output_snapshot_for_engine, update_chart_canvas_output,
};
use crate::{DefaultTooltipFormatter, TooltipFormatter, TooltipTextLine};

use super::data_zoom_overlay::{
    DataZoomOverlayState, data_zoom_overlay_tool, data_zoom_tracks_for_engine,
};
use super::legend_overlay::{LegendOverlayState, LegendSeriesEntry, legend_overlay_tool};
use super::tooltip_overlay::{AxisPointerLabelOverlay, TooltipOverlayState, tooltip_overlay_tool};
use super::visual_map_overlay::{
    VisualMapOverlayState, visual_map_overlay_tool, visual_map_tracks_for_engine,
};

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
    pan_x: bool,
    pan_y: bool,
    start_x: DataWindow,
    start_y: DataWindow,
}

#[derive(Debug, Default, Clone, Copy)]
struct ChartActiveAxesState {
    x_axis: Option<delinea::AxisId>,
    y_axis: Option<delinea::AxisId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChartAxisRegion {
    Plot,
    XAxis(delinea::AxisId),
    YAxis(delinea::AxisId),
    Outside,
}

#[derive(Debug, Clone, Copy)]
struct ChartAxisBandLayout {
    axis: delinea::AxisId,
    position: delinea::AxisPosition,
    rect: Rect,
}

#[derive(Debug, Clone)]
struct ChartPanelLayout {
    plot: Rect,
    x_axes: Vec<ChartAxisBandLayout>,
    y_axes: Vec<ChartAxisBandLayout>,
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

fn active_grid_for_chart(model: &delinea::engine::model::ChartModel) -> Option<delinea::GridId> {
    model
        .series_in_order()
        .find(|series| series.visible)
        .and_then(|series| model.axes.get(&series.x_axis).map(|axis| axis.grid))
}

fn chart_panel_layout_for_engine(
    engine: &ChartEngine,
    bounds: Rect,
    style: ChartStyle,
) -> Option<ChartPanelLayout> {
    let model = engine.model();
    let active_grid = active_grid_for_chart(model)?;

    let has_visual_map = model.series_in_order().any(|series| {
        series.visible
            && model
                .axes
                .get(&series.x_axis)
                .is_some_and(|axis| axis.grid == active_grid)
            && model.visual_map_by_series.contains_key(&series.id)
    });

    let axis_band_x = style.axis_band_x.0.max(0.0);
    let axis_band_y = style.axis_band_y.0.max(0.0);
    let visual_map_band_x = if has_visual_map {
        style.visual_map_band_x.0.max(0.0)
    } else {
        0.0
    };

    let mut x_top: Vec<delinea::AxisId> = Vec::new();
    let mut x_bottom: Vec<delinea::AxisId> = Vec::new();
    let mut y_left: Vec<delinea::AxisId> = Vec::new();
    let mut y_right: Vec<delinea::AxisId> = Vec::new();

    for (axis_id, axis) in &model.axes {
        if axis.grid != active_grid {
            continue;
        }

        match (axis.kind, axis.position) {
            (delinea::AxisKind::X, delinea::AxisPosition::Top) => x_top.push(*axis_id),
            (delinea::AxisKind::X, delinea::AxisPosition::Bottom) => x_bottom.push(*axis_id),
            (delinea::AxisKind::Y, delinea::AxisPosition::Left) => y_left.push(*axis_id),
            (delinea::AxisKind::Y, delinea::AxisPosition::Right) => y_right.push(*axis_id),
            _ => {}
        }
    }

    let mut inner = bounds;
    inner.origin.x.0 += style.padding.left.0;
    inner.origin.y.0 += style.padding.top.0;
    inner.size.width.0 =
        (inner.size.width.0 - style.padding.left.0 - style.padding.right.0).max(0.0);
    inner.size.height.0 =
        (inner.size.height.0 - style.padding.top.0 - style.padding.bottom.0).max(0.0);

    let left_total = axis_band_x * (y_left.len() as f32);
    let right_total = axis_band_x * (y_right.len() as f32);
    let top_total = axis_band_y * (x_top.len() as f32);
    let bottom_total = axis_band_y * (x_bottom.len() as f32);

    let plot_w = (inner.size.width.0 - left_total - right_total - visual_map_band_x).max(0.0);
    let plot_h = (inner.size.height.0 - top_total - bottom_total).max(0.0);
    let plot = Rect::new(
        Point::new(
            Px(inner.origin.x.0 + left_total),
            Px(inner.origin.y.0 + top_total),
        ),
        Size::new(Px(plot_w), Px(plot_h)),
    );

    let mut x_axes: Vec<ChartAxisBandLayout> = Vec::with_capacity(x_top.len() + x_bottom.len());
    for (i, axis) in x_top.iter().copied().enumerate() {
        let rect = Rect::new(
            Point::new(
                plot.origin.x,
                Px(plot.origin.y.0 - axis_band_y * (i as f32 + 1.0)),
            ),
            Size::new(plot.size.width, Px(axis_band_y)),
        );
        x_axes.push(ChartAxisBandLayout {
            axis,
            position: delinea::AxisPosition::Top,
            rect,
        });
    }
    for (i, axis) in x_bottom.iter().copied().enumerate() {
        let rect = Rect::new(
            Point::new(
                plot.origin.x,
                Px(plot.origin.y.0 + plot.size.height.0 + axis_band_y * (i as f32)),
            ),
            Size::new(plot.size.width, Px(axis_band_y)),
        );
        x_axes.push(ChartAxisBandLayout {
            axis,
            position: delinea::AxisPosition::Bottom,
            rect,
        });
    }

    let mut y_axes: Vec<ChartAxisBandLayout> = Vec::with_capacity(y_left.len() + y_right.len());
    for (i, axis) in y_left.iter().copied().enumerate() {
        let rect = Rect::new(
            Point::new(
                Px(plot.origin.x.0 - axis_band_x * (i as f32 + 1.0)),
                plot.origin.y,
            ),
            Size::new(Px(axis_band_x), plot.size.height),
        );
        y_axes.push(ChartAxisBandLayout {
            axis,
            position: delinea::AxisPosition::Left,
            rect,
        });
    }
    for (i, axis) in y_right.iter().copied().enumerate() {
        let rect = Rect::new(
            Point::new(
                Px(plot.origin.x.0 + plot.size.width.0 + axis_band_x * (i as f32)),
                plot.origin.y,
            ),
            Size::new(Px(axis_band_x), plot.size.height),
        );
        y_axes.push(ChartAxisBandLayout {
            axis,
            position: delinea::AxisPosition::Right,
            rect,
        });
    }

    Some(ChartPanelLayout {
        plot,
        x_axes,
        y_axes,
    })
}

fn chart_axis_region(layout: &ChartPanelLayout, position: Point) -> ChartAxisRegion {
    for axis in &layout.x_axes {
        if axis.rect.contains(position) {
            return ChartAxisRegion::XAxis(axis.axis);
        }
    }
    for axis in &layout.y_axes {
        if axis.rect.contains(position) {
            return ChartAxisRegion::YAxis(axis.axis);
        }
    }
    if layout.plot.contains(position) {
        ChartAxisRegion::Plot
    } else {
        ChartAxisRegion::Outside
    }
}

fn axis_pointer_hover_point_for_layout(layout: &ChartPanelLayout, position: Point) -> Point {
    let plot = layout.plot;
    if plot.contains(position) {
        return position;
    }

    let plot_left = plot.origin.x.0;
    let plot_top = plot.origin.y.0;
    let plot_right = plot.origin.x.0 + plot.size.width.0;
    let plot_bottom = plot.origin.y.0 + plot.size.height.0;

    let x_in_plot = position.x.0.clamp(plot_left, plot_right);
    let y_in_plot = position.y.0.clamp(plot_top, plot_bottom);

    if layout
        .x_axes
        .iter()
        .any(|axis| axis.rect.contains(position))
    {
        let y = (plot_bottom - 1.0).max(plot_top);
        return Point::new(Px(x_in_plot), Px(y));
    }

    if let Some(axis) = layout
        .y_axes
        .iter()
        .find(|axis| axis.rect.contains(position))
    {
        let x = match axis.position {
            delinea::AxisPosition::Right => (plot_right - 1.0).max(plot_left),
            _ => (plot_left + 1.0).min(plot_right),
        };
        return Point::new(Px(x), Px(y_in_plot));
    }

    position
}

fn update_active_axes_for_position(
    active_axes: &Arc<Mutex<ChartActiveAxesState>>,
    layout: &ChartPanelLayout,
    position: Point,
) {
    let region = chart_axis_region(layout, position);
    if let Ok(mut state) = active_axes.lock() {
        match region {
            ChartAxisRegion::XAxis(axis) => state.x_axis = Some(axis),
            ChartAxisRegion::YAxis(axis) => state.y_axis = Some(axis),
            ChartAxisRegion::Plot | ChartAxisRegion::Outside => {}
        }
    }
}

fn chart_axis_is_present_in_layout(
    layout: &ChartPanelLayout,
    axis: delinea::AxisId,
    kind: delinea::AxisKind,
) -> bool {
    match kind {
        delinea::AxisKind::X => layout.x_axes.iter().any(|band| band.axis == axis),
        delinea::AxisKind::Y => layout.y_axes.iter().any(|band| band.axis == axis),
    }
}

fn active_axes_for_layout(
    engine: &ChartEngine,
    layout: &ChartPanelLayout,
    active_axes: &Arc<Mutex<ChartActiveAxesState>>,
) -> Option<(delinea::AxisId, delinea::AxisId)> {
    let (primary_x, primary_y) = primary_axes(engine)?;
    let state = active_axes
        .lock()
        .ok()
        .map(|state| *state)
        .unwrap_or_default();
    let x_axis = state
        .x_axis
        .filter(|axis| chart_axis_is_present_in_layout(layout, *axis, delinea::AxisKind::X))
        .unwrap_or(primary_x);
    let y_axis = state
        .y_axis
        .filter(|axis| chart_axis_is_present_in_layout(layout, *axis, delinea::AxisKind::Y))
        .unwrap_or(primary_y);

    Some((x_axis, y_axis))
}

fn axis_range(engine: &ChartEngine, axis: delinea::AxisId) -> delinea::AxisRange {
    engine
        .model()
        .axes
        .get(&axis)
        .map(|axis| axis.range)
        .unwrap_or_default()
}

fn axis_is_fixed(engine: &ChartEngine, axis: delinea::AxisId) -> Option<DataWindow> {
    match axis_range(engine, axis) {
        delinea::AxisRange::Fixed { min, max } => {
            let mut window = DataWindow { min, max };
            window.clamp_non_degenerate();
            Some(window)
        }
        _ => None,
    }
}

fn axis_pan_locked(engine: &ChartEngine, axis: delinea::AxisId) -> bool {
    engine
        .state()
        .axis_locks
        .get(&axis)
        .copied()
        .unwrap_or_default()
        .pan_locked
}

fn axis_zoom_locked(engine: &ChartEngine, axis: delinea::AxisId) -> bool {
    engine
        .state()
        .axis_locks
        .get(&axis)
        .copied()
        .unwrap_or_default()
        .zoom_locked
}

pub(crate) fn paint_color(style: ChartStyle, paint: delinea::PaintId) -> Color {
    let palette = &style.series_palette;
    palette[(paint.0 as usize) % palette.len()]
}

fn series_color(
    style: ChartStyle,
    series: delinea::SeriesId,
    series_rank_by_id: &BTreeMap<delinea::SeriesId, usize>,
) -> Color {
    let palette = &style.series_palette;
    let order = series_rank_by_id.get(&series).copied().unwrap_or(0);
    palette[order % palette.len()]
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ChartCanvasPanelMode {
    #[default]
    Full,
    GridView(delinea::GridId),
    Overlay,
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

fn compute_series_rank_by_id(
    engine: &ChartEngine,
    mode: ChartCanvasPanelMode,
) -> BTreeMap<delinea::SeriesId, usize> {
    let model = engine.model();
    model
        .series_in_order()
        .filter(|s| series_is_in_panel_grid(mode, model, s.id))
        .enumerate()
        .map(|(order, s)| (s.id, order))
        .collect()
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
    cx.observe_model(&engine, fret_ui::Invalidation::Paint);

    let legend_state: Arc<Mutex<LegendOverlayState>> = cx.slot_state(
        || Arc::new(Mutex::new(LegendOverlayState::default())),
        |st| st.clone(),
    );
    let tooltip_state: Arc<Mutex<TooltipOverlayState>> = cx.slot_state(
        || Arc::new(Mutex::new(TooltipOverlayState::default())),
        |st| st.clone(),
    );
    let visual_map_state: Arc<Mutex<VisualMapOverlayState>> = cx.slot_state(
        || Arc::new(Mutex::new(VisualMapOverlayState::default())),
        |st| st.clone(),
    );
    let data_zoom_state: Arc<Mutex<DataZoomOverlayState>> = cx.slot_state(
        || Arc::new(Mutex::new(DataZoomOverlayState::default())),
        |st| st.clone(),
    );
    let active_axes_state: Arc<Mutex<ChartActiveAxesState>> = cx.slot_state(
        || Arc::new(Mutex::new(ChartActiveAxesState::default())),
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
    let style = props.style;
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
    let mut data_zoom_tracks = Vec::new();

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
        let next_series_rank_by_id = compute_series_rank_by_id(engine, mode);
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
        series_rank_by_id = next_series_rank_by_id;

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

        data_zoom_tracks = data_zoom_tracks_for_engine(engine, bounds, style);
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
    let visual_map_tracks = engine
        .read(cx.app, |_app, engine| {
            visual_map_tracks_for_engine(engine, bounds, style)
        })
        .ok()
        .unwrap_or_default();
    if let Ok(mut st) = visual_map_state.lock() {
        st.sync_tracks(visual_map_tracks);
    }
    if let Ok(mut st) = data_zoom_state.lock() {
        st.sync_tracks(data_zoom_tracks);
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

    let engine_c = engine.clone();
    let input_map = props.input_map;

    let pan_drag_down = pan_drag.clone();
    let active_axes_down = active_axes_state.clone();
    let style_down = style;
    let on_pan_down: OnCanvasToolPointerDown = Arc::new(move |host, _action_cx, tool_cx, down| {
        if !input_map.pan.matches(down.button, down.modifiers) {
            return CanvasToolDownResult::unhandled();
        }

        let Some((x_axis, y_axis, pan_x, pan_y, start_x, start_y)) = host
            .models_mut()
            .read(&engine_c, |engine| {
                let layout = chart_panel_layout_for_engine(engine, tool_cx.bounds, style_down)?;
                update_active_axes_for_position(&active_axes_down, &layout, down.position);
                let region = chart_axis_region(&layout, down.position);
                let in_plot = layout.plot.contains(down.position);
                let in_axis = matches!(
                    region,
                    ChartAxisRegion::XAxis(_) | ChartAxisRegion::YAxis(_)
                );
                if !in_plot && !in_axis {
                    return None;
                }

                let (active_x_axis, active_y_axis) =
                    active_axes_for_layout(engine, &layout, &active_axes_down)?;
                let (x_axis, y_axis, mut pan_x, mut pan_y) = match region {
                    ChartAxisRegion::Plot => (
                        active_x_axis,
                        active_y_axis,
                        !down.modifiers.ctrl,
                        !down.modifiers.shift,
                    ),
                    ChartAxisRegion::XAxis(axis) => (axis, active_y_axis, true, false),
                    ChartAxisRegion::YAxis(axis) => (active_x_axis, axis, false, true),
                    ChartAxisRegion::Outside => return None,
                };

                if pan_x && axis_is_fixed(engine, x_axis).is_some() {
                    pan_x = false;
                }
                if pan_y && axis_is_fixed(engine, y_axis).is_some() {
                    pan_y = false;
                }
                if pan_x && axis_pan_locked(engine, x_axis) {
                    pan_x = false;
                }
                if pan_y && axis_pan_locked(engine, y_axis) {
                    pan_y = false;
                }
                if !pan_x && !pan_y {
                    return None;
                }

                Some((
                    x_axis,
                    y_axis,
                    pan_x,
                    pan_y,
                    window_for_axis_x(engine, x_axis),
                    window_for_axis_y(engine, y_axis),
                ))
            })
            .ok()
            .flatten()
        else {
            return CanvasToolDownResult::unhandled();
        };

        let _ = host.models_mut().update(&pan_drag_down, |st| {
            *st = Some(ChartPanDrag {
                start_pos: down.position,
                x_axis,
                y_axis,
                pan_x,
                pan_y,
                start_x,
                start_y,
            });
        });

        CanvasToolDownResult::activate_and_capture()
    });

    let pan_drag_move = pan_drag.clone();
    let engine_c = engine.clone();
    let style_move = style;
    let on_pan_move: OnCanvasToolPointerMove = Arc::new(move |host, action_cx, tool_cx, mv| {
        let Some(drag) = host
            .models_mut()
            .read(&pan_drag_move, |st| *st)
            .ok()
            .flatten()
        else {
            return false;
        };

        let Some(layout) = host
            .models_mut()
            .read(&engine_c, |engine| {
                chart_panel_layout_for_engine(engine, tool_cx.bounds, style_move)
            })
            .ok()
            .flatten()
        else {
            return false;
        };
        let width = layout.plot.size.width.0;
        let height = layout.plot.size.height.0;
        if width <= 0.0 || height <= 0.0 {
            return false;
        }

        let dx = mv.position.x.0 - drag.start_pos.x.0;
        let dy = mv.position.y.0 - drag.start_pos.y.0;

        let _ = host.models_mut().update(&engine_c, |engine| {
            if drag.pan_x
                && axis_is_fixed(engine, drag.x_axis).is_none()
                && !axis_pan_locked(engine, drag.x_axis)
            {
                engine.apply_action(Action::PanDataWindowXFromBase {
                    axis: drag.x_axis,
                    base: drag.start_x,
                    delta_px: dx,
                    viewport_span_px: width,
                });
            }
            if drag.pan_y
                && axis_is_fixed(engine, drag.y_axis).is_none()
                && !axis_pan_locked(engine, drag.y_axis)
            {
                engine.apply_action(Action::PanDataWindowYFromBase {
                    axis: drag.y_axis,
                    base: drag.start_y,
                    delta_px: -dy,
                    viewport_span_px: height,
                });
            }
            engine.apply_action(Action::HoverAt {
                point: axis_pointer_hover_point_for_layout(&layout, mv.position),
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
    let active_axes_hover = active_axes_state.clone();
    let style_hover = style;
    let on_hover_move: OnCanvasToolPointerMove = Arc::new(move |host, action_cx, _tool_cx, mv| {
        let _ = host.models_mut().update(&engine_c, |engine| {
            let Some(layout) = chart_panel_layout_for_engine(engine, _tool_cx.bounds, style_hover)
            else {
                return;
            };
            update_active_axes_for_position(&active_axes_hover, &layout, mv.position);
            let region = chart_axis_region(&layout, mv.position);
            let in_plot = layout.plot.contains(mv.position);
            let in_axis = matches!(
                region,
                ChartAxisRegion::XAxis(_) | ChartAxisRegion::YAxis(_)
            );
            if in_plot || in_axis {
                engine.apply_action(Action::HoverAt {
                    point: axis_pointer_hover_point_for_layout(&layout, mv.position),
                });
            }
        });
        host.request_redraw(action_cx.window);
        true
    });

    let engine_c = engine.clone();
    let input_map_c = input_map;
    let active_axes_wheel = active_axes_state.clone();
    let style_wheel = style;
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

        let Some((
            x_axis,
            y_axis,
            zoom_x,
            zoom_y,
            base_x,
            base_y,
            center_x,
            center_y_from_bottom,
            width,
            height,
        )) = host
            .models_mut()
            .read(&engine_c, |engine| {
                let layout = chart_panel_layout_for_engine(engine, tool_cx.bounds, style_wheel)?;
                update_active_axes_for_position(&active_axes_wheel, &layout, wheel.position);

                let width = layout.plot.size.width.0;
                let height = layout.plot.size.height.0;
                if width <= 0.0 || height <= 0.0 {
                    return None;
                }

                let region = chart_axis_region(&layout, wheel.position);
                let in_plot = layout.plot.contains(wheel.position);
                let in_axis = matches!(
                    region,
                    ChartAxisRegion::XAxis(_) | ChartAxisRegion::YAxis(_)
                );
                if !in_plot && !in_axis {
                    return None;
                }

                let local_x = (wheel.position.x.0 - layout.plot.origin.x.0).clamp(0.0, width);
                let local_y = (wheel.position.y.0 - layout.plot.origin.y.0).clamp(0.0, height);
                let center_x = local_x;
                let center_y_from_bottom = height - local_y;

                let (active_x_axis, active_y_axis) =
                    active_axes_for_layout(engine, &layout, &active_axes_wheel)?;
                let (x_axis, y_axis) = match region {
                    ChartAxisRegion::XAxis(axis) => (axis, active_y_axis),
                    ChartAxisRegion::YAxis(axis) => (active_x_axis, axis),
                    ChartAxisRegion::Plot => (active_x_axis, active_y_axis),
                    ChartAxisRegion::Outside => return None,
                };

                let (zoom_x, zoom_y) = match region {
                    ChartAxisRegion::XAxis(_) => (true, false),
                    ChartAxisRegion::YAxis(_) => (false, true),
                    ChartAxisRegion::Plot => (!wheel.modifiers.ctrl, !wheel.modifiers.shift),
                    ChartAxisRegion::Outside => return None,
                };

                Some((
                    x_axis,
                    y_axis,
                    zoom_x,
                    zoom_y,
                    window_for_axis_x(engine, x_axis),
                    window_for_axis_y(engine, y_axis),
                    center_x,
                    center_y_from_bottom,
                    width,
                    height,
                ))
            })
            .ok()
            .flatten()
        else {
            return false;
        };

        // Match ImPlot's default feel: zoom factor ~= 2^(delta_y * 0.0025)
        let log2_scale = delta_y * 0.0025;

        let _ = host.models_mut().update(&engine_c, |engine| {
            if zoom_x
                && axis_is_fixed(engine, x_axis).is_none()
                && !axis_zoom_locked(engine, x_axis)
            {
                engine.apply_action(Action::ZoomDataWindowXFromBase {
                    axis: x_axis,
                    base: base_x,
                    center_px: center_x,
                    log2_scale,
                    viewport_span_px: width,
                });
            }
            if zoom_y
                && axis_is_fixed(engine, y_axis).is_none()
                && !axis_zoom_locked(engine, y_axis)
            {
                engine.apply_action(Action::ZoomDataWindowYFromBase {
                    axis: y_axis,
                    base: base_y,
                    center_px: center_y_from_bottom,
                    log2_scale,
                    viewport_span_px: height,
                });
            }
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
        tools.push(visual_map_overlay_tool(
            engine.clone(),
            visual_map_state.clone(),
            style,
        ));
        tools.push(data_zoom_overlay_tool(
            engine.clone(),
            data_zoom_state.clone(),
            style,
        ));
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
    let series_rank_by_id = Arc::new(series_rank_by_id);
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
                            let stroke_color = if let Some(series) = node.source_series {
                                series_color(style, series, &series_rank_by_id)
                            } else if let Some((paint, _)) = &poly.stroke {
                                paint_color(style, *paint)
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
                                    background = series_color(style, series, &series_rank_by_id);
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
                                    fill = series_color(style, series, &series_rank_by_id);
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

    fn first_chart_bar_spec() -> (ChartSpec, DatasetId, SeriesId, Vec<f64>, Vec<f64>, Vec<f64>) {
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
        let x_field = FieldId::new(1);
        let desktop_field = FieldId::new(2);
        let mobile_field = FieldId::new(3);
        let desktop_series = SeriesId::new(1);
        let mobile_series = SeriesId::new(2);

        let categories = vec![
            "January".to_string(),
            "February".to_string(),
            "March".to_string(),
            "April".to_string(),
            "May".to_string(),
            "June".to_string(),
        ];
        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let desktop = vec![186.0, 305.0, 237.0, 73.0, 209.0, 214.0];
        let mobile = vec![80.0, 200.0, 120.0, 190.0, 130.0, 140.0];

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
                        id: desktop_field,
                        column: 1,
                    },
                    FieldSpec {
                        id: mobile_field,
                        column: 2,
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
                    scale: AxisScale::Category(delinea::CategoryAxisScale { categories }),
                    range: Default::default(),
                },
                delinea::AxisSpec {
                    id: y_axis,
                    name: Some("Visitors".to_string()),
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: None,
                    scale: Default::default(),
                    range: Default::default(),
                },
            ],
            data_zoom_x: vec![],
            data_zoom_y: vec![],
            tooltip: None,
            axis_pointer: Some(delinea::AxisPointerSpec::default()),
            visual_maps: vec![],
            series: vec![
                SeriesSpec {
                    id: desktop_series,
                    name: Some("Desktop".to_string()),
                    kind: SeriesKind::Bar,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: desktop_field,
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
                    id: mobile_series,
                    name: Some("Mobile".to_string()),
                    kind: SeriesKind::Bar,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: mobile_field,
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

        (spec, dataset_id, desktop_series, x, desktop, mobile)
    }

    fn seed_dataset(engine: &mut ChartEngine, dataset_id: DatasetId, x: Vec<f64>, y: Vec<f64>) {
        let mut table = DataTable::default();
        table.push_column(Column::F64(x));
        table.push_column(Column::F64(y));
        engine.datasets_mut().insert(dataset_id, table);
    }

    fn seed_first_chart_dataset(
        engine: &mut ChartEngine,
        dataset_id: DatasetId,
        x: Vec<f64>,
        desktop: Vec<f64>,
        mobile: Vec<f64>,
    ) {
        let mut table = DataTable::default();
        table.push_column(Column::F64(x));
        table.push_column(Column::F64(desktop));
        table.push_column(Column::F64(mobile));
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
                    position: Some(delinea::AxisPosition::Bottom),
                    scale: AxisScale::default(),
                    range: None,
                },
                delinea::AxisSpec {
                    id: y_left,
                    name: None,
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: Some(delinea::AxisPosition::Left),
                    scale: AxisScale::default(),
                    range: None,
                },
                delinea::AxisSpec {
                    id: y_right,
                    name: None,
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: Some(delinea::AxisPosition::Right),
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

    fn multi_axis_visual_map_spec(mode: delinea::VisualMapMode) -> ChartSpec {
        let mut spec = multi_axis_spec();
        let series_id = spec.series[0].id;
        let y_field = spec.series[0].encode.y;
        spec.visual_maps.push(delinea::VisualMapSpec {
            id: delinea::VisualMapId::new(1),
            mode,
            dataset: None,
            series: vec![series_id],
            field: y_field,
            domain: (-1.0, 1.0),
            initial_range: matches!(mode, delinea::VisualMapMode::Continuous)
                .then_some((-0.25, 0.75)),
            initial_piece_mask: None,
            point_radius_mul_range: None,
            stroke_width_range: None,
            opacity_mul_range: None,
            buckets: 8,
            out_of_range_opacity: 0.25,
        });
        spec
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

    fn reversed_series_id_spec() -> (ChartSpec, DatasetId, Vec<f64>, Vec<f64>, Vec<f64>) {
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
        let x_field = FieldId::new(1);
        let high_id_field = FieldId::new(2);
        let low_id_field = FieldId::new(3);

        let mut spec = ChartSpec {
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
                        id: high_id_field,
                        column: 1,
                    },
                    FieldSpec {
                        id: low_id_field,
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
            series: Vec::new(),
        };

        spec.series.push(SeriesSpec {
            id: SeriesId::new(42),
            name: Some("First in order".to_string()),
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: high_id_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        });
        spec.series.push(SeriesSpec {
            id: SeriesId::new(1),
            name: Some("Second in order".to_string()),
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: low_id_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        });

        (
            spec,
            dataset_id,
            vec![0.0, 1.0, 2.0],
            vec![0.0, 1.0, 0.0],
            vec![2.0, 3.0, 2.0],
        )
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

    fn seed_reversed_series_dataset(
        engine: &mut ChartEngine,
        dataset_id: DatasetId,
        x: Vec<f64>,
        high_id_y: Vec<f64>,
        low_id_y: Vec<f64>,
    ) {
        let mut table = DataTable::default();
        table.push_column(Column::F64(x));
        table.push_column(Column::F64(high_id_y));
        table.push_column(Column::F64(low_id_y));
        engine.datasets_mut().insert(dataset_id, table);
    }

    fn many_legend_series_spec(series_count: usize) -> ChartSpec {
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
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
            series: (0..series_count)
                .map(|index| SeriesSpec {
                    id: SeriesId::new((index + 1) as u64),
                    name: Some(format!("Series {index:02}")),
                    kind: SeriesKind::Line,
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
                })
                .collect(),
        }
    }

    fn legend_item_point(
        bounds: Rect,
        style: ChartStyle,
        row: usize,
        series_count: usize,
    ) -> Point {
        legend_item_point_with_scroll(bounds, style, row, series_count, Px(0.0))
    }

    fn legend_item_point_with_scroll(
        bounds: Rect,
        style: ChartStyle,
        row: usize,
        series_count: usize,
        scroll_y: Px,
    ) -> Point {
        let text_w = 10.0f32;
        let text_h = 10.0f32;
        let selector_text_w = 10.0f32;
        let selector_text_h = 10.0f32;
        let selector_gap = 8.0f32;
        let selector_count = 3usize;

        let pad = style.legend_padding;
        let sw = style.legend_swatch_size.0.max(1.0);
        let sw_gap = style.legend_swatch_gap.0.max(0.0);
        let gap = style.legend_item_gap.0.max(0.0);
        let row_h = text_h.max(sw);
        let legend_w = (pad.left.0 + sw + sw_gap + text_w + pad.right.0).max(1.0);
        let selector_total_w =
            selector_text_w * selector_count as f32 + selector_gap * (selector_count - 1) as f32;
        let selector_row_h = (selector_text_h + 4.0).max(1.0);
        let items_h = ((row_h + gap) * (series_count.saturating_sub(1) as f32) + row_h).max(1.0);
        let full_h = (pad.top.0 + selector_row_h + items_h + pad.bottom.0).max(1.0);
        let margin = 8.0f32;
        let min_h = (pad.top.0 + row_h + pad.bottom.0).max(1.0);
        let max_h = (bounds.size.height.0 - 2.0 * margin).max(min_h);
        let legend_h = full_h.min(max_h);
        let _view_h = (legend_h - selector_row_h - pad.top.0 - pad.bottom.0).max(1.0);

        let x0 = (bounds.origin.x.0 + bounds.size.width.0 - legend_w - margin)
            .max(bounds.origin.x.0 + margin);
        let y0 = bounds.origin.y.0 + margin;
        let _selector_x0 = x0 + legend_w - pad.right.0 - selector_total_w;
        let items_y = y0 + pad.top.0 + selector_row_h;
        Point::new(
            Px(x0 + pad.left.0 + 1.0),
            Px(items_y + row as f32 * (row_h + gap) + 0.5 * row_h - scroll_y.0),
        )
    }

    fn legend_test_max_scroll_y(bounds: Rect, style: ChartStyle, series_count: usize) -> Px {
        let text_h = 10.0f32;
        let selector_text_h = 10.0f32;
        let pad = style.legend_padding;
        let sw = style.legend_swatch_size.0.max(1.0);
        let gap = style.legend_item_gap.0.max(0.0);
        let row_h = text_h.max(sw);
        let selector_row_h = (selector_text_h + 4.0).max(1.0);
        let items_h = ((row_h + gap) * (series_count.saturating_sub(1) as f32) + row_h).max(1.0);
        let full_h = (pad.top.0 + selector_row_h + items_h + pad.bottom.0).max(1.0);
        let margin = 8.0f32;
        let min_h = (pad.top.0 + row_h + pad.bottom.0).max(1.0);
        let max_h = (bounds.size.height.0 - 2.0 * margin).max(min_h);
        let legend_h = full_h.min(max_h);
        let view_h = (legend_h - selector_row_h - pad.top.0 - pad.bottom.0).max(1.0);

        crate::legend_logic::legend_max_scroll_y(Px(items_h), Px(view_h))
    }

    fn legend_selector_point(bounds: Rect, style: ChartStyle, selector_index: usize) -> Point {
        let text_w = 10.0f32;
        let selector_text_w = 10.0f32;
        let selector_text_h = 10.0f32;
        let selector_gap = 8.0f32;
        let selector_count = 3usize;

        let pad = style.legend_padding;
        let sw = style.legend_swatch_size.0.max(1.0);
        let sw_gap = style.legend_swatch_gap.0.max(0.0);
        let legend_w = (pad.left.0 + sw + sw_gap + text_w + pad.right.0).max(1.0);
        let selector_total_w =
            selector_text_w * selector_count as f32 + selector_gap * (selector_count - 1) as f32;
        let selector_row_h = (selector_text_h + 4.0).max(1.0);

        let margin = 8.0f32;
        let x0 = (bounds.origin.x.0 + bounds.size.width.0 - legend_w - margin)
            .max(bounds.origin.x.0 + margin);
        let y0 = bounds.origin.y.0 + margin;
        let selector_x0 = x0 + legend_w - pad.right.0 - selector_total_w;
        let x = selector_x0 + selector_index as f32 * (selector_text_w + selector_gap);
        Point::new(
            Px((x + 0.5 * selector_text_w).max(x0 + 1.0)),
            Px(y0 + pad.top.0 + 0.5 * selector_row_h),
        )
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
    fn chart_canvas_panel_paints_and_drags_continuous_visual_map_on_declarative_path() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(240.0)),
        );
        let mut services = FakeServices;
        let spec = multi_axis_visual_map_spec(delinea::VisualMapMode::Continuous);

        let engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("chart spec should be valid"));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "chart-declarative-visual-map-panel",
            |cx| {
                let mut props =
                    ChartCanvasPanelProps::new(spec.clone()).test_id("chart-visual-map");
                props.engine = Some(engine.clone());
                vec![chart_canvas_panel(cx, props)]
            },
        );
        ui.set_root(root);

        let render_frame = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            ui.layout_all(app, services, bounds, 1.0);
            let mut scene = Scene::default();
            ui.paint_all(app, services, bounds, &mut scene, 1.0);
            app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
            scene
        };

        let _ = render_frame(&mut ui, &mut app, &mut services);
        let scene = render_frame(&mut ui, &mut app, &mut services);

        let tracks = app
            .models()
            .read(&engine, |engine| {
                crate::declarative::visual_map_overlay::visual_map_tracks_for_engine(
                    engine,
                    bounds,
                    ChartStyle::default(),
                )
            })
            .expect("visual-map tracks should be readable");
        assert_eq!(
            tracks.len(),
            1,
            "expected a single declarative visual map track"
        );

        let track = tracks[0];
        let track_order = ChartStyle::default().draw_order.0.saturating_add(8_600);
        let domain = crate::visual_map_logic::visual_map_domain_window(track.model);
        let initial_selection = scene
            .ops()
            .iter()
            .find_map(|op| match op {
                SceneOp::Quad { order, rect, .. } if order.0 == track_order.saturating_add(2) => {
                    Some(*rect)
                }
                _ => None,
            })
            .expect("expected a declarative visual-map selection quad");
        let expected_initial = Rect::new(
            Point::new(
                track.track.origin.x,
                Px(crate::visual_map_logic::visual_map_y_at_value(
                    track.track,
                    domain,
                    track.current_window.max,
                )
                .min(crate::visual_map_logic::visual_map_y_at_value(
                    track.track,
                    domain,
                    track.current_window.min,
                ))),
            ),
            Size::new(
                track.track.size.width,
                Px((crate::visual_map_logic::visual_map_y_at_value(
                    track.track,
                    domain,
                    track.current_window.max,
                ) - crate::visual_map_logic::visual_map_y_at_value(
                    track.track,
                    domain,
                    track.current_window.min,
                ))
                .abs()
                .max(1.0)),
            ),
        );
        assert_eq!(
            initial_selection, expected_initial,
            "declarative visual-map paint should reflect the current continuous window"
        );

        let drag_x = track.track.origin.x.0 + 0.5 * track.track.size.width.0;
        let drag_y = track.track.origin.y.0 + 0.5 * track.track.size.height.0;
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(drag_x), Px(drag_y)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: fret_core::PointerId(1),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert!(
            ui.captured_for(fret_core::PointerId(1)).is_some(),
            "continuous visual-map drag should capture"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(Px(drag_x), Px(drag_y + 16.0)),
                buttons: fret_core::MouseButtons {
                    left: true,
                    right: false,
                    middle: false,
                },
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(1),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let changed = app.take_changed_models();
        if !changed.is_empty() {
            assert!(ui.propagate_model_changes(&mut app, &changed));
        }
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(drag_x), Px(drag_y + 16.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: false,
                click_count: 1,
                pointer_id: fret_core::PointerId(1),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let changed = app.take_changed_models();
        if !changed.is_empty() {
            assert!(ui.propagate_model_changes(&mut app, &changed));
        }
        assert!(
            ui.captured_for(fret_core::PointerId(1)).is_none(),
            "continuous visual-map drag should release capture"
        );

        let _ = render_frame(&mut ui, &mut app, &mut services);
        let scene_after = render_frame(&mut ui, &mut app, &mut services);
        let updated_tracks = app
            .models()
            .read(&engine, |engine| {
                crate::declarative::visual_map_overlay::visual_map_tracks_for_engine(
                    engine,
                    bounds,
                    ChartStyle::default(),
                )
            })
            .expect("updated visual-map tracks should be readable");
        let updated_track = updated_tracks[0];
        assert_ne!(
            updated_track.current_window, track.current_window,
            "continuous visual-map drag should update the current data window"
        );

        let updated_selection = scene_after
            .ops()
            .iter()
            .find_map(|op| match op {
                SceneOp::Quad { order, rect, .. } if order.0 == track_order.saturating_add(2) => {
                    Some(*rect)
                }
                _ => None,
            })
            .expect("expected a declarative visual-map selection quad after dragging");
        let updated_domain = crate::visual_map_logic::visual_map_domain_window(updated_track.model);
        let updated_y_min = crate::visual_map_logic::visual_map_y_at_value(
            updated_track.track,
            updated_domain,
            updated_track.current_window.min,
        );
        let updated_y_max = crate::visual_map_logic::visual_map_y_at_value(
            updated_track.track,
            updated_domain,
            updated_track.current_window.max,
        );
        let updated_expected = Rect::new(
            Point::new(
                updated_track.track.origin.x,
                Px(updated_y_max.min(updated_y_min)),
            ),
            Size::new(
                updated_track.track.size.width,
                Px((updated_y_max - updated_y_min).abs().max(1.0)),
            ),
        );
        assert_eq!(
            updated_selection, updated_expected,
            "declarative visual-map paint should track the updated continuous window"
        );
    }

    #[test]
    fn chart_canvas_panel_visual_map_y_mapping_respects_domain_endpoints_on_declarative_path() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(240.0)),
        );
        let mut services = FakeServices;
        let mut spec = multi_axis_visual_map_spec(delinea::VisualMapMode::Continuous);
        spec.visual_maps[0].initial_range = None;
        let style = ChartStyle::default();

        let engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("chart spec should be valid"));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "chart-declarative-visual-map-endpoint-panel",
            |cx| {
                let mut props =
                    ChartCanvasPanelProps::new(spec.clone()).test_id("chart-visual-map-endpoints");
                props.engine = Some(engine.clone());
                vec![chart_canvas_panel(cx, props)]
            },
        );
        ui.set_root(root);

        pump_chart_frame(&mut ui, &mut app, &mut services, bounds);
        let mut scene = Scene::default();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let track = app
            .models()
            .read(&engine, |engine| {
                visual_map_tracks_for_engine(engine, bounds, style)
                    .into_iter()
                    .next()
                    .expect("expected a single declarative visual-map track")
            })
            .expect("visual-map tracks should be readable");
        let domain = crate::visual_map_logic::visual_map_domain_window(track.model);
        let bottom = track.track.origin.y.0 + track.track.size.height.0;
        assert_eq!(
            crate::visual_map_logic::visual_map_y_at_value(track.track, domain, domain.min),
            bottom,
            "declarative visual-map helper should map the domain minimum to the track bottom"
        );
        assert_eq!(
            crate::visual_map_logic::visual_map_y_at_value(track.track, domain, domain.max),
            track.track.origin.y.0,
            "declarative visual-map helper should map the domain maximum to the track top"
        );

        let track_order = style.draw_order.0.saturating_add(8_600);
        let selection = scene
            .ops()
            .iter()
            .find_map(|op| match op {
                SceneOp::Quad { order, rect, .. } if order.0 == track_order.saturating_add(2) => {
                    Some(*rect)
                }
                _ => None,
            })
            .expect("expected a declarative visual-map selection quad");
        assert_eq!(
            selection, track.track,
            "a full-domain visual-map selection should fill the declarative track"
        );
    }

    #[test]
    fn chart_canvas_panel_visual_map_track_applies_style_padding_on_declarative_path() {
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
        let spec = multi_axis_visual_map_spec(delinea::VisualMapMode::Continuous);

        let engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("chart spec should be valid"));

        let render_frame = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            let engine = engine.clone();
            let spec = spec.clone();
            let root = render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "chart-declarative-visual-map-padding-panel",
                |cx| {
                    let mut props = ChartCanvasPanelProps::new(spec.clone())
                        .test_id("chart-visual-map-padding");
                    props.engine = Some(engine);
                    let mut style = ChartStyle::default();
                    style.visual_map_band_x = Px(80.0);
                    style.visual_map_padding = Px(10.0);
                    props.style = style;
                    vec![chart_canvas_panel(cx, props)]
                },
            );
            ui.set_root(root);
            ui.layout_all(app, services, bounds, 1.0);
            let mut scene = Scene::default();
            ui.paint_all(app, services, bounds, &mut scene, 1.0);
        };

        render_frame(&mut ui, &mut app, &mut services);
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        render_frame(&mut ui, &mut app, &mut services);

        let style = {
            let mut style = ChartStyle::default();
            style.visual_map_band_x = Px(80.0);
            style.visual_map_padding = Px(10.0);
            style
        };
        let tracks = app
            .models()
            .read(&engine, |engine| {
                visual_map_tracks_for_engine(engine, bounds, style)
            })
            .expect("visual-map tracks should be readable");
        assert_eq!(tracks.len(), 1);

        let layout = app
            .models()
            .read(&engine, |engine| {
                chart_panel_layout_for_engine(engine, bounds, style)
            })
            .expect("chart engine model should be readable")
            .expect("expected a chart panel layout");
        let right_axis_count = layout
            .y_axes
            .iter()
            .filter(|band| band.position == delinea::AxisPosition::Right)
            .count() as f32;
        let outer = Rect::new(
            Point::new(
                Px(layout.plot.origin.x.0
                    + layout.plot.size.width.0
                    + style.axis_band_x.0 * right_axis_count),
                layout.plot.origin.y,
            ),
            Size::new(
                Px(style.visual_map_band_x.0.max(0.0)),
                layout.plot.size.height,
            ),
        );
        let track = tracks[0].track;
        assert_eq!(track.origin.x.0, outer.origin.x.0 + 10.0);
        assert_eq!(track.origin.y.0, outer.origin.y.0 + 10.0);
    }

    #[test]
    fn chart_canvas_panel_toggles_piecewise_visual_map_mask_on_declarative_path() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(240.0)),
        );
        let mut services = FakeServices;
        let spec = multi_axis_visual_map_spec(delinea::VisualMapMode::Piecewise);

        let engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("chart spec should be valid"));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "chart-declarative-piecewise-visual-map-panel",
            |cx| {
                let mut props =
                    ChartCanvasPanelProps::new(spec.clone()).test_id("chart-piecewise-visual-map");
                props.engine = Some(engine.clone());
                vec![chart_canvas_panel(cx, props)]
            },
        );
        ui.set_root(root);

        let render_frame = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            ui.layout_all(app, services, bounds, 1.0);
            let mut scene = Scene::default();
            ui.paint_all(app, services, bounds, &mut scene, 1.0);
            app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
            scene
        };

        let _ = render_frame(&mut ui, &mut app, &mut services);
        let track = app
            .models()
            .read(&engine, |engine| {
                crate::declarative::visual_map_overlay::visual_map_tracks_for_engine(
                    engine,
                    bounds,
                    ChartStyle::default(),
                )
            })
            .expect("visual-map tracks should be readable")[0];
        let initial_mask = track.current_piece_mask;
        assert_eq!(
            initial_mask,
            crate::visual_map_logic::visual_map_full_piece_mask(track.model),
            "piecewise visual map should start fully selected"
        );

        let click_x = track.track.origin.x.0 + 0.5 * track.track.size.width.0;
        let click_y = track.track.origin.y.0 + 0.5 * track.track.size.height.0;
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(click_x), Px(click_y)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: fret_core::PointerId(2),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(click_x), Px(click_y)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_id: fret_core::PointerId(2),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let _ = render_frame(&mut ui, &mut app, &mut services);
        let updated_mask = app
            .models()
            .read(&engine, |engine| {
                crate::declarative::visual_map_overlay::visual_map_tracks_for_engine(
                    engine,
                    bounds,
                    ChartStyle::default(),
                )
            })
            .expect("updated visual-map tracks should be readable")[0]
            .current_piece_mask;
        assert_ne!(
            updated_mask, initial_mask,
            "piecewise visual-map click should toggle at least one bucket"
        );
    }

    #[test]
    fn chart_canvas_panel_paints_and_drags_x_data_zoom_on_declarative_path() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(240.0)),
        );
        let mut services = FakeServices;
        let spec = multi_axis_spec();
        let x_axis = spec.axes[0].id;

        let engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("chart spec should be valid"));
        app.models_mut()
            .update(&engine, |engine| {
                engine.apply_action(delinea::Action::SetDataWindowX {
                    axis: x_axis,
                    window: Some(delinea::engine::window::DataWindow { min: 0.2, max: 0.8 }),
                });
            })
            .expect("chart engine model should exist");

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "chart-declarative-data-zoom-panel",
            |cx| {
                let mut props = ChartCanvasPanelProps::new(spec.clone()).test_id("chart-data-zoom");
                props.engine = Some(engine.clone());
                vec![chart_canvas_panel(cx, props)]
            },
        );
        ui.set_root(root);

        let render_frame = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            ui.layout_all(app, services, bounds, 1.0);
            let mut scene = Scene::default();
            ui.paint_all(app, services, bounds, &mut scene, 1.0);
            app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
            scene
        };

        let _ = render_frame(&mut ui, &mut app, &mut services);
        let scene = render_frame(&mut ui, &mut app, &mut services);

        let tracks = app
            .models_mut()
            .update(&engine, |engine| {
                crate::declarative::data_zoom_overlay::data_zoom_tracks_for_engine(
                    engine,
                    bounds,
                    ChartStyle::default(),
                )
            })
            .expect("data-zoom tracks should be readable");
        assert_eq!(
            tracks.len(),
            2,
            "expected declarative data-zoom tracks for the primary x and y axes"
        );

        let x_track = tracks
            .iter()
            .find(|track| {
                matches!(
                    track.axis_kind,
                    crate::declarative::data_zoom_overlay::DataZoomAxisKind::X
                )
            })
            .copied()
            .expect("expected a declarative x data-zoom track");

        let y_track = tracks
            .iter()
            .find(|track| {
                matches!(
                    track.axis_kind,
                    crate::declarative::data_zoom_overlay::DataZoomAxisKind::Y
                )
            })
            .copied()
            .expect("expected a declarative y data-zoom track");

        assert!(
            y_track.track.size.height.0 > 0.0,
            "expected a declarative y data-zoom track"
        );

        let track_order = ChartStyle::default().draw_order.0.saturating_add(8_650);
        let initial_t0 =
            crate::slider_logic::slider_norm(x_track.extent, x_track.current_window.min);
        let initial_t1 =
            crate::slider_logic::slider_norm(x_track.extent, x_track.current_window.max);
        let initial_left = x_track.track.origin.x.0 + initial_t0 * x_track.track.size.width.0;
        let initial_right = x_track.track.origin.x.0 + initial_t1 * x_track.track.size.width.0;
        let expected_initial = Rect::new(
            Point::new(Px(initial_left.min(initial_right)), x_track.track.origin.y),
            Size::new(
                Px((initial_right - initial_left).abs().max(1.0)),
                x_track.track.size.height,
            ),
        );
        let initial_selection = scene
            .ops()
            .iter()
            .find_map(|op| match op {
                SceneOp::Quad { order, rect, .. }
                    if order.0 == track_order.saturating_add(1) && *rect == expected_initial =>
                {
                    Some(*rect)
                }
                _ => None,
            })
            .expect("expected a declarative x data-zoom selection quad");
        assert_eq!(
            initial_selection, expected_initial,
            "declarative x data-zoom paint should reflect the current window"
        );

        let drag_x = x_track.track.origin.x.0 + 0.5 * x_track.track.size.width.0;
        let drag_y = x_track.track.origin.y.0 + 0.5 * x_track.track.size.height.0;
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(drag_x), Px(drag_y)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: fret_core::PointerId(3),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert!(
            ui.captured_for(fret_core::PointerId(3)).is_some(),
            "x data-zoom drag should capture"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(Px(drag_x + 24.0), Px(drag_y)),
                buttons: fret_core::MouseButtons {
                    left: true,
                    right: false,
                    middle: false,
                },
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(3),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let changed = app.take_changed_models();
        if !changed.is_empty() {
            assert!(ui.propagate_model_changes(&mut app, &changed));
        }
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(drag_x + 24.0), Px(drag_y)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: false,
                click_count: 1,
                pointer_id: fret_core::PointerId(3),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let changed = app.take_changed_models();
        if !changed.is_empty() {
            assert!(ui.propagate_model_changes(&mut app, &changed));
        }
        assert!(
            ui.captured_for(fret_core::PointerId(3)).is_none(),
            "x data-zoom drag should release capture"
        );

        let _ = render_frame(&mut ui, &mut app, &mut services);
        let scene_after = render_frame(&mut ui, &mut app, &mut services);
        let updated_tracks = app
            .models_mut()
            .update(&engine, |engine| {
                crate::declarative::data_zoom_overlay::data_zoom_tracks_for_engine(
                    engine,
                    bounds,
                    ChartStyle::default(),
                )
            })
            .expect("updated data-zoom tracks should be readable");
        let updated_x_track = updated_tracks
            .iter()
            .find(|track| {
                matches!(
                    track.axis_kind,
                    crate::declarative::data_zoom_overlay::DataZoomAxisKind::X
                )
            })
            .copied()
            .expect("expected an updated declarative x data-zoom track");
        assert_ne!(
            updated_x_track.current_window, x_track.current_window,
            "x data-zoom drag should update the current data window"
        );

        let updated_t0 =
            crate::slider_logic::slider_norm(x_track.extent, updated_x_track.current_window.min);
        let updated_t1 =
            crate::slider_logic::slider_norm(x_track.extent, updated_x_track.current_window.max);
        let updated_left = x_track.track.origin.x.0 + updated_t0 * x_track.track.size.width.0;
        let updated_right = x_track.track.origin.x.0 + updated_t1 * x_track.track.size.width.0;
        let updated_expected = Rect::new(
            Point::new(Px(updated_left.min(updated_right)), x_track.track.origin.y),
            Size::new(
                Px((updated_right - updated_left).abs().max(1.0)),
                x_track.track.size.height,
            ),
        );
        let updated_selection = scene_after
            .ops()
            .iter()
            .find_map(|op| match op {
                SceneOp::Quad { order, rect, .. }
                    if order.0 == track_order.saturating_add(1) && *rect == updated_expected =>
                {
                    Some(*rect)
                }
                _ => None,
            })
            .expect("expected a declarative x data-zoom selection quad after dragging");
        assert_eq!(
            updated_selection, updated_expected,
            "declarative x data-zoom paint should track the updated window"
        );
    }

    #[test]
    fn chart_canvas_panel_data_zoom_slider_clamps_and_never_inverts_on_declarative_path() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_debug_enabled(true);
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(240.0)),
        );
        let mut services = FakeServices;
        let spec = multi_axis_spec();
        let x_axis = spec.axes[0].id;
        let engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("chart spec should be valid"));
        let style = ChartStyle::default();
        let start_window = DataWindow { min: 0.2, max: 0.3 };

        let assert_window_close = |actual: DataWindow, expected: DataWindow| {
            let eps = 1e-6;
            assert!(
                (actual.min - expected.min).abs() <= eps
                    && (actual.max - expected.max).abs() <= eps,
                "expected window close to {expected:?}, got {actual:?}"
            );
        };
        let set_x_window = |app: &mut App, window: DataWindow| {
            app.models_mut()
                .update(&engine, |engine| {
                    engine.apply_action(Action::SetDataWindowX {
                        axis: x_axis,
                        window: Some(window),
                    });
                })
                .expect("chart engine model should exist");
        };
        let read_x_track = |app: &mut App| {
            app.models_mut()
                .update(&engine, |engine| {
                    data_zoom_tracks_for_engine(engine, bounds, style)
                        .into_iter()
                        .find(|track| {
                            matches!(
                                track.axis_kind,
                                crate::declarative::data_zoom_overlay::DataZoomAxisKind::X
                            )
                        })
                        .expect("expected a declarative x data-zoom track")
                })
                .expect("data-zoom tracks should be readable")
        };
        let propagate_changes = |ui: &mut UiTree<App>, app: &mut App| {
            let changed = app.take_changed_models();
            if !changed.is_empty() {
                assert!(ui.propagate_model_changes(app, &changed));
            }
        };
        let reset_for_drag = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            set_x_window(app, start_window);
            let _ = app.take_changed_models();
            let root = render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "chart-declarative-data-zoom-clamp-panel",
                |cx| {
                    let mut props =
                        ChartCanvasPanelProps::new(spec.clone()).test_id("chart-data-zoom-clamp");
                    props.engine = Some(engine.clone());
                    vec![chart_canvas_panel(cx, props)]
                },
            );
            ui.set_root(root);
            pump_chart_frame(ui, app, services, bounds);
            pump_chart_frame(ui, app, services, bounds);
            read_x_track(app)
        };
        let drag_slider = |ui: &mut UiTree<App>,
                           app: &mut App,
                           services: &mut FakeServices,
                           start: Point,
                           end: Point,
                           pointer_id: u64| {
            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(fret_core::PointerEvent::Down {
                    position: start,
                    button: MouseButton::Left,
                    modifiers: Modifiers::default(),
                    click_count: 1,
                    pointer_id: fret_core::PointerId(pointer_id),
                    pointer_type: fret_core::PointerType::Mouse,
                }),
            );
            assert!(
                ui.captured_for(fret_core::PointerId(pointer_id)).is_some(),
                "data-zoom slider drag should capture"
            );

            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(fret_core::PointerEvent::Move {
                    position: end,
                    buttons: fret_core::MouseButtons {
                        left: true,
                        right: false,
                        middle: false,
                    },
                    modifiers: Modifiers::default(),
                    pointer_id: fret_core::PointerId(pointer_id),
                    pointer_type: fret_core::PointerType::Mouse,
                }),
            );
            propagate_changes(ui, app);
            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(fret_core::PointerEvent::Up {
                    position: end,
                    button: MouseButton::Left,
                    modifiers: Modifiers::default(),
                    is_click: false,
                    click_count: 1,
                    pointer_id: fret_core::PointerId(pointer_id),
                    pointer_type: fret_core::PointerType::Mouse,
                }),
            );
            propagate_changes(ui, app);
            assert!(
                ui.captured_for(fret_core::PointerId(pointer_id)).is_none(),
                "data-zoom slider drag should release capture"
            );
            pump_chart_frame(ui, app, services, bounds);
            pump_chart_frame(ui, app, services, bounds);
        };

        let track = reset_for_drag(&mut ui, &mut app, &mut services);
        assert_window_close(track.extent, DataWindow { min: 0.0, max: 1.0 });
        let center = 0.5 * (start_window.min + start_window.max);
        let center_x = track.track.origin.x.0
            + crate::slider_logic::slider_norm(track.extent, center) * track.track.size.width.0;
        let mid_y = track.track.origin.y.0 + 0.5 * track.track.size.height.0;
        drag_slider(
            &mut ui,
            &mut app,
            &mut services,
            Point::new(Px(center_x), Px(mid_y)),
            Point::new(Px(track.track.origin.x.0 - 10_000.0), Px(mid_y)),
            31,
        );
        assert_window_close(
            read_x_track(&mut app).current_window,
            DataWindow { min: 0.0, max: 0.1 },
        );

        let track = reset_for_drag(&mut ui, &mut app, &mut services);
        let center_x = track.track.origin.x.0
            + crate::slider_logic::slider_norm(track.extent, center) * track.track.size.width.0;
        drag_slider(
            &mut ui,
            &mut app,
            &mut services,
            Point::new(Px(center_x), Px(mid_y)),
            Point::new(
                Px(track.track.origin.x.0 + track.track.size.width.0 + 10_000.0),
                Px(mid_y),
            ),
            32,
        );
        assert_window_close(
            read_x_track(&mut app).current_window,
            DataWindow { min: 0.9, max: 1.0 },
        );

        let track = reset_for_drag(&mut ui, &mut app, &mut services);
        let left_handle_x = track.track.origin.x.0
            + crate::slider_logic::slider_norm(track.extent, start_window.min)
                * track.track.size.width.0;
        drag_slider(
            &mut ui,
            &mut app,
            &mut services,
            Point::new(Px(left_handle_x), Px(mid_y)),
            Point::new(
                Px(track.track.origin.x.0 + track.track.size.width.0 + 10_000.0),
                Px(mid_y),
            ),
            33,
        );
        let clamped_min_handle = read_x_track(&mut app).current_window;
        assert!(
            clamped_min_handle.max > clamped_min_handle.min,
            "dragging the min handle past max must not invert the window"
        );
        assert!(
            clamped_min_handle.min >= track.extent.min
                && clamped_min_handle.max <= track.extent.max,
            "min-handle clamp must stay within extent"
        );
        assert!((clamped_min_handle.max - start_window.max).abs() <= 1e-6);

        let track = reset_for_drag(&mut ui, &mut app, &mut services);
        let right_handle_x = track.track.origin.x.0
            + crate::slider_logic::slider_norm(track.extent, start_window.max)
                * track.track.size.width.0;
        drag_slider(
            &mut ui,
            &mut app,
            &mut services,
            Point::new(Px(right_handle_x), Px(mid_y)),
            Point::new(Px(track.track.origin.x.0 - 10_000.0), Px(mid_y)),
            34,
        );
        let clamped_max_handle = read_x_track(&mut app).current_window;
        assert!(
            clamped_max_handle.max > clamped_max_handle.min,
            "dragging the max handle past min must not invert the window"
        );
        assert!(
            clamped_max_handle.min >= track.extent.min
                && clamped_max_handle.max <= track.extent.max,
            "max-handle clamp must stay within extent"
        );
        assert!((clamped_max_handle.min - start_window.min).abs() <= 1e-6);
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
    fn chart_canvas_panel_legend_double_click_isolates_and_restores_all_series_on_declarative_path()
    {
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
        let (spec, dataset_id, x, y_line, y_scatter) = line_scatter_chart_spec();
        let first = SeriesId::new(1);
        let second = SeriesId::new(2);
        let style = ChartStyle::default();

        let engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("chart spec should be valid"));
        app.models_mut()
            .update(&engine, |engine| {
                seed_line_scatter_dataset(engine, dataset_id, x, y_line, y_scatter)
            })
            .expect("chart engine model should exist");

        let render_frame = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            let engine = engine.clone();
            let spec = spec.clone();
            let root = render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "chart-declarative-legend-double-click-panel",
                |cx| {
                    let mut props = ChartCanvasPanelProps::new(spec.clone())
                        .test_id("chart-legend-double-click");
                    props.engine = Some(engine);
                    props.input_map = crate::input_map::ChartInputMap::default();
                    props.style = style;
                    vec![chart_canvas_panel(cx, props)]
                },
            );
            ui.set_root(root);
            ui.layout_all(app, services, bounds, 1.0);
            let mut scene = Scene::default();
            ui.paint_all(app, services, bounds, &mut scene, 1.0);
        };

        render_frame(&mut ui, &mut app, &mut services);
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        render_frame(&mut ui, &mut app, &mut services);

        let second_legend_row = legend_item_point(bounds, style, 1, 2);
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: second_legend_row,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 2,
                pointer_id: fret_core::PointerId(14),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let changed = app.take_changed_models();
        if !changed.is_empty() {
            assert!(ui.propagate_model_changes(&mut app, &changed));
        }

        let (first_visible, second_visible) = app
            .models()
            .read(&engine, |engine| {
                (
                    engine.model().series.get(&first).unwrap().visible,
                    engine.model().series.get(&second).unwrap().visible,
                )
            })
            .expect("chart engine model should be readable");
        assert!(
            !first_visible && second_visible,
            "double-clicking the second legend row should isolate that series"
        );

        render_frame(&mut ui, &mut app, &mut services);
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: second_legend_row,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 2,
                pointer_id: fret_core::PointerId(15),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let changed = app.take_changed_models();
        if !changed.is_empty() {
            assert!(ui.propagate_model_changes(&mut app, &changed));
        }

        let (first_visible, second_visible) = app
            .models()
            .read(&engine, |engine| {
                (
                    engine.model().series.get(&first).unwrap().visible,
                    engine.model().series.get(&second).unwrap().visible,
                )
            })
            .expect("chart engine model should be readable");
        assert!(
            first_visible && second_visible,
            "double-clicking the isolated legend row again should restore all series"
        );
    }

    #[test]
    fn chart_canvas_panel_legend_selectors_update_series_visibility_on_declarative_path() {
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
        let (spec, dataset_id, x, y_line, y_scatter) = line_scatter_chart_spec();
        let first = SeriesId::new(1);
        let second = SeriesId::new(2);
        let style = ChartStyle::default();

        let engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("chart spec should be valid"));
        app.models_mut()
            .update(&engine, |engine| {
                seed_line_scatter_dataset(engine, dataset_id, x, y_line, y_scatter)
            })
            .expect("chart engine model should exist");

        let render_frame = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            let engine = engine.clone();
            let spec = spec.clone();
            let root = render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "chart-declarative-legend-selector-panel",
                |cx| {
                    let mut props =
                        ChartCanvasPanelProps::new(spec.clone()).test_id("chart-legend-selector");
                    props.engine = Some(engine);
                    props.input_map = crate::input_map::ChartInputMap::default();
                    props.style = style;
                    vec![chart_canvas_panel(cx, props)]
                },
            );
            ui.set_root(root);
            ui.layout_all(app, services, bounds, 1.0);
            let mut scene = Scene::default();
            ui.paint_all(app, services, bounds, &mut scene, 1.0);
        };

        let dispatch_selector = |ui: &mut UiTree<App>,
                                 app: &mut App,
                                 services: &mut FakeServices,
                                 selector_index: usize,
                                 pointer_id: u64| {
            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(fret_core::PointerEvent::Down {
                    position: legend_selector_point(bounds, style, selector_index),
                    button: MouseButton::Left,
                    modifiers: Modifiers::default(),
                    click_count: 1,
                    pointer_id: fret_core::PointerId(pointer_id),
                    pointer_type: fret_core::PointerType::Mouse,
                }),
            );
            let changed = app.take_changed_models();
            if !changed.is_empty() {
                assert!(ui.propagate_model_changes(app, &changed));
            }
        };

        let read_visibility = |app: &mut App| {
            app.models()
                .read(&engine, |engine| {
                    (
                        engine.model().series.get(&first).unwrap().visible,
                        engine.model().series.get(&second).unwrap().visible,
                    )
                })
                .expect("chart engine model should be readable")
        };

        render_frame(&mut ui, &mut app, &mut services);
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        render_frame(&mut ui, &mut app, &mut services);

        dispatch_selector(&mut ui, &mut app, &mut services, 1, 16);
        assert_eq!(
            read_visibility(&mut app),
            (false, false),
            "legend None selector should hide all series"
        );

        render_frame(&mut ui, &mut app, &mut services);
        dispatch_selector(&mut ui, &mut app, &mut services, 0, 17);
        assert_eq!(
            read_visibility(&mut app),
            (true, true),
            "legend All selector should show all series"
        );

        app.models_mut()
            .update(&engine, |engine| {
                engine.apply_action(Action::SetSeriesVisible {
                    series: first,
                    visible: false,
                });
            })
            .expect("chart engine model should exist");
        render_frame(&mut ui, &mut app, &mut services);
        dispatch_selector(&mut ui, &mut app, &mut services, 2, 18);
        assert_eq!(
            read_visibility(&mut app),
            (true, false),
            "legend Invert selector should invert current series visibility"
        );
    }

    #[test]
    fn chart_canvas_panel_legend_scroll_clamps_to_content_height_on_declarative_path() {
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
        let series_count = 40usize;
        let spec = many_legend_series_spec(series_count);
        let style = ChartStyle::default();

        let engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("chart spec should be valid"));

        let render_frame = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            let engine = engine.clone();
            let spec = spec.clone();
            let root = render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "chart-declarative-legend-scroll-panel",
                |cx| {
                    let mut props =
                        ChartCanvasPanelProps::new(spec.clone()).test_id("chart-legend-scroll");
                    props.engine = Some(engine);
                    props.input_map = crate::input_map::ChartInputMap::default();
                    props.style = style;
                    vec![chart_canvas_panel(cx, props)]
                },
            );
            ui.set_root(root);
            ui.layout_all(app, services, bounds, 1.0);
            let mut scene = Scene::default();
            ui.paint_all(app, services, bounds, &mut scene, 1.0);
        };

        let dispatch_wheel =
            |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices, delta_y: f32| {
                ui.dispatch_event(
                    app,
                    services,
                    &Event::Pointer(fret_core::PointerEvent::Wheel {
                        position: legend_item_point(bounds, style, 0, series_count),
                        delta: Point::new(Px(0.0), Px(delta_y)),
                        modifiers: Modifiers::default(),
                        pointer_id: fret_core::PointerId(19),
                        pointer_type: fret_core::PointerType::Mouse,
                    }),
                );
                let changed = app.take_changed_models();
                if !changed.is_empty() {
                    assert!(ui.propagate_model_changes(app, &changed));
                }
            };

        let read_visible = |app: &mut App, series: SeriesId| {
            app.models()
                .read(&engine, |engine| {
                    engine.model().series.get(&series).unwrap().visible
                })
                .expect("chart engine model should be readable")
        };
        let hidden_series = |app: &mut App| {
            app.models()
                .read(&engine, |engine| {
                    engine
                        .model()
                        .series_in_order()
                        .filter(|series| !series.visible)
                        .map(|series| series.id.0)
                        .collect::<Vec<_>>()
                })
                .expect("chart engine model should be readable")
        };

        let dispatch_row_click = |ui: &mut UiTree<App>,
                                  app: &mut App,
                                  services: &mut FakeServices,
                                  row: usize,
                                  scroll_y: Px,
                                  pointer_id: u64| {
            let position =
                legend_item_point_with_scroll(bounds, style, row, series_count, scroll_y);
            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(fret_core::PointerEvent::Down {
                    position,
                    button: MouseButton::Left,
                    modifiers: Modifiers::default(),
                    click_count: 1,
                    pointer_id: fret_core::PointerId(pointer_id),
                    pointer_type: fret_core::PointerType::Mouse,
                }),
            );
            let changed = app.take_changed_models();
            if !changed.is_empty() {
                assert!(ui.propagate_model_changes(app, &changed));
            }
        };

        render_frame(&mut ui, &mut app, &mut services);
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        render_frame(&mut ui, &mut app, &mut services);

        dispatch_wheel(&mut ui, &mut app, &mut services, -200.0);
        render_frame(&mut ui, &mut app, &mut services);
        dispatch_row_click(&mut ui, &mut app, &mut services, 12, Px(150.0), 20);
        assert!(
            !read_visible(&mut app, SeriesId::new(13)),
            "initial negative legend wheel should scroll down and expose series 13"
        );

        dispatch_wheel(&mut ui, &mut app, &mut services, -10_000.0);
        let max_scroll = legend_test_max_scroll_y(bounds, style, series_count);
        assert_eq!(max_scroll.0, 422.0);
        render_frame(&mut ui, &mut app, &mut services);
        dispatch_row_click(&mut ui, &mut app, &mut services, 30, max_scroll, 21);
        assert!(
            !read_visible(&mut app, SeriesId::new(31)),
            "large negative legend wheel should clamp at the content bottom and expose series 31; hidden series: {:?}",
            hidden_series(&mut app)
        );

        dispatch_wheel(&mut ui, &mut app, &mut services, 10_000.0);
        render_frame(&mut ui, &mut app, &mut services);
        dispatch_row_click(&mut ui, &mut app, &mut services, 0, Px(0.0), 22);
        assert!(
            !read_visible(&mut app, SeriesId::new(1)),
            "large positive legend wheel should clamp back to the top and expose series 1; hidden series: {:?}",
            hidden_series(&mut app)
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
    fn chart_canvas_panel_pointer_hover_publishes_tooltip_lines_to_output_model_on_declarative_path()
     {
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
        let (spec, dataset_id, desktop_series, x, desktop, mobile) = first_chart_bar_spec();

        let engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("chart spec should be valid"));
        app.models_mut()
            .update(&engine, |engine| {
                seed_first_chart_dataset(engine, dataset_id, x, desktop, mobile)
            })
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
                "chart-declarative-pointer-hover-panel",
                |cx| {
                    let mut props = ChartCanvasPanelProps::new(spec.clone())
                        .output_model(output)
                        .test_id("chart-pointer-hover-canvas");
                    props.engine = Some(engine);
                    props.input_map = crate::input_map::ChartInputMap::default();
                    vec![chart_canvas_panel(cx, props)]
                },
            );
            ui.set_root(root);
            ui.layout_all(app, services, bounds, 1.0);
            let mut scene = Scene::default();
            ui.paint_all(app, services, bounds, &mut scene, 1.0);
        };

        render_frame(&mut ui, &mut app, &mut services);
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        render_frame(&mut ui, &mut app, &mut services);

        let hover_point = app
            .models_mut()
            .update(&engine, |engine| {
                point_for_series_data_index(engine, bounds, desktop_series, 0)
                    .expect("expected a point for the first desktop bar")
            })
            .expect("chart engine model should exist");
        assert!(
            bounds.contains(hover_point),
            "expected the derived hover point to land inside the non-zero-origin chart bounds"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                position: hover_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(7),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let changed = app.take_changed_models();
        if !changed.is_empty() {
            assert!(ui.propagate_model_changes(&mut app, &changed));
        }

        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        render_frame(&mut ui, &mut app, &mut services);
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        render_frame(&mut ui, &mut app, &mut services);

        let published = output
            .read(&mut app, |_app, state| state.clone())
            .expect("expected output model to be readable");

        assert!(
            published.revision > 0,
            "expected pointer hover to advance the shared output model revision"
        );
        assert!(
            !published.snapshot.tooltip_lines.is_empty(),
            "expected pointer hover to publish tooltip lines into the shared output model"
        );
        assert_eq!(
            published.snapshot.tooltip_lines[0].kind,
            crate::TooltipTextLineKind::AxisHeader,
            "expected the first tooltip line to be the axis header"
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

    #[test]
    fn chart_canvas_panel_uses_series_order_for_palette_on_declarative_path() {
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
        let (spec, dataset_id, x, high_id_y, low_id_y) = reversed_series_id_spec();
        let mut style = ChartStyle::default();
        style.series_palette[0] = Color::from_srgb_hex_rgb(0xff_00_00);
        style.series_palette[1] = Color::from_srgb_hex_rgb(0x00_ff_00);

        let engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("chart spec should be valid"));
        app.models_mut()
            .update(&engine, |engine| {
                seed_reversed_series_dataset(engine, dataset_id, x, high_id_y, low_id_y)
            })
            .expect("chart engine model should exist");

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "chart-declarative-series-order-palette-panel",
            |cx| {
                let mut props = ChartCanvasPanelProps::new(spec.clone());
                props.engine = Some(engine.clone());
                props.style = style;
                vec![chart_canvas_panel(cx, props)]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let path_series_by_draw_order: BTreeMap<u32, SeriesId> = app
            .models()
            .read(&engine, |engine| {
                engine
                    .output()
                    .marks
                    .nodes
                    .iter()
                    .filter_map(|node| {
                        let MarkPayloadRef::Polyline(poly) = &node.payload else {
                            return None;
                        };
                        let series = node.source_series?;
                        (node.kind == MarkKind::Polyline && poly.points.end > poly.points.start)
                            .then_some((style.draw_order.0.saturating_add(node.order.0), series))
                    })
                    .collect()
            })
            .expect("chart engine model should exist");

        assert_eq!(
            path_series_by_draw_order.len(),
            2,
            "expected one painted polyline mark for each declared line series"
        );

        let mut path_color_by_series = BTreeMap::new();
        for op in scene.ops() {
            let SceneOp::Path { order, paint, .. } = op else {
                continue;
            };
            let Some(series) = path_series_by_draw_order.get(&order.0) else {
                continue;
            };
            let fret_core::Paint::Solid(color) = paint.paint else {
                continue;
            };
            path_color_by_series.entry(*series).or_insert(color);
        }

        assert_eq!(
            path_color_by_series.get(&SeriesId::new(42)).copied(),
            Some(style.series_palette[0]),
            "first declared series should use palette slot 0 even when its SeriesId is not 0"
        );
        assert_eq!(
            path_color_by_series.get(&SeriesId::new(1)).copied(),
            Some(style.series_palette[1]),
            "second declared series should use palette slot 1 even when its SeriesId sorts before the first"
        );
        assert_ne!(
            style.series_palette[0], style.series_palette[1],
            "test palette must distinguish the first two series slots"
        );
    }

    #[test]
    fn chart_canvas_panel_axis_pointer_hover_point_clamps_axis_band_into_plot_on_declarative_path()
    {
        let plot = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        let layout = ChartPanelLayout {
            plot,
            x_axes: vec![ChartAxisBandLayout {
                axis: AxisId::new(1),
                position: delinea::AxisPosition::Bottom,
                rect: Rect::new(
                    Point::new(Px(0.0), Px(100.0)),
                    Size::new(Px(100.0), Px(20.0)),
                ),
            }],
            y_axes: vec![
                ChartAxisBandLayout {
                    axis: AxisId::new(2),
                    position: delinea::AxisPosition::Left,
                    rect: Rect::new(
                        Point::new(Px(-20.0), Px(0.0)),
                        Size::new(Px(20.0), Px(100.0)),
                    ),
                },
                ChartAxisBandLayout {
                    axis: AxisId::new(3),
                    position: delinea::AxisPosition::Right,
                    rect: Rect::new(
                        Point::new(Px(100.0), Px(0.0)),
                        Size::new(Px(20.0), Px(100.0)),
                    ),
                },
            ],
        };

        let p = axis_pointer_hover_point_for_layout(&layout, Point::new(Px(50.0), Px(110.0)));
        assert!(plot.contains(p));
        assert_eq!(p.x.0, 50.0);
        assert_eq!(p.y.0, 99.0);

        let p = axis_pointer_hover_point_for_layout(&layout, Point::new(Px(-10.0), Px(25.0)));
        assert!(plot.contains(p));
        assert_eq!(p.x.0, 1.0);
        assert_eq!(p.y.0, 25.0);

        let p = axis_pointer_hover_point_for_layout(&layout, Point::new(Px(110.0), Px(75.0)));
        assert!(plot.contains(p));
        assert_eq!(p.x.0, 99.0);
        assert_eq!(p.y.0, 75.0);
    }

    #[test]
    fn chart_canvas_panel_plot_pan_prefers_last_hovered_axis_band_on_declarative_path() {
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
        let spec = multi_axis_spec();
        let x_axis = AxisId::new(1);
        let y_left = AxisId::new(2);
        let y_right = AxisId::new(3);

        let engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("chart spec should be valid"));

        let render_frame = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            let engine = engine.clone();
            let spec = spec.clone();
            let root = render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "chart-declarative-active-axis-panel",
                |cx| {
                    let mut props =
                        ChartCanvasPanelProps::new(spec.clone()).test_id("chart-active-axis");
                    props.engine = Some(engine);
                    props.input_map = crate::input_map::ChartInputMap::default();
                    vec![chart_canvas_panel(cx, props)]
                },
            );
            ui.set_root(root);
            ui.layout_all(app, services, bounds, 1.0);
            let mut scene = Scene::default();
            ui.paint_all(app, services, bounds, &mut scene, 1.0);
        };

        render_frame(&mut ui, &mut app, &mut services);
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        render_frame(&mut ui, &mut app, &mut services);

        let layout = app
            .models()
            .read(&engine, |engine| {
                chart_panel_layout_for_engine(engine, bounds, ChartStyle::default())
            })
            .expect("chart engine model should be readable")
            .expect("expected a chart panel layout");
        let right_band = layout
            .y_axes
            .iter()
            .find(|band| band.position == delinea::AxisPosition::Right)
            .expect("expected a right y-axis band");
        let right_band_point = Point::new(
            Px(right_band.rect.origin.x.0 + 1.0),
            Px(right_band.rect.origin.y.0 + 0.5 * right_band.rect.size.height.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                position: right_band_point,
                buttons: fret_core::MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(11),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let changed = app.take_changed_models();
        if !changed.is_empty() {
            assert!(ui.propagate_model_changes(&mut app, &changed));
        }

        let start = Point::new(
            Px(layout.plot.origin.x.0 + 0.5 * layout.plot.size.width.0),
            Px(layout.plot.origin.y.0 + 0.5 * layout.plot.size.height.0),
        );
        let end = Point::new(start.x, Px(start.y.0 + 24.0));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: start,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: fret_core::PointerId(12),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert!(
            ui.captured_for(fret_core::PointerId(12)).is_some(),
            "plot pan should capture after active-axis selection"
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                position: end,
                buttons: fret_core::MouseButtons {
                    left: true,
                    right: false,
                    middle: false,
                },
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(12),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let changed = app.take_changed_models();
        if !changed.is_empty() {
            assert!(ui.propagate_model_changes(&mut app, &changed));
        }
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                position: end,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: false,
                click_count: 1,
                pointer_id: fret_core::PointerId(12),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let (x_window, left_window, right_window) = app
            .models()
            .read(&engine, |engine| {
                (
                    engine
                        .state()
                        .data_zoom_x
                        .get(&x_axis)
                        .and_then(|state| state.window),
                    engine.state().data_window_y.get(&y_left).copied(),
                    engine.state().data_window_y.get(&y_right).copied(),
                )
            })
            .expect("chart engine model should be readable");

        assert!(
            x_window.is_some(),
            "plot pan should still use the primary x axis"
        );
        assert_eq!(
            left_window, None,
            "plot pan should not create a window for the primary left y axis after hovering the right y band"
        );
        assert!(
            right_window.is_some(),
            "plot pan should use the last hovered right y-axis band"
        );
    }

    #[test]
    fn chart_canvas_panel_plot_pan_primary_axes_skip_hidden_series_on_declarative_path() {
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
        let spec = multi_axis_spec();
        let x_axis = AxisId::new(1);
        let y_left = AxisId::new(2);
        let y_right = AxisId::new(3);

        let engine: Model<ChartEngine> = app
            .models_mut()
            .insert(ChartEngine::new(spec.clone()).expect("chart spec should be valid"));
        app.models_mut()
            .update(&engine, |engine| {
                engine.apply_action(Action::SetSeriesVisible {
                    series: SeriesId::new(1),
                    visible: false,
                });
            })
            .expect("chart engine model should exist");

        let render_frame = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            let engine = engine.clone();
            let spec = spec.clone();
            let root = render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "chart-declarative-hidden-primary-axis-panel",
                |cx| {
                    let mut props =
                        ChartCanvasPanelProps::new(spec.clone()).test_id("chart-hidden-primary");
                    props.engine = Some(engine);
                    props.input_map = crate::input_map::ChartInputMap::default();
                    vec![chart_canvas_panel(cx, props)]
                },
            );
            ui.set_root(root);
            ui.layout_all(app, services, bounds, 1.0);
            let mut scene = Scene::default();
            ui.paint_all(app, services, bounds, &mut scene, 1.0);
        };

        render_frame(&mut ui, &mut app, &mut services);
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        render_frame(&mut ui, &mut app, &mut services);

        let layout = app
            .models()
            .read(&engine, |engine| {
                chart_panel_layout_for_engine(engine, bounds, ChartStyle::default())
            })
            .expect("chart engine model should be readable")
            .expect("expected a chart panel layout");
        let start = Point::new(
            Px(layout.plot.origin.x.0 + 0.5 * layout.plot.size.width.0),
            Px(layout.plot.origin.y.0 + 0.5 * layout.plot.size.height.0),
        );
        let end = Point::new(start.x, Px(start.y.0 + 24.0));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: start,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: fret_core::PointerId(13),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert!(
            ui.captured_for(fret_core::PointerId(13)).is_some(),
            "plot pan should capture with the hidden first series"
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                position: end,
                buttons: fret_core::MouseButtons {
                    left: true,
                    right: false,
                    middle: false,
                },
                modifiers: Modifiers::default(),
                pointer_id: fret_core::PointerId(13),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        let changed = app.take_changed_models();
        if !changed.is_empty() {
            assert!(ui.propagate_model_changes(&mut app, &changed));
        }
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                position: end,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: false,
                click_count: 1,
                pointer_id: fret_core::PointerId(13),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let (x_window, left_window, right_window) = app
            .models()
            .read(&engine, |engine| {
                (
                    engine
                        .state()
                        .data_zoom_x
                        .get(&x_axis)
                        .and_then(|state| state.window),
                    engine.state().data_window_y.get(&y_left).copied(),
                    engine.state().data_window_y.get(&y_right).copied(),
                )
            })
            .expect("chart engine model should be readable");

        assert!(
            x_window.is_some(),
            "plot pan should still use the hidden series' shared x axis"
        );
        assert_eq!(
            left_window, None,
            "plot pan should not use the hidden first series' left y-axis as primary"
        );
        assert!(
            right_window.is_some(),
            "plot pan should use the first visible series' right y-axis as primary"
        );
    }
}
