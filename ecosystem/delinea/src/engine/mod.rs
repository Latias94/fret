use fret_core::{Point, Px, Rect, Size};

use crate::action::Action;
use crate::data::DatasetStore;
use crate::engine::interaction::AxisInteractionLocks;
use crate::engine::lod::LodScratch;
use crate::engine::model::{ChartModel, ModelError};
use crate::engine::stages::{
    BarLayoutStage, FilterProcessorStage, MarksStage, NearestXIndexKey, NearestXIndexStage,
    OrdinalIndexStage, StackDimsStage,
};
use crate::ids::{ChartId, GridId, Revision};
use crate::link::{LinkConfig, LinkEvent};
use crate::marks::MarkTree;
use crate::scheduler::{StepResult, WorkBudget};
use crate::selection::BrushSelection2D;
use crate::spec::AxisPointerTrigger;
use crate::stats::EngineStats;
use crate::text::TextMeasurer;
use crate::tooltip::{
    TooltipAxisOutput, TooltipItemOutput, TooltipOutput, TooltipSeriesEntry, TooltipSeriesValue,
};
use crate::transform::stack_base_at_index;
use crate::transform::{RowRange, RowSelection};
use crate::transform_graph::{DataViewStage, TransformGraph};
use crate::view::ViewState;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

pub mod axis;
pub mod bar;
pub mod hit_test;
pub mod interaction;
pub mod lod;
pub mod model;
pub mod stages;
pub mod window;
pub mod window_policy;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChartState {
    pub revision: Revision,
    pub link: LinkConfig,
    pub data_zoom_x: BTreeMap<crate::ids::AxisId, DataZoomXState>,
    pub data_window_y: BTreeMap<crate::ids::AxisId, window::DataWindowY>,
    /// ECharts-like percent windows (0..=100) for axes.
    ///
    /// When present for an axis, the engine derives a value-space window from the effective
    /// data extent and applies it in an order-sensitive way (X before Y within a grid).
    pub axis_percent_windows: BTreeMap<crate::ids::AxisId, (f64, f64)>,
    pub axis_locks: BTreeMap<crate::ids::AxisId, AxisInteractionLocks>,
    pub visual_map_range:
        BTreeMap<crate::ids::VisualMapId, Option<crate::engine::model::VisualMapRange>>,
    pub visual_map_piece_mask: BTreeMap<crate::ids::VisualMapId, Option<u64>>,
    pub hover_px: Option<Point>,
    pub brush_selection_2d: Option<BrushSelection2D>,
    pub dataset_row_ranges: BTreeMap<crate::ids::DatasetId, crate::transform::RowRange>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DataZoomXState {
    pub window: Option<window::DataWindowX>,
    pub filter_mode: crate::spec::FilterMode,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChartOutput {
    pub revision: Revision,
    pub viewport: Option<Rect>,
    pub plot_viewports_by_grid: BTreeMap<GridId, Rect>,
    pub marks: MarkTree,
    pub axis_windows: BTreeMap<crate::ids::AxisId, window::DataWindow>,
    pub link_events: Vec<LinkEvent>,
    pub hover: Option<HoverHit>,
    pub axis_pointer: Option<AxisPointerOutput>,
    pub brush_selection_2d: Option<BrushSelection2D>,
    pub brush_x_row_ranges_by_series: BTreeMap<crate::ids::SeriesId, RowRange>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AxisPointerOutput {
    pub grid: Option<GridId>,
    pub axis_kind: crate::spec::AxisKind,
    pub axis: crate::ids::AxisId,
    pub axis_value: f64,
    pub crosshair_px: Point,
    pub hit: Option<HoverHit>,
    pub shadow_rect_px: Option<Rect>,
    pub tooltip: TooltipOutput,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HoverHit {
    pub series: crate::ids::SeriesId,
    pub data_index: u32,
    pub point_px: Point,
    pub dist2_px: f32,
    pub x_value: f64,
    pub y_value: f64,
}

#[derive(Debug)]
pub enum EngineError {
    #[allow(dead_code)]
    MissingViewport,
    #[allow(dead_code)]
    MissingPlotViewport { grid: GridId },
}

impl core::fmt::Display for EngineError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingViewport => write!(f, "missing viewport"),
            Self::MissingPlotViewport { grid } => {
                write!(f, "missing plot viewport for grid {grid:?}")
            }
        }
    }
}

impl std::error::Error for EngineError {}

pub struct ChartEngine {
    id: ChartId,
    model: ChartModel,
    datasets: DatasetStore,
    state: ChartState,
    output: ChartOutput,
    stats: EngineStats,
    view: ViewState,
    transform_graph: TransformGraph,
    filter_processor_stage: FilterProcessorStage,
    participation: crate::engine::stages::ParticipationState,
    ordinal_index_stage: OrdinalIndexStage,
    nearest_x_index_stage: NearestXIndexStage,
    bar_layout_stage: BarLayoutStage,
    stack_dims_stage: StackDimsStage,
    marks_stage: MarksStage,
    lod_scratch: LodScratch,
    axis_pointer_cache: AxisPointerCache,
    brush_link_cache: BrushLinkCache,
    axis_pointer_link_cache: AxisPointerLinkCache,
    domain_window_link_cache: DomainWindowLinkCache,
}

#[derive(Debug, Default, Clone)]
struct AxisPointerCache {
    last_hover_px: Option<Point>,
    last_marks_rev: Revision,
    hit: Option<HoverHit>,
    output: Option<AxisPointerOutput>,
}

#[derive(Debug, Default, Clone)]
struct BrushLinkCache {
    last_brush: Option<BrushSelection2D>,
}

#[derive(Debug, Default, Clone)]
struct AxisPointerLinkCache {
    last_anchor: Option<crate::link::AxisPointerAnchor>,
}

#[derive(Debug, Default, Clone)]
struct DomainWindowLinkCache {
    last_windows: BTreeMap<crate::ids::AxisId, window::DataWindow>,
}

impl ChartEngine {
    fn emit_domain_window_link_events(&mut self) {
        let mut current = BTreeMap::new();
        for (axis, st) in &self.state.data_zoom_x {
            if let Some(window) = st.window {
                current.insert(*axis, window);
            }
        }
        for (axis, window) in &self.state.data_window_y {
            current.insert(*axis, *window);
        }

        let mut all_axes: BTreeSet<crate::ids::AxisId> = BTreeSet::new();
        all_axes.extend(self.domain_window_link_cache.last_windows.keys().copied());
        all_axes.extend(current.keys().copied());

        for axis in all_axes {
            let prev = self
                .domain_window_link_cache
                .last_windows
                .get(&axis)
                .copied();
            let next = current.get(&axis).copied();
            if prev != next {
                self.output
                    .link_events
                    .push(LinkEvent::DomainWindowChanged { axis, window: next });
            }
        }

        self.domain_window_link_cache.last_windows = current;
    }

    fn emit_axis_pointer_link_event(&mut self) {
        let anchor = self.output.axis_pointer.as_ref().and_then(|o| {
            if o.axis_value.is_finite() {
                Some(crate::link::AxisPointerAnchor {
                    grid: o.grid,
                    axis_kind: o.axis_kind,
                    axis: o.axis,
                    value: o.axis_value,
                })
            } else {
                None
            }
        });

        if self.axis_pointer_link_cache.last_anchor != anchor {
            self.axis_pointer_link_cache.last_anchor = anchor.clone();
            self.output
                .link_events
                .push(LinkEvent::AxisPointerChanged { anchor });
        }
    }

    fn apply_percent_windows_pre_view(&mut self) {
        for (axis, (start, end)) in self.state.axis_percent_windows.clone() {
            let Some(axis_model) = self.model.axes.get(&axis) else {
                continue;
            };
            if axis_model.kind != crate::spec::AxisKind::X {
                continue;
            }

            let Some(extent) =
                self.transform_graph
                    .x_data_extent(&self.model, &self.datasets, &self.state, axis)
            else {
                continue;
            };

            let range = self.axis_range(axis);
            let Some(window) =
                TransformGraph::percent_range_to_value_window(extent, range, start, end)
            else {
                continue;
            };

            let entry = self
                .state
                .data_zoom_x
                .entry(axis)
                .or_insert(DataZoomXState {
                    window: None,
                    filter_mode: crate::spec::FilterMode::Filter,
                });
            if entry.window != Some(window) {
                entry.window = Some(window);
            }
        }
    }
    pub fn new(spec: crate::spec::ChartSpec) -> Result<Self, ModelError> {
        let id = spec.id;
        let model = ChartModel::from_spec(spec)?;
        let mut state = ChartState::default();
        for (axis, zoom_id) in &model.data_zoom_x_by_axis {
            let filter_mode = model
                .data_zoom_x
                .get(zoom_id)
                .map(|z| z.filter_mode)
                .unwrap_or_default();
            state.data_zoom_x.insert(
                *axis,
                DataZoomXState {
                    window: None,
                    filter_mode,
                },
            );
        }
        let mut engine = Self {
            id,
            model,
            datasets: DatasetStore::default(),
            state,
            output: ChartOutput::default(),
            stats: EngineStats::default(),
            view: ViewState::default(),
            transform_graph: TransformGraph::default(),
            filter_processor_stage: FilterProcessorStage::default(),
            participation: crate::engine::stages::ParticipationState::default(),
            ordinal_index_stage: OrdinalIndexStage::default(),
            nearest_x_index_stage: NearestXIndexStage::default(),
            bar_layout_stage: BarLayoutStage::default(),
            stack_dims_stage: StackDimsStage::default(),
            marks_stage: MarksStage::default(),
            lod_scratch: LodScratch::default(),
            axis_pointer_cache: AxisPointerCache::default(),
            brush_link_cache: BrushLinkCache::default(),
            axis_pointer_link_cache: AxisPointerLinkCache::default(),
            domain_window_link_cache: DomainWindowLinkCache::default(),
        };
        engine.init_visual_map_state();
        Ok(engine)
    }

    pub fn id(&self) -> ChartId {
        self.id
    }

    pub fn model(&self) -> &ChartModel {
        &self.model
    }

    pub fn model_mut(&mut self) -> &mut ChartModel {
        &mut self.model
    }

    pub fn datasets_mut(&mut self) -> &mut DatasetStore {
        &mut self.datasets
    }

    pub fn state(&self) -> &ChartState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut ChartState {
        &mut self.state
    }

    pub fn output(&self) -> &ChartOutput {
        &self.output
    }

    pub fn drain_link_events(&mut self) -> Vec<LinkEvent> {
        std::mem::take(&mut self.output.link_events)
    }

    pub fn view(&self) -> &ViewState {
        &self.view
    }

    pub fn participation(&self) -> &crate::engine::stages::ParticipationState {
        &self.participation
    }

    pub fn filter_plan_output(&self) -> &crate::engine::stages::FilterPlanOutput {
        self.transform_graph.filter_plan_output()
    }

    pub fn stats(&self) -> &EngineStats {
        &self.stats
    }

    pub fn apply_action(&mut self, action: Action) {
        match action {
            Action::HoverAt { point } => {
                self.state.hover_px = Some(point);
            }
            Action::SetBrushSelection2D {
                x_axis,
                y_axis,
                x,
                y,
            } => {
                let grid_x = self.model.axes.get(&x_axis).map(|a| a.grid);
                let grid_y = self.model.axes.get(&y_axis).map(|a| a.grid);
                if grid_x.is_none() || grid_x != grid_y {
                    self.state.brush_selection_2d = None;
                    return;
                }

                let next = BrushSelection2D {
                    grid: grid_x,
                    x_axis,
                    y_axis,
                    x,
                    y,
                };
                if self.state.brush_selection_2d != Some(next) {
                    self.state.brush_selection_2d = Some(next);
                }
            }
            Action::ClearBrushSelection => {
                self.state.brush_selection_2d = None;
            }
            Action::ToggleAxisPanLock { axis } => {
                let entry =
                    crate::engine::interaction::lock_entry(&mut self.state.axis_locks, axis);
                entry.toggle_pan();
                self.state.revision.bump();
            }
            Action::ToggleAxisZoomLock { axis } => {
                let entry =
                    crate::engine::interaction::lock_entry(&mut self.state.axis_locks, axis);
                entry.toggle_zoom();
                self.state.revision.bump();
            }
            Action::PanDataWindowXFromBase {
                axis,
                base,
                delta_px,
                viewport_span_px,
            } => {
                self.state.axis_percent_windows.remove(&axis);
                self.apply_pan_from_base(axis, base, delta_px, viewport_span_px);
            }
            Action::PanDataWindowYFromBase {
                axis,
                base,
                delta_px,
                viewport_span_px,
            } => {
                self.state.axis_percent_windows.remove(&axis);
                self.apply_pan_from_base(axis, base, delta_px, viewport_span_px);
            }
            Action::ZoomDataWindowXFromBase {
                axis,
                base,
                center_px,
                log2_scale,
                viewport_span_px,
            } => {
                self.state.axis_percent_windows.remove(&axis);
                self.apply_zoom_from_base(axis, base, center_px, log2_scale, viewport_span_px);
            }
            Action::ZoomDataWindowYFromBase {
                axis,
                base,
                center_px,
                log2_scale,
                viewport_span_px,
            } => {
                self.state.axis_percent_windows.remove(&axis);
                self.apply_zoom_from_base(axis, base, center_px, log2_scale, viewport_span_px);
            }
            Action::SetDataWindowXFromZoom {
                axis,
                base,
                window,
                anchor,
            } => {
                self.state.axis_percent_windows.remove(&axis);
                self.apply_zoom_set_window(axis, base, window, anchor);
            }
            Action::SetDataWindowYFromZoom {
                axis,
                base,
                window,
                anchor,
            } => {
                self.state.axis_percent_windows.remove(&axis);
                self.apply_zoom_set_window(axis, base, window, anchor);
            }
            Action::SetDataWindowX { axis, window } => {
                self.state.axis_percent_windows.remove(&axis);
                let range = self.axis_range(axis);
                let default_mode = self
                    .model
                    .data_zoom_x_by_axis
                    .get(&axis)
                    .and_then(|id| self.model.data_zoom_x.get(id))
                    .map(|z| z.filter_mode)
                    .unwrap_or_default();
                let entry = self
                    .state
                    .data_zoom_x
                    .entry(axis)
                    .or_insert(DataZoomXState {
                        window: None,
                        filter_mode: default_mode,
                    });
                if let Some(mut window) = window {
                    window.clamp_non_degenerate();
                    window = window.apply_constraints(range.locked_min(), range.locked_max());
                    entry.window = Some(window);
                } else {
                    entry.window = None;
                }
                self.state.revision.bump();
                self.marks_stage.mark_dirty();
            }
            Action::SetDataWindowY { axis, window } => {
                self.state.axis_percent_windows.remove(&axis);
                let range = self.axis_range(axis);
                if let Some(mut window) = window {
                    window.clamp_non_degenerate();
                    window = window.apply_constraints(range.locked_min(), range.locked_max());
                    self.state.data_window_y.insert(axis, window);
                } else {
                    self.state.data_window_y.remove(&axis);
                }
                self.state.revision.bump();
                self.marks_stage.mark_dirty();
            }
            Action::SetAxisWindowPercent { axis, range } => {
                if let Some((a, b)) = range {
                    if a.is_finite() && b.is_finite() {
                        self.state.axis_percent_windows.insert(axis, (a, b));
                        if let Some(axis_model) = self.model.axes.get(&axis) {
                            match axis_model.kind {
                                crate::spec::AxisKind::X => {
                                    if let Some(st) = self.state.data_zoom_x.get_mut(&axis) {
                                        st.window = None;
                                    }
                                }
                                crate::spec::AxisKind::Y => {
                                    self.state.data_window_y.remove(&axis);
                                }
                            }
                        }
                    } else {
                        self.state.axis_percent_windows.remove(&axis);
                    }
                } else {
                    self.state.axis_percent_windows.remove(&axis);
                }
                self.state.revision.bump();
                self.marks_stage.mark_dirty();
            }
            Action::SetDataWindowXFilterMode { axis, mode } => {
                let default_mode = self
                    .model
                    .data_zoom_x_by_axis
                    .get(&axis)
                    .and_then(|id| self.model.data_zoom_x.get(id))
                    .map(|z| z.filter_mode)
                    .unwrap_or_default();
                let entry = self
                    .state
                    .data_zoom_x
                    .entry(axis)
                    .or_insert(DataZoomXState {
                        window: None,
                        filter_mode: default_mode,
                    });
                entry.filter_mode = mode.unwrap_or(default_mode);
                self.state.revision.bump();
                self.marks_stage.mark_dirty();
            }
            Action::SetViewWindow2D {
                x_axis,
                y_axis,
                x,
                y,
            } => {
                let mut changed = false;

                self.state.axis_percent_windows.remove(&x_axis);
                self.state.axis_percent_windows.remove(&y_axis);

                if !self.axis_is_fixed(x_axis) && !self.axis_locks(x_axis).zoom_locked {
                    let x_range = self.axis_range(x_axis);
                    let default_mode = self
                        .model
                        .data_zoom_x_by_axis
                        .get(&x_axis)
                        .and_then(|id| self.model.data_zoom_x.get(id))
                        .map(|z| z.filter_mode)
                        .unwrap_or_default();
                    let entry = self
                        .state
                        .data_zoom_x
                        .entry(x_axis)
                        .or_insert(DataZoomXState {
                            window: None,
                            filter_mode: default_mode,
                        });

                    let next = x.map(|mut w| {
                        w.clamp_non_degenerate();
                        w.apply_constraints(x_range.locked_min(), x_range.locked_max())
                    });
                    if entry.window != next {
                        entry.window = next;
                        changed = true;
                    }
                }

                if !self.axis_is_fixed(y_axis) && !self.axis_locks(y_axis).zoom_locked {
                    let y_range = self.axis_range(y_axis);
                    let next = y.map(|mut w| {
                        w.clamp_non_degenerate();
                        w.apply_constraints(y_range.locked_min(), y_range.locked_max())
                    });

                    match next {
                        Some(w) => {
                            if self.state.data_window_y.get(&y_axis).copied() != Some(w) {
                                self.state.data_window_y.insert(y_axis, w);
                                changed = true;
                            }
                        }
                        None => {
                            if self.state.data_window_y.remove(&y_axis).is_some() {
                                changed = true;
                            }
                        }
                    }
                }

                if changed {
                    self.state.revision.bump();
                    self.marks_stage.mark_dirty();
                }
            }
            Action::SetViewWindow2DFromZoom {
                x_axis,
                y_axis,
                base_x,
                base_y,
                x,
                y,
            } => {
                let mut changed = false;

                self.state.axis_percent_windows.remove(&x_axis);
                self.state.axis_percent_windows.remove(&y_axis);

                if !self.axis_is_fixed(x_axis) && !self.axis_locks(x_axis).zoom_locked {
                    let x_range = self.axis_range(x_axis);
                    let base_x = {
                        let mut base_x = base_x;
                        base_x.clamp_non_degenerate();
                        base_x.apply_constraints(x_range.locked_min(), x_range.locked_max())
                    };

                    let (min_value_span, max_value_span) = self.axis_zoom_span_limits(x_axis);
                    let default_mode = self
                        .model
                        .data_zoom_x_by_axis
                        .get(&x_axis)
                        .and_then(|id| self.model.data_zoom_x.get(id))
                        .map(|z| z.filter_mode)
                        .unwrap_or_default();

                    let next = x.map(|mut w| {
                        w.clamp_non_degenerate();
                        let mut w = w.apply_constraints(x_range.locked_min(), x_range.locked_max());
                        w = w.apply_span_limits_from_base(
                            base_x,
                            min_value_span,
                            max_value_span,
                            window::WindowSpanAnchor::Center,
                        );
                        w.apply_constraints(x_range.locked_min(), x_range.locked_max())
                    });

                    let entry = self
                        .state
                        .data_zoom_x
                        .entry(x_axis)
                        .or_insert(DataZoomXState {
                            window: None,
                            filter_mode: default_mode,
                        });
                    if entry.window != next {
                        entry.window = next;
                        changed = true;
                    }
                }

                if !self.axis_is_fixed(y_axis) && !self.axis_locks(y_axis).zoom_locked {
                    let y_range = self.axis_range(y_axis);
                    let base_y = {
                        let mut base_y = base_y;
                        base_y.clamp_non_degenerate();
                        base_y.apply_constraints(y_range.locked_min(), y_range.locked_max())
                    };

                    let (min_value_span, max_value_span) = self.axis_zoom_span_limits_y(y_axis);

                    let next = y.map(|mut w| {
                        w.clamp_non_degenerate();
                        let mut w = w.apply_constraints(y_range.locked_min(), y_range.locked_max());
                        w = w.apply_span_limits_from_base(
                            base_y,
                            min_value_span,
                            max_value_span,
                            window::WindowSpanAnchor::Center,
                        );
                        w.apply_constraints(y_range.locked_min(), y_range.locked_max())
                    });

                    match next {
                        Some(w) => {
                            if self.state.data_window_y.get(&y_axis).copied() != Some(w) {
                                self.state.data_window_y.insert(y_axis, w);
                                changed = true;
                            }
                        }
                        None => {
                            if self.state.data_window_y.remove(&y_axis).is_some() {
                                changed = true;
                            }
                        }
                    }
                }

                if changed {
                    self.state.revision.bump();
                    self.marks_stage.mark_dirty();
                }
            }
            Action::SetVisualMapRange { visual_map, range } => {
                let range = sanitize_range_option(range);
                self.state.visual_map_range.insert(visual_map, range);
                self.state.revision.bump();
                self.marks_stage.mark_dirty();
            }
            Action::SetVisualMapPieceMask { visual_map, mask } => {
                let mask = mask.and_then(|m| {
                    let Some(vm) = self.model.visual_maps.get(&visual_map).copied() else {
                        return Some(m);
                    };
                    let buckets = vm.buckets.clamp(1, 64) as u32;
                    let full_mask = if buckets >= 64 {
                        u64::MAX
                    } else {
                        (1u64 << buckets) - 1
                    };
                    let m = m & full_mask;
                    (m != full_mask).then_some(m)
                });
                self.state.visual_map_piece_mask.insert(visual_map, mask);
                self.state.revision.bump();
                self.marks_stage.mark_dirty();
            }
            Action::SetLinkGroup { group } => {
                self.state.link.group = group;
                self.state.revision.bump();
            }
            Action::SetLinkBrushXExportPolicy { policy } => {
                self.state.link.brush_x_export_policy = policy;
                self.state.revision.bump();
            }
            Action::SetSeriesVisible { series, visible } => {
                if let Some(existing) = self.model.series.get_mut(&series)
                    && existing.visible != visible
                {
                    existing.visible = visible;
                    self.model.revs.bump_visual();
                    self.marks_stage.mark_dirty();
                }
                self.state.revision.bump();
            }
            Action::SetSeriesVisibility { updates } => {
                let mut changed = false;
                for (series, visible) in updates {
                    if let Some(existing) = self.model.series.get_mut(&series)
                        && existing.visible != visible
                    {
                        existing.visible = visible;
                        changed = true;
                    }
                }
                if changed {
                    self.model.revs.bump_visual();
                    self.marks_stage.mark_dirty();
                }
                self.state.revision.bump();
            }
            Action::SetDatasetRowRange { dataset, range } => {
                if let Some(mut range) = range {
                    range.clamp_to_len(usize::MAX);
                    self.state.dataset_row_ranges.insert(dataset, range);
                } else {
                    self.state.dataset_row_ranges.remove(&dataset);
                }
                self.state.revision.bump();
                self.marks_stage.mark_dirty();
            }
        }
    }

    fn init_visual_map_state(&mut self) {
        for (id, map) in &self.model.visual_maps {
            self.state.visual_map_range.insert(*id, map.initial_range);
            self.state
                .visual_map_piece_mask
                .insert(*id, map.initial_piece_mask);
        }
    }

    fn axis_locks(&self, axis: crate::ids::AxisId) -> AxisInteractionLocks {
        self.state
            .axis_locks
            .get(&axis)
            .copied()
            .unwrap_or_default()
    }

    fn axis_range(&self, axis: crate::ids::AxisId) -> crate::spec::AxisRange {
        self.model
            .axes
            .get(&axis)
            .map(|a| a.range)
            .unwrap_or_default()
    }

    fn axis_is_fixed(&self, axis: crate::ids::AxisId) -> bool {
        self.axis_range(axis).is_fixed()
    }

    fn axis_zoom_span_limits(&self, axis: crate::ids::AxisId) -> (Option<f64>, Option<f64>) {
        self.model
            .data_zoom_x_by_axis
            .get(&axis)
            .and_then(|id| self.model.data_zoom_x.get(id))
            .map(|z| (z.min_value_span, z.max_value_span))
            .unwrap_or((None, None))
    }

    fn axis_zoom_span_limits_y(&self, axis: crate::ids::AxisId) -> (Option<f64>, Option<f64>) {
        self.model
            .data_zoom_y_by_axis
            .get(&axis)
            .and_then(|id| self.model.data_zoom_y.get(id))
            .map(|z| (z.min_value_span, z.max_value_span))
            .unwrap_or((None, None))
    }

    fn apply_pan_from_base(
        &mut self,
        axis: crate::ids::AxisId,
        mut base: window::DataWindow,
        delta_px: f32,
        viewport_span_px: f32,
    ) {
        let Some(axis_model) = self.model.axes.get(&axis) else {
            return;
        };
        if self.axis_is_fixed(axis) {
            return;
        }
        if self.axis_locks(axis).pan_locked {
            return;
        }

        base.clamp_non_degenerate();
        let mut window = base.pan_by_px(delta_px, viewport_span_px);

        let range = self.axis_range(axis);
        window = window.apply_constraints(range.locked_min(), range.locked_max());

        match axis_model.kind {
            crate::spec::AxisKind::X => {
                let default_mode = self
                    .model
                    .data_zoom_x_by_axis
                    .get(&axis)
                    .and_then(|id| self.model.data_zoom_x.get(id))
                    .map(|z| z.filter_mode)
                    .unwrap_or_default();
                self.state
                    .data_zoom_x
                    .entry(axis)
                    .or_insert(DataZoomXState {
                        window: None,
                        filter_mode: default_mode,
                    })
                    .window = Some(window);
            }
            crate::spec::AxisKind::Y => {
                self.state.data_window_y.insert(axis, window);
            }
        }

        self.state.revision.bump();
        self.marks_stage.mark_dirty();
    }

    fn apply_zoom_from_base(
        &mut self,
        axis: crate::ids::AxisId,
        mut base: window::DataWindow,
        center_px: f32,
        log2_scale: f32,
        viewport_span_px: f32,
    ) {
        let Some(axis_model) = self.model.axes.get(&axis) else {
            return;
        };
        if self.axis_is_fixed(axis) {
            return;
        }
        if self.axis_locks(axis).zoom_locked {
            return;
        }

        let range = self.axis_range(axis);
        base.clamp_non_degenerate();
        base = base.apply_constraints(range.locked_min(), range.locked_max());

        let mut window = base.zoom_by_px(center_px, log2_scale, viewport_span_px);
        window = window.apply_constraints(range.locked_min(), range.locked_max());

        match axis_model.kind {
            crate::spec::AxisKind::X => {
                let (min_value_span, max_value_span) = self.axis_zoom_span_limits(axis);
                window = window.apply_span_limits_from_base(
                    base,
                    min_value_span,
                    max_value_span,
                    window::WindowSpanAnchor::Center,
                );
                window = window.apply_constraints(range.locked_min(), range.locked_max());
            }
            crate::spec::AxisKind::Y => {
                let (min_value_span, max_value_span) = self.axis_zoom_span_limits_y(axis);
                window = window.apply_span_limits_from_base(
                    base,
                    min_value_span,
                    max_value_span,
                    window::WindowSpanAnchor::Center,
                );
                window = window.apply_constraints(range.locked_min(), range.locked_max());
            }
        }

        match axis_model.kind {
            crate::spec::AxisKind::X => {
                let default_mode = self
                    .model
                    .data_zoom_x_by_axis
                    .get(&axis)
                    .and_then(|id| self.model.data_zoom_x.get(id))
                    .map(|z| z.filter_mode)
                    .unwrap_or_default();
                self.state
                    .data_zoom_x
                    .entry(axis)
                    .or_insert(DataZoomXState {
                        window: None,
                        filter_mode: default_mode,
                    })
                    .window = Some(window);
            }
            crate::spec::AxisKind::Y => {
                self.state.data_window_y.insert(axis, window);
            }
        }

        self.state.revision.bump();
        self.marks_stage.mark_dirty();
    }

    fn apply_zoom_set_window(
        &mut self,
        axis: crate::ids::AxisId,
        mut base: window::DataWindow,
        mut window: window::DataWindow,
        anchor: window::WindowSpanAnchor,
    ) {
        let Some(axis_model) = self.model.axes.get(&axis) else {
            return;
        };
        if self.axis_is_fixed(axis) {
            return;
        }
        if self.axis_locks(axis).zoom_locked {
            return;
        }

        let range = self.axis_range(axis);

        base.clamp_non_degenerate();
        base = base.apply_constraints(range.locked_min(), range.locked_max());

        window.clamp_non_degenerate();
        window = window.apply_constraints(range.locked_min(), range.locked_max());

        match axis_model.kind {
            crate::spec::AxisKind::X => {
                let (min_value_span, max_value_span) = self.axis_zoom_span_limits(axis);
                window = window.apply_span_limits_from_base(
                    base,
                    min_value_span,
                    max_value_span,
                    anchor,
                );
                window = window.apply_constraints(range.locked_min(), range.locked_max());
            }
            crate::spec::AxisKind::Y => {
                let (min_value_span, max_value_span) = self.axis_zoom_span_limits_y(axis);
                window = window.apply_span_limits_from_base(
                    base,
                    min_value_span,
                    max_value_span,
                    anchor,
                );
                window = window.apply_constraints(range.locked_min(), range.locked_max());
            }
        }

        match axis_model.kind {
            crate::spec::AxisKind::X => {
                let default_mode = self
                    .model
                    .data_zoom_x_by_axis
                    .get(&axis)
                    .and_then(|id| self.model.data_zoom_x.get(id))
                    .map(|z| z.filter_mode)
                    .unwrap_or_default();
                self.state
                    .data_zoom_x
                    .entry(axis)
                    .or_insert(DataZoomXState {
                        window: None,
                        filter_mode: default_mode,
                    })
                    .window = Some(window);
            }
            crate::spec::AxisKind::Y => {
                self.state.data_window_y.insert(axis, window);
            }
        }

        self.state.revision.bump();
        self.marks_stage.mark_dirty();
    }

    pub fn apply_patch(
        &mut self,
        patch: crate::engine::model::ChartPatch,
        mode: crate::engine::model::PatchMode,
    ) -> Result<crate::engine::model::PatchReport, ModelError> {
        let report = self.model.apply_patch(patch, mode)?;
        if report.viewport_changed || report.structure_changed || report.marks_changed {
            self.marks_stage.mark_dirty();
        }
        Ok(report)
    }

    pub fn step(
        &mut self,
        _measurer: &mut dyn TextMeasurer,
        mut budget: WorkBudget,
    ) -> Result<StepResult, EngineError> {
        self.output.viewport = self.model.viewport;
        self.output.plot_viewports_by_grid.clear();
        let grid_ids: Vec<GridId> = self.model.grids.keys().copied().collect();
        if grid_ids.is_empty() {
            if self.output.viewport.is_none() {
                return Err(EngineError::MissingViewport);
            }
        } else {
            for grid in &grid_ids {
                if let Some(vp) = self.model.plot_viewports_by_grid.get(grid).copied() {
                    self.output.plot_viewports_by_grid.insert(*grid, vp);
                }
            }

            if self.output.plot_viewports_by_grid.len() != grid_ids.len() {
                let Some(chart_viewport) = self.output.viewport else {
                    for grid in &grid_ids {
                        if !self.output.plot_viewports_by_grid.contains_key(grid) {
                            return Err(EngineError::MissingPlotViewport { grid: *grid });
                        }
                    }
                    unreachable!("covered by per-grid viewport check");
                };

                let grid_count = grid_ids.len().max(1);
                let origin = chart_viewport.origin;
                let total_w = chart_viewport.size.width.0.max(0.0);
                let total_h = chart_viewport.size.height.0.max(0.0);
                let cell_h = if grid_count == 0 {
                    total_h
                } else {
                    total_h / (grid_count as f32)
                };

                for (i, grid) in grid_ids.iter().copied().enumerate() {
                    if self.output.plot_viewports_by_grid.contains_key(&grid) {
                        continue;
                    }
                    let y = origin.y.0 + cell_h * (i as f32);
                    let h = if i + 1 == grid_count {
                        (origin.y.0 + total_h - y).max(0.0)
                    } else {
                        cell_h.max(0.0)
                    };
                    let rect =
                        Rect::new(Point::new(origin.x, Px(y)), Size::new(Px(total_w), Px(h)));
                    self.output.plot_viewports_by_grid.insert(grid, rect);
                }
            }
        }

        if self.output.viewport.is_none() && grid_ids.len() == 1 {
            let grid = grid_ids[0];
            self.output.viewport = self.output.plot_viewports_by_grid.get(&grid).copied();
        }
        if self.output.viewport.is_none() && self.output.plot_viewports_by_grid.is_empty() {
            return Err(EngineError::MissingViewport);
        }

        // Apply percent window inputs early so the view rebuild and data-view requests can observe
        // the derived X value windows (ECharts-class dataZoomProcessor ordering scaffold).
        self.apply_percent_windows_pre_view();

        if self.state.link.group.is_some() {
            self.emit_domain_window_link_events();
        }

        self.output.brush_selection_2d = self.state.brush_selection_2d;
        self.output.brush_x_row_ranges_by_series.clear();

        let view_changed = self
            .view
            .sync_inputs(&self.model, &self.datasets, &self.state);
        if view_changed {
            self.view.rebuild(&self.model, &self.datasets, &self.state);
        }
        if self.state.link.group.is_some()
            && self.brush_link_cache.last_brush != self.state.brush_selection_2d
        {
            self.brush_link_cache.last_brush = self.state.brush_selection_2d;
            self.output
                .link_events
                .push(LinkEvent::BrushSelectionChanged {
                    selection: self.state.brush_selection_2d,
                });
        }

        let hover_px = self.state.hover_px;

        self.stack_dims_stage.begin_frame();
        self.stack_dims_stage
            .request_for_visible_stacks(&self.model);
        self.stack_dims_stage
            .prepare_requests(&self.model, &self.datasets);
        let stack_dims_done = self
            .stack_dims_stage
            .step(&self.model, &self.datasets, &mut budget);

        self.bar_layout_stage.begin_frame();
        self.bar_layout_stage.request_for_visible_bars(&self.model);
        self.bar_layout_stage.prepare_requests();
        let bar_layout_done = self
            .bar_layout_stage
            .step(&self.model, &self.datasets, &mut budget);

        self.transform_graph.begin_frame();
        self.filter_processor_stage.request_data_views(
            &self.model,
            &self.datasets,
            &self.state,
            &self.view,
            &mut self.transform_graph,
        );
        self.transform_graph.prepare_requests(&self.datasets);
        let selection_done =
            self.transform_graph
                .step(&self.model, &self.datasets, &self.view, &mut budget);

        // Multi-dimensional `weakFilter` (v1 subset) is materialized as an indices-backed selection.
        // Apply the cached selection when available so all downstream consumers observe the correct
        // row participation contract.
        let filter_result = self.filter_processor_stage.apply(
            &mut self.transform_graph,
            &self.model,
            &self.datasets,
            &mut self.state,
            &mut self.view,
        );
        let xy_weak_filter_pending = filter_result.xy_weak_filter_pending;

        self.stats.filter_plan_runs += 1;
        self.stats.filter_plan_grids += filter_result.plan_grids as u64;
        self.stats.filter_plan_steps_run += filter_result.plan_steps_run as u64;
        self.stats.filter_xy_weakfilter_applied_series +=
            filter_result.xy_weak_filter_applied_series as u64;
        self.stats.filter_xy_weakfilter_pending_series +=
            filter_result.xy_weak_filter_pending_series as u64;
        self.stats.filter_xy_weakfilter_skipped_view_len_cap_series +=
            filter_result.xy_weak_filter_skipped_view_len_cap_series as u64;
        self.stats.filter_x_indices_applied_series += filter_result.x_indices_applied_series as u64;
        self.stats.filter_y_indices_applied_series += filter_result.y_indices_applied_series as u64;
        self.stats.filter_y_indices_skipped_view_len_cap_series +=
            filter_result.y_indices_skipped_view_len_cap_series as u64;
        self.stats
            .filter_y_indices_skipped_indices_scan_avoid_series +=
            filter_result.y_indices_skipped_indices_scan_avoid_series as u64;

        self.participation
            .rebuild_from_plan_output(&self.model, self.transform_graph.filter_plan_output());

        self.ordinal_index_stage.begin_frame();
        if self
            .model
            .axis_pointer
            .as_ref()
            .is_some_and(|p| p.enabled && p.trigger == AxisPointerTrigger::Axis)
        {
            request_ordinal_indices_for_axis_pointer(
                &mut self.ordinal_index_stage,
                &self.model,
                &self.datasets,
                &self.participation,
            );
        }
        self.ordinal_index_stage.prepare_requests(&self.datasets);
        let ordinal_indices_done = self.ordinal_index_stage.step(&self.datasets, &mut budget);

        self.nearest_x_index_stage.begin_frame();
        if hover_px.is_some()
            && self
                .model
                .axis_pointer
                .as_ref()
                .is_some_and(|p| p.enabled && p.trigger == AxisPointerTrigger::Axis)
        {
            request_nearest_x_indices_for_axis_pointer(
                &mut self.nearest_x_index_stage,
                &self.model,
                &self.datasets,
                &self.participation,
            );
        }
        self.nearest_x_index_stage.prepare_requests(&self.datasets);
        let nearest_x_done = self.nearest_x_index_stage.step(&self.datasets, &mut budget);

        // Brush selection is an output-only interaction (ADR 1144). We compute the derived X-only
        // row range output after the participation contract has been updated so the output is
        // scoped to the effective series view (base range + X dataZoom + optional indices views).
        if let Some(brush) = self.state.brush_selection_2d {
            let mut x_link_keys: BTreeSet<(crate::ids::DatasetId, crate::ids::FieldId)> =
                BTreeSet::new();
            if self.state.link.brush_x_export_policy
                == crate::link::BrushXExportPolicy::SameDatasetXField
            {
                for series_id in &self.model.series_order {
                    let Some(series) = self.model.series.get(series_id) else {
                        continue;
                    };
                    if !series.visible {
                        continue;
                    }
                    if series.x_axis != brush.x_axis || series.y_axis != brush.y_axis {
                        continue;
                    }
                    x_link_keys.insert((series.dataset, series.encode.x));
                }
            }

            for series_id in &self.model.series_order {
                let Some(series) = self.model.series.get(series_id) else {
                    continue;
                };
                if !series.visible {
                    continue;
                }

                match self.state.link.brush_x_export_policy {
                    crate::link::BrushXExportPolicy::AxisPairOnly => {
                        if series.x_axis != brush.x_axis || series.y_axis != brush.y_axis {
                            continue;
                        }
                    }
                    crate::link::BrushXExportPolicy::SameDatasetXField => {
                        if series.x_axis != brush.x_axis || series.y_axis != brush.y_axis {
                            let key = (series.dataset, series.encode.x);
                            if !x_link_keys.contains(&key) {
                                continue;
                            }
                        }
                    }
                }

                let Some(participation) = self.participation.series_participation(*series_id)
                else {
                    continue;
                };
                let RowSelection::Range(base_range) = participation.selection else {
                    continue;
                };

                let Some(dataset_model) = self.model.datasets.get(&series.dataset) else {
                    continue;
                };
                let Some(x_col) = dataset_model.fields.get(&series.encode.x).copied() else {
                    continue;
                };
                let Some(table) = self
                    .datasets
                    .dataset(self.model.root_dataset_id(series.dataset))
                else {
                    continue;
                };
                let Some(x_values) = table.column_f64(x_col) else {
                    continue;
                };

                let range = crate::transform::row_range_for_x_window(x_values, base_range, brush.x);
                self.output
                    .brush_x_row_ranges_by_series
                    .insert(*series_id, range);
            }
        }

        self.marks_stage
            .sync_inputs(&self.model, &self.datasets, &self.participation);
        let wants_append_rebuild = self.marks_stage.take_append_rebuild();
        if self.marks_stage.is_dirty() {
            self.output.marks.clear();
            self.output.axis_windows.clear();
            self.marks_stage.reset();
        } else if wants_append_rebuild {
            // Append-only updates should not clear marks: keep the previous frame geometry visible
            // while we incrementally extend/rebuild under budget.
            self.marks_stage.begin_append_rebuild();
        }

        self.stats.stage_data_runs += 1;
        self.stats.stage_layout_runs += 1;
        self.stats.stage_visual_runs += 1;
        self.stats.stage_marks_runs += 1;

        let done = if xy_weak_filter_pending {
            false
        } else {
            self.marks_stage.step(
                &self.model,
                &self.datasets,
                &self.state,
                self.transform_graph.data_views(),
                &self.stack_dims_stage,
                &self.bar_layout_stage,
                &self.participation,
                &self.output.plot_viewports_by_grid,
                &mut budget,
                &mut self.lod_scratch,
                &mut self.output.marks,
                &mut self.stats,
            )
        };

        self.output
            .axis_windows
            .clone_from(self.marks_stage.axis_windows());
        for axis in self.model.axes.values() {
            if let crate::scale::AxisScale::Category(scale) = &axis.scale
                && !scale.categories.is_empty()
                && !self.output.axis_windows.contains_key(&axis.id)
            {
                self.output.axis_windows.insert(
                    axis.id,
                    window::DataWindow {
                        min: -0.5,
                        max: scale.categories.len() as f64 - 0.5,
                    },
                );
            }
        }

        let unfinished = xy_weak_filter_pending
            || !done
            || !selection_done
            || !stack_dims_done
            || !ordinal_indices_done
            || !bar_layout_done
            || !nearest_x_done;
        let marks_rev = self.output.marks.revision;
        if self.axis_pointer_cache.last_marks_rev != marks_rev {
            self.axis_pointer_cache.last_marks_rev = marks_rev;
            self.axis_pointer_cache.last_hover_px = None;
            self.axis_pointer_cache.hit = None;
            self.axis_pointer_cache.output = None;
        }

        let axis_pointer = self.model.axis_pointer.as_ref().filter(|p| p.enabled);

        if axis_pointer.is_none() {
            self.axis_pointer_cache.last_hover_px = None;
            self.axis_pointer_cache.hit = None;
            self.axis_pointer_cache.output = None;
        }

        if let Some(hover_px) = hover_px {
            let should_recompute = match axis_pointer {
                Some(spec) => should_recompute_hover(
                    self.axis_pointer_cache.last_hover_px,
                    hover_px,
                    spec.throttle_px,
                ),
                None => false,
            };
            if should_recompute {
                self.axis_pointer_cache.last_hover_px = Some(hover_px);
                self.axis_pointer_cache.hit = None;
                self.axis_pointer_cache.output = None;

                if let Some(spec) = axis_pointer {
                    let raw_hit = hit_test::hover_hit_test(
                        &self.model,
                        &self.datasets,
                        &self.output.marks,
                        hover_px,
                        &self.stack_dims_stage,
                    );

                    let mut hovered_grid_viewport: Option<Rect> = None;
                    if let Some(hit) = raw_hit {
                        if let Some(series) = self.model.series.get(&hit.series)
                            && let Some(x_axis) = self.model.axes.get(&series.x_axis)
                        {
                            if let Some(viewport) = self
                                .output
                                .plot_viewports_by_grid
                                .get(&x_axis.grid)
                                .copied()
                                && rect_contains_point(viewport, hover_px)
                            {
                                hovered_grid_viewport = Some(viewport);
                            }
                        }
                    }
                    if hovered_grid_viewport.is_none() {
                        hovered_grid_viewport = self
                            .output
                            .plot_viewports_by_grid
                            .values()
                            .find(|rect| rect_contains_point(**rect, hover_px))
                            .copied();
                    }

                    if let Some(viewport) = hovered_grid_viewport {
                        match spec.trigger {
                            AxisPointerTrigger::Item => {
                                let output = compute_item_axis_pointer_output(
                                    &self.model,
                                    hover_px,
                                    raw_hit,
                                    spec,
                                );
                                self.axis_pointer_cache.hit = output.as_ref().and_then(|o| o.hit);
                                self.axis_pointer_cache.output = output;
                            }
                            AxisPointerTrigger::Axis => {
                                let output = compute_axis_axis_pointer_output(
                                    &self.model,
                                    &self.datasets,
                                    &self.participation,
                                    self.transform_graph.data_views(),
                                    &self.stack_dims_stage,
                                    &self.ordinal_index_stage,
                                    &self.nearest_x_index_stage,
                                    &self.output.axis_windows,
                                    viewport,
                                    hover_px,
                                    raw_hit,
                                    spec,
                                );
                                self.axis_pointer_cache.hit = output.as_ref().and_then(|o| o.hit);
                                self.axis_pointer_cache.output = output;
                            }
                        }
                    }
                }
            }
        } else {
            self.axis_pointer_cache.last_hover_px = None;
            self.axis_pointer_cache.hit = None;
            self.axis_pointer_cache.output = None;
        }

        self.output.hover = self.axis_pointer_cache.hit;
        self.output.axis_pointer = self.axis_pointer_cache.output.clone();

        if self.state.link.group.is_some() {
            self.emit_axis_pointer_link_event();
        }

        self.output.revision.bump();
        Ok(StepResult { unfinished })
    }
}

fn sanitize_range_option(
    range: Option<(f64, f64)>,
) -> Option<crate::engine::model::VisualMapRange> {
    let (min, max) = range?;
    crate::engine::model::VisualMapRange { min, max }.sanitize()
}

fn rect_contains_point(rect: Rect, point: Point) -> bool {
    let x0 = rect.origin.x.0;
    let y0 = rect.origin.y.0;
    let x1 = x0 + rect.size.width.0;
    let y1 = y0 + rect.size.height.0;
    point.x.0 >= x0 && point.x.0 <= x1 && point.y.0 >= y0 && point.y.0 <= y1
}

fn should_recompute_hover(prev: Option<Point>, next: Point, throttle_px: f32) -> bool {
    let Some(prev) = prev else {
        return true;
    };
    let dx = next.x.0 - prev.x.0;
    let dy = next.y.0 - prev.y.0;
    let dist2 = dx * dx + dy * dy;
    let t = throttle_px.max(0.0);
    dist2 >= t * t
}

fn request_nearest_x_indices_for_axis_pointer(
    stage: &mut NearestXIndexStage,
    model: &ChartModel,
    datasets: &DatasetStore,
    participation: &crate::engine::stages::ParticipationState,
) {
    let Some(primary) = model.series_in_order().find(|s| s.visible) else {
        return;
    };

    let trigger_axis = if primary.kind == crate::spec::SeriesKind::Bar {
        crate::engine::bar::bar_mapping_for_series(model, primary.id)
            .map(|m| m.category_axis)
            .unwrap_or(primary.x_axis)
    } else {
        primary.x_axis
    };

    let Some(axis) = model.axes.get(&trigger_axis) else {
        return;
    };

    // Category axes use ordinal mapping and do not need nearest-X acceleration.
    if matches!(axis.scale, crate::scale::AxisScale::Category(_)) {
        return;
    }
    if axis.kind != crate::spec::AxisKind::X {
        return;
    }

    for series in model.series_in_order() {
        if !series.visible {
            continue;
        }
        if series.kind == crate::spec::SeriesKind::Bar {
            continue;
        }
        if series.x_axis != trigger_axis {
            continue;
        }

        let Some(table) = datasets.dataset(model.root_dataset_id(series.dataset)) else {
            continue;
        };
        let contract = participation.series_contract(series.id, table.row_count);
        let (RowSelection::All | RowSelection::Range(_)) = contract.selection else {
            continue;
        };
        let Some(dataset) = model.datasets.get(&series.dataset) else {
            continue;
        };
        let Some(x_col) = dataset.fields.get(&series.encode.x).copied() else {
            continue;
        };
        let Some(x_values) = table.column_f64(x_col) else {
            continue;
        };

        let selection_range = contract.selection_range;
        let visible_len = selection_range.end.saturating_sub(selection_range.start);
        if visible_len <= MAX_UNSORTED_AXIS_SCAN_POINTS {
            continue;
        }

        if crate::transform::is_probably_monotonic_in_range(x_values, selection_range).is_some() {
            continue;
        }

        let root = model.root_dataset_id(series.dataset);
        stage.request(NearestXIndexKey::new(
            series.dataset,
            root,
            x_col,
            selection_range,
            contract.x_policy.filter,
        ));
    }
}

fn compute_item_axis_pointer_output(
    model: &ChartModel,
    hover_px: Point,
    hit: Option<HoverHit>,
    spec: &crate::engine::model::AxisPointerModel,
) -> Option<AxisPointerOutput> {
    let hit = hit?;
    let trigger2 = spec.trigger_distance_px.max(0.0) * spec.trigger_distance_px.max(0.0);
    if hit.dist2_px > trigger2 {
        return None;
    }

    let crosshair_px = if spec.snap { hit.point_px } else { hover_px };

    let series = model.series.get(&hit.series);
    let (x_axis, y_axis) = series.map(|s| (s.x_axis, s.y_axis)).unwrap_or_default();
    let grid = model
        .axes
        .get(&x_axis)
        .map(|a| a.grid)
        .or_else(|| model.axes.get(&y_axis).map(|a| a.grid));

    let tooltip = TooltipOutput::Item(TooltipItemOutput {
        series: hit.series,
        data_index: hit.data_index,
        x_axis,
        y_axis,
        x_value: hit.x_value,
        y_value: hit.y_value,
    });

    Some(AxisPointerOutput {
        grid,
        axis_kind: crate::spec::AxisKind::X,
        axis: x_axis,
        axis_value: hit.x_value,
        crosshair_px,
        hit: Some(hit),
        shadow_rect_px: None,
        tooltip,
    })
}

fn compute_axis_axis_pointer_output(
    model: &ChartModel,
    datasets: &DatasetStore,
    participation: &crate::engine::stages::ParticipationState,
    data_views: &DataViewStage,
    stack_dims: &StackDimsStage,
    ordinal_indices: &OrdinalIndexStage,
    nearest_x_indices: &NearestXIndexStage,
    axis_windows: &BTreeMap<crate::ids::AxisId, window::DataWindow>,
    viewport: Rect,
    hover_px: Point,
    hit: Option<HoverHit>,
    spec: &crate::engine::model::AxisPointerModel,
) -> Option<AxisPointerOutput> {
    let primary = model.series_in_order().find(|s| s.visible)?;
    let trigger_axis = if primary.kind == crate::spec::SeriesKind::Bar {
        crate::engine::bar::bar_mapping_for_series(model, primary.id)
            .map(|m| m.category_axis)
            .unwrap_or(primary.x_axis)
    } else {
        primary.x_axis
    };
    let trigger_axis_kind = model
        .axes
        .get(&trigger_axis)
        .map(|a| a.kind)
        .unwrap_or(crate::spec::AxisKind::X);
    let grid = model.axes.get(&trigger_axis).map(|a| a.grid);

    let trigger_window = axis_windows.get(&trigger_axis).copied().unwrap_or_default();
    let trigger2 = spec.trigger_distance_px.max(0.0) * spec.trigger_distance_px.max(0.0);

    let axis_value_hover = match trigger_axis_kind {
        crate::spec::AxisKind::X => {
            crate::engine::axis::data_at_x_in_rect(trigger_window, hover_px.x.0, viewport)
        }
        crate::spec::AxisKind::Y => {
            crate::engine::axis::data_at_y_in_rect(trigger_window, hover_px.y.0, viewport)
        }
    };

    let mut axis_value = axis_value_hover;
    if let Some(axis) = model.axes.get(&trigger_axis)
        && let crate::scale::AxisScale::Category(scale) = &axis.scale
        && !scale.is_empty()
        && axis_value.is_finite()
    {
        axis_value =
            (axis_value.round() as isize).clamp(0, scale.len().saturating_sub(1) as isize) as f64;
    }
    let mut hit_for_marker = hit.filter(|h| h.dist2_px <= trigger2);

    if spec.snap {
        if let Some((snapped_axis_value, snapped_hit)) = snap_axis_pointer_to_nearest_sample(
            model,
            datasets,
            participation,
            data_views,
            stack_dims,
            nearest_x_indices,
            axis_windows,
            viewport,
            primary,
            trigger_axis,
            trigger_axis_kind,
            trigger_window,
            axis_value,
            hover_px,
        ) {
            axis_value = snapped_axis_value;

            // For `trigger=Axis`, `trigger_distance_px` only gates whether a marker dot is shown.
            // The tooltip/crosshair remain active and use the snapped axis value regardless.
            if let Some(snapped_hit) = snapped_hit {
                if snapped_hit.dist2_px <= trigger2 {
                    hit_for_marker = Some(snapped_hit);
                } else {
                    hit_for_marker = None;
                }
            } else {
                hit_for_marker = None;
            }
        }
    }
    if !axis_value.is_finite() {
        return None;
    }

    let crosshair_px = if spec.snap {
        match trigger_axis_kind {
            crate::spec::AxisKind::X => Point::new(
                Px(crate::engine::axis::x_px_at_data_in_rect(
                    trigger_window,
                    axis_value,
                    viewport,
                )),
                hover_px.y,
            ),
            crate::spec::AxisKind::Y => Point::new(
                hover_px.x,
                Px(crate::engine::axis::y_px_at_data_in_rect(
                    trigger_window,
                    axis_value,
                    viewport,
                )),
            ),
        }
    } else {
        hover_px
    };

    let shadow_rect_px = match spec.pointer_type {
        crate::spec::AxisPointerType::Shadow => {
            match model.axes.get(&trigger_axis).map(|a| &a.scale) {
                Some(crate::scale::AxisScale::Category(scale)) if !scale.is_empty() => {
                    let idx = (axis_value.round() as isize)
                        .clamp(0, scale.len().saturating_sub(1) as isize)
                        as f64;
                    let edge0 = idx - 0.5;
                    let edge1 = idx + 0.5;

                    match trigger_axis_kind {
                        crate::spec::AxisKind::X => {
                            let x0 = crate::engine::axis::x_px_at_data_in_rect(
                                trigger_window,
                                edge0,
                                viewport,
                            );
                            let x1 = crate::engine::axis::x_px_at_data_in_rect(
                                trigger_window,
                                edge1,
                                viewport,
                            );
                            let left = x0.min(x1);
                            let right = x0.max(x1);
                            let min_x = viewport.origin.x.0;
                            let max_x = viewport.origin.x.0 + viewport.size.width.0;
                            let left = left.clamp(min_x, max_x);
                            let right = right.clamp(min_x, max_x);
                            Some(Rect::new(
                                Point::new(Px(left), viewport.origin.y),
                                Size::new(Px((right - left).max(0.0)), viewport.size.height),
                            ))
                        }
                        crate::spec::AxisKind::Y => {
                            let y0 = crate::engine::axis::y_px_at_data_in_rect(
                                trigger_window,
                                edge0,
                                viewport,
                            );
                            let y1 = crate::engine::axis::y_px_at_data_in_rect(
                                trigger_window,
                                edge1,
                                viewport,
                            );
                            let top = y0.min(y1);
                            let bottom = y0.max(y1);
                            let min_y = viewport.origin.y.0;
                            let max_y = viewport.origin.y.0 + viewport.size.height.0;
                            let top = top.clamp(min_y, max_y);
                            let bottom = bottom.clamp(min_y, max_y);
                            Some(Rect::new(
                                Point::new(viewport.origin.x, Px(top)),
                                Size::new(viewport.size.width, Px((bottom - top).max(0.0))),
                            ))
                        }
                    }
                }
                _ => None,
            }
        }
        crate::spec::AxisPointerType::Line => None,
    };

    let mut tooltip = TooltipAxisOutput {
        axis: trigger_axis,
        axis_kind: trigger_axis_kind,
        axis_value,
        series: Vec::default(),
    };

    let category_len = model.axes.get(&trigger_axis).and_then(|a| match &a.scale {
        crate::scale::AxisScale::Category(scale) => Some(scale.len()),
        _ => None,
    });
    let category_ordinal = category_len.and_then(|len| ordinal_from_value(axis_value, len));

    for series in model.series_in_order() {
        if !series.visible {
            continue;
        }

        let bar_mapping = crate::engine::bar::bar_mapping_for_series(model, series.id)
            .filter(|_| series.kind == crate::spec::SeriesKind::Bar);
        let series_trigger_axis = bar_mapping
            .map(|m| m.category_axis)
            .unwrap_or(series.x_axis);
        if series_trigger_axis != trigger_axis {
            continue;
        }

        let Some(table) = datasets.dataset(model.root_dataset_id(series.dataset)) else {
            continue;
        };
        let Some(dataset) = model.datasets.get(&series.dataset) else {
            continue;
        };
        let x_col = dataset.fields.get(&series.encode.x).copied();
        let y0_col = dataset.fields.get(&series.encode.y).copied();

        let x = x_col.and_then(|c| table.column_f64(c));
        let y0 = y0_col.and_then(|c| table.column_f64(c));

        let y1 = if series.kind == crate::spec::SeriesKind::Band {
            series
                .encode
                .y2
                .and_then(|y2_field| dataset.fields.get(&y2_field).copied())
                .and_then(|y2_col| table.column_f64(y2_col))
        } else {
            None
        };

        let contract = participation.series_contract(series.id, table.row_count);
        let selection_range = contract.selection_range;
        let filter = contract.x_policy.filter;
        let base_selection = contract.selection.clone();
        let empty_mask = contract.empty_mask;

        let selection_for_index = base_selection.clone();
        let filter_for_index = if series_trigger_axis == series.x_axis {
            filter
        } else {
            crate::engine::window_policy::AxisFilter1D::default()
        };

        let root_dataset = model.root_dataset_id(series.dataset);
        let table_view = x_col.map(|x_col| {
            data_views.table_view_for(
                table,
                series.dataset,
                root_dataset,
                x_col,
                selection_range,
                filter,
                base_selection,
            )
        });

        let model_rev = model.revs.marks;
        let table_rev = table.revision;
        let nearest_index = x_col.and_then(|x_col| {
            let key = NearestXIndexKey::new(
                series.dataset,
                root_dataset,
                x_col,
                selection_range,
                filter_for_index,
            );
            nearest_x_indices.items_for(key, table_rev)
        });

        let mut sample: Option<SampledSeriesValue> = None;
        if let (Some(category_len), Some(ordinal)) = (category_len, category_ordinal)
            && !matches!(selection_for_index, RowSelection::Indices(_))
        {
            let ordinal_col = if let Some(mapping) = bar_mapping {
                dataset.fields.get(&mapping.category_field).copied()
            } else {
                x_col
            };
            if let Some(ordinal_col) = ordinal_col {
                let key = crate::engine::stages::OrdinalIndexKey::new(
                    series.dataset,
                    root_dataset,
                    ordinal_col,
                    category_len,
                    selection_range,
                    filter_for_index,
                );
                if let Some(raw_index) =
                    ordinal_indices.raw_index_of_ordinal(key, ordinal, table_rev)
                {
                    if let Some(mapping) = bar_mapping {
                        let allowed = if empty_mask.x_active || empty_mask.y_active {
                            if let (Some(x), Some(y0)) = (x, y0) {
                                empty_mask.allows_raw_index(raw_index, x, y0, None)
                            } else {
                                false
                            }
                        } else {
                            true
                        };

                        if allowed {
                            let value_col = dataset.fields.get(&mapping.value_field).copied();
                            let value = value_col
                                .and_then(|c| table.column_f64(c))
                                .and_then(|v| v.get(raw_index).copied())
                                .unwrap_or(f64::NAN);
                            if value.is_finite() {
                                let value = if let Some(stack) = series.stack {
                                    stack_dims
                                        .stacked_value(
                                            stack, series.id, raw_index, model_rev, table_rev,
                                        )
                                        .unwrap_or(value)
                                } else {
                                    value
                                };
                                sample = Some(SampledSeriesValue {
                                    y0: value,
                                    y1: None,
                                });
                            }
                        }
                    } else if let (Some(y0), Some(x)) = (y0, x) {
                        sample = sample_at_raw_index_with_empty_mask(
                            series.id, empty_mask, model, datasets, stack_dims, model_rev,
                            table_rev, raw_index, x, y0, y1,
                        );
                    }
                }
            }
        }

        let sample = sample.or_else(|| {
            if series.kind == crate::spec::SeriesKind::Bar {
                return None;
            }
            let Some(x) = x else {
                return None;
            };
            let Some(y0) = y0 else {
                return None;
            };
            let Some(table_view) = table_view.as_ref() else {
                return None;
            };
            if series.kind == crate::spec::SeriesKind::Scatter {
                sample_scatter_at_x_view(
                    model,
                    datasets,
                    stack_dims,
                    model_rev,
                    table_rev,
                    series.id,
                    empty_mask,
                    axis_value,
                    x,
                    y0,
                    filter_for_index,
                    table_view,
                    nearest_index,
                )
            } else {
                sample_series_at_x_view(
                    model,
                    datasets,
                    stack_dims,
                    model_rev,
                    table_rev,
                    series.id,
                    empty_mask,
                    axis_value,
                    x,
                    y0,
                    y1,
                    filter_for_index,
                    table_view,
                    nearest_index,
                )
            }
        });

        let value_axis = bar_mapping.map(|m| m.value_axis).unwrap_or(series.y_axis);
        let value = match sample {
            Some(sample) => {
                if let Some(y1) = sample.y1 {
                    TooltipSeriesValue::Range {
                        min: sample.y0,
                        max: y1,
                    }
                } else {
                    TooltipSeriesValue::Scalar(sample.y0)
                }
            }
            None => TooltipSeriesValue::Missing,
        };

        tooltip.series.push(TooltipSeriesEntry {
            series: series.id,
            value_axis,
            value,
        });
    }

    Some(AxisPointerOutput {
        grid,
        axis_kind: trigger_axis_kind,
        axis: trigger_axis,
        axis_value,
        crosshair_px,
        hit: hit_for_marker,
        shadow_rect_px,
        tooltip: TooltipOutput::Axis(tooltip),
    })
}

fn snap_axis_pointer_to_nearest_sample(
    model: &ChartModel,
    datasets: &DatasetStore,
    participation: &crate::engine::stages::ParticipationState,
    data_views: &DataViewStage,
    stack_dims: &StackDimsStage,
    nearest_x_indices: &NearestXIndexStage,
    axis_windows: &BTreeMap<crate::ids::AxisId, window::DataWindow>,
    viewport: Rect,
    primary: &crate::engine::model::SeriesModel,
    trigger_axis: crate::ids::AxisId,
    trigger_axis_kind: crate::spec::AxisKind,
    trigger_window: window::DataWindow,
    axis_value: f64,
    hover_px: Point,
) -> Option<(f64, Option<HoverHit>)> {
    if !axis_value.is_finite() {
        return None;
    }

    // Category axes already effectively snap via ordinal rounding.
    if let Some(axis) = model.axes.get(&trigger_axis)
        && matches!(&axis.scale, crate::scale::AxisScale::Category(_))
    {
        let len = model
            .axes
            .get(&trigger_axis)
            .and_then(|a| match &a.scale {
                crate::scale::AxisScale::Category(scale) => Some(scale.len()),
                _ => None,
            })
            .unwrap_or(0);
        let ord = ordinal_from_value(axis_value, len)?;
        return Some((ord as f64, None));
    }

    match trigger_axis_kind {
        crate::spec::AxisKind::X => snap_axis_pointer_x_to_series(
            model,
            datasets,
            participation,
            data_views,
            stack_dims,
            nearest_x_indices,
            axis_windows,
            viewport,
            primary,
            trigger_window,
            axis_value,
            hover_px,
        ),
        crate::spec::AxisKind::Y => snap_axis_pointer_y_to_series(
            model,
            datasets,
            participation,
            data_views,
            stack_dims,
            axis_windows,
            viewport,
            primary,
            trigger_window,
            axis_value,
            hover_px,
        ),
    }
}

fn snap_axis_pointer_x_to_series(
    model: &ChartModel,
    datasets: &DatasetStore,
    participation: &crate::engine::stages::ParticipationState,
    data_views: &DataViewStage,
    stack_dims: &StackDimsStage,
    nearest_x_indices: &NearestXIndexStage,
    axis_windows: &BTreeMap<crate::ids::AxisId, window::DataWindow>,
    viewport: Rect,
    primary: &crate::engine::model::SeriesModel,
    trigger_window: window::DataWindow,
    axis_value: f64,
    hover_px: Point,
) -> Option<(f64, Option<HoverHit>)> {
    if primary.kind == crate::spec::SeriesKind::Bar {
        return None;
    }

    let root_dataset = model.root_dataset_id(primary.dataset);
    let table = datasets.dataset(root_dataset)?;
    let table_rev = table.revision;
    let model_rev = model.revs.data;

    let dataset = model.datasets.get(&primary.dataset)?;
    let x_col = dataset.fields.get(&primary.encode.x).copied()?;
    let y0_col = dataset.fields.get(&primary.encode.y).copied()?;

    let x = table.column_f64(x_col)?;
    let y0 = table.column_f64(y0_col)?;
    let y1 = primary
        .encode
        .y2
        .and_then(|y2_field| dataset.fields.get(&y2_field).copied())
        .and_then(|y2_col| table.column_f64(y2_col));

    let contract = participation.series_contract(primary.id, table.row_count);
    let selection_range = contract.selection_range;
    let filter = contract.x_policy.filter;
    let base_selection = contract.selection;
    let empty_mask = contract.empty_mask;

    let root_dataset = model.root_dataset_id(primary.dataset);
    let table_view = data_views.table_view_for(
        table,
        primary.dataset,
        root_dataset,
        x_col,
        selection_range,
        filter,
        base_selection,
    );

    let nearest_key = NearestXIndexKey::new(
        primary.dataset,
        root_dataset,
        x_col,
        selection_range,
        filter,
    );
    let nearest_index = nearest_x_indices.items_for(nearest_key, table_rev);
    let (raw_index, x_raw) =
        nearest_raw_index_at_x_view(axis_value, x, filter, &table_view, nearest_index)?;
    let sampled = sample_at_raw_index_with_empty_mask(
        primary.id,
        empty_mask,
        model,
        datasets,
        stack_dims,
        model_rev,
        table_rev,
        raw_index,
        x,
        y0,
        y1.as_deref(),
    )?;

    let y_window = axis_windows
        .get(&primary.y_axis)
        .copied()
        .unwrap_or_default();

    let px_x = crate::engine::axis::x_px_at_data_in_rect(trigger_window, x_raw, viewport);
    let px_y = crate::engine::axis::y_px_at_data_in_rect(y_window, sampled.y0, viewport);
    let point_px = Point::new(Px(px_x), Px(px_y));

    let dx = hover_px.x.0 - point_px.x.0;
    let dy = hover_px.y.0 - point_px.y.0;
    let dist2_px = dx * dx + dy * dy;

    let hit = HoverHit {
        series: primary.id,
        data_index: raw_index as u32,
        point_px,
        dist2_px,
        x_value: x_raw,
        y_value: sampled.y0,
    };

    Some((x_raw, Some(hit)))
}

fn nearest_raw_index_at_x_view(
    x_value: f64,
    x: &[f64],
    x_filter: crate::engine::window_policy::AxisFilter1D,
    table_view: &crate::data::DataTableView<'_>,
    nearest_index: Option<&[crate::engine::stages::NearestXIndexItem]>,
) -> Option<(usize, f64)> {
    if let Some(index) = nearest_index {
        if let Some(hit) =
            crate::engine::stages::nearest_raw_index_in_sorted_x_index(index, x_value)
        {
            if x_filter.contains(hit.1) {
                return Some(hit);
            }
        }
    }

    let view_len = table_view.len();
    if view_len == 0 {
        return None;
    }

    let selection = table_view.selection();
    if let RowSelection::All | RowSelection::Range(_) = selection {
        let len = x.len();
        let (start, end) = match selection {
            RowSelection::All => (0usize, len),
            RowSelection::Range(range) => {
                let r = range.as_std_range(len);
                (r.start, r.end)
            }
            RowSelection::Indices(_) => unreachable!(),
        };

        if start < end
            && crate::transform::is_probably_monotonic_in_range(x, RowRange { start, end })
                .is_some()
        {
            let xs = &x[start..end];
            if xs.len() == 1 {
                let v = xs[0];
                return v.is_finite().then_some((start, v));
            }

            let idx = lower_bound(xs, x_value);
            let i1 = (start + idx).min(end - 1);
            let i0 = i1.saturating_sub(1).max(start);

            let x0 = x.get(i0).copied().unwrap_or(f64::NAN);
            let x1 = x.get(i1).copied().unwrap_or(f64::NAN);
            if !x0.is_finite() || !x1.is_finite() {
                return None;
            }

            let d0 = (x_value - x0).abs();
            let d1 = (x_value - x1).abs();
            let x0_ok = x_filter.contains(x0);
            let x1_ok = x_filter.contains(x1);
            return match (x0_ok, x1_ok) {
                (false, false) => None,
                (false, true) => Some((i1, x1)),
                (true, false) => Some((i0, x0)),
                (true, true) => {
                    if d1 < d0 {
                        Some((i1, x1))
                    } else {
                        Some((i0, x0))
                    }
                }
            };
        }
    }

    if view_len > MAX_UNSORTED_AXIS_SCAN_POINTS {
        return None;
    }

    let mut best_raw_index: Option<usize> = None;
    let mut best_x: f64 = f64::NAN;
    let mut best_dist = f64::INFINITY;

    for view_index in 0..view_len {
        let Some(raw_index) = table_view.get_raw_index(view_index) else {
            continue;
        };
        let x_raw = x.get(raw_index).copied().unwrap_or(f64::NAN);
        if !x_raw.is_finite() {
            continue;
        }
        if !x_filter.contains(x_raw) {
            continue;
        }
        let dist = (x_value - x_raw).abs();
        if dist < best_dist {
            best_dist = dist;
            best_raw_index = Some(raw_index);
            best_x = x_raw;
        }
    }

    best_raw_index.map(|i| (i, best_x))
}

fn snap_axis_pointer_y_to_series(
    model: &ChartModel,
    datasets: &DatasetStore,
    participation: &crate::engine::stages::ParticipationState,
    data_views: &DataViewStage,
    stack_dims: &StackDimsStage,
    axis_windows: &BTreeMap<crate::ids::AxisId, window::DataWindow>,
    viewport: Rect,
    primary: &crate::engine::model::SeriesModel,
    trigger_window: window::DataWindow,
    axis_value: f64,
    hover_px: Point,
) -> Option<(f64, Option<HoverHit>)> {
    // Bars use category axes and snap via ordinal rounding above.
    if primary.kind == crate::spec::SeriesKind::Bar {
        return None;
    }

    // Only support Y-axis snapping when the trigger axis is the series value axis (future-facing;
    // v1 typically triggers on X, except for bar charts).
    let trigger_axis = primary.y_axis;
    if model
        .axes
        .get(&trigger_axis)
        .is_some_and(|a| a.kind != crate::spec::AxisKind::Y)
    {
        return None;
    }

    let root_dataset = model.root_dataset_id(primary.dataset);
    let table = datasets.dataset(root_dataset)?;
    let table_rev = table.revision;
    let model_rev = model.revs.data;

    let dataset = model.datasets.get(&primary.dataset)?;
    let x_col = dataset.fields.get(&primary.encode.x).copied()?;
    let y0_col = dataset.fields.get(&primary.encode.y).copied()?;

    let x = table.column_f64(x_col)?;
    let y0 = table.column_f64(y0_col)?;
    let y1 = primary
        .encode
        .y2
        .and_then(|y2_field| dataset.fields.get(&y2_field).copied())
        .and_then(|y2_col| table.column_f64(y2_col));

    let contract = participation.series_contract(primary.id, table.row_count);
    let selection_range = contract.selection_range;
    let filter = contract.x_policy.filter;
    let base_selection = contract.selection.clone();
    let empty_mask = contract.empty_mask;

    let table_view = data_views.table_view_for(
        table,
        primary.dataset,
        root_dataset,
        x_col,
        selection_range,
        filter,
        base_selection,
    );

    let view_len = table_view.len();
    if view_len == 0 {
        return None;
    }
    if view_len > MAX_UNSORTED_AXIS_SCAN_POINTS {
        return None;
    }

    let mut best: Option<(usize, f64, f64)> = None; // (raw_index, x_raw, y_eff)
    let mut best_dist = f64::INFINITY;

    for view_index in 0..view_len {
        let Some(raw_index) = table_view.get_raw_index(view_index) else {
            continue;
        };

        let x_raw = x.get(raw_index).copied().unwrap_or(f64::NAN);
        if !x_raw.is_finite() {
            continue;
        }
        if !filter.contains(x_raw) {
            continue;
        }

        let Some(sampled) = sample_at_raw_index_with_empty_mask(
            primary.id,
            empty_mask,
            model,
            datasets,
            stack_dims,
            model_rev,
            table_rev,
            raw_index,
            x,
            y0,
            y1.as_deref(),
        ) else {
            continue;
        };

        if !sampled.y0.is_finite() {
            continue;
        }
        let dist = (axis_value - sampled.y0).abs();
        if dist < best_dist {
            best_dist = dist;
            best = Some((raw_index, x_raw, sampled.y0));
        }
    }

    let (raw_index, x_raw, y_eff) = best?;
    if !y_eff.is_finite() {
        return None;
    }

    let x_window = axis_windows
        .get(&primary.x_axis)
        .copied()
        .unwrap_or_default();

    let px_x = crate::engine::axis::x_px_at_data_in_rect(x_window, x_raw, viewport);
    let px_y = crate::engine::axis::y_px_at_data_in_rect(trigger_window, y_eff, viewport);
    let point_px = Point::new(Px(px_x), Px(px_y));

    let dx = hover_px.x.0 - point_px.x.0;
    let dy = hover_px.y.0 - point_px.y.0;
    let dist2_px = dx * dx + dy * dy;

    let hit = HoverHit {
        series: primary.id,
        data_index: raw_index as u32,
        point_px,
        dist2_px,
        x_value: x_raw,
        y_value: y_eff,
    };

    Some((y_eff, Some(hit)))
}

#[derive(Debug, Clone, Copy)]
struct SampledSeriesValue {
    y0: f64,
    y1: Option<f64>,
}

fn raw_index_is_visible_under_empty_mask(
    empty_mask: crate::view::SeriesEmptyMask,
    raw_index: usize,
    x: &[f64],
    y0: &[f64],
    y1: Option<&[f64]>,
) -> bool {
    empty_mask.allows_raw_index(raw_index, x, y0, y1)
}

fn sample_at_raw_index_with_empty_mask(
    series_id: crate::ids::SeriesId,
    empty_mask: crate::view::SeriesEmptyMask,
    model: &ChartModel,
    datasets: &DatasetStore,
    stack_dims: &StackDimsStage,
    model_rev: Revision,
    table_rev: Revision,
    raw_index: usize,
    x: &[f64],
    y0: &[f64],
    y1: Option<&[f64]>,
) -> Option<SampledSeriesValue> {
    if !raw_index_is_visible_under_empty_mask(empty_mask, raw_index, x, y0, y1) {
        return None;
    }

    sample_at_raw_index(
        model, datasets, stack_dims, model_rev, table_rev, series_id, raw_index, y0, y1,
    )
}

fn ordinal_from_value(value: f64, len: usize) -> Option<u32> {
    if !value.is_finite() || len == 0 {
        return None;
    }

    let ord = value.round() as i64;
    if ord < 0 || ord >= len as i64 {
        return None;
    }
    Some(ord as u32)
}

fn sample_at_raw_index(
    model: &ChartModel,
    datasets: &DatasetStore,
    stack_dims: &StackDimsStage,
    model_rev: Revision,
    table_rev: Revision,
    series_id: crate::ids::SeriesId,
    raw_index: usize,
    y0: &[f64],
    y1: Option<&[f64]>,
) -> Option<SampledSeriesValue> {
    let mut y = y0.get(raw_index).copied()?;
    if !y.is_finite() {
        return None;
    }

    if model
        .series
        .get(&series_id)
        .is_some_and(|s| s.stack.is_some())
    {
        let stack = model.series.get(&series_id).and_then(|s| s.stack);
        if let Some(stack) = stack {
            if let Some(stacked) =
                stack_dims.stacked_y(stack, series_id, raw_index, model_rev, table_rev)
            {
                y = stacked;
            } else {
                y += stack_base_cached(
                    model, datasets, stack_dims, model_rev, table_rev, series_id, raw_index, y,
                );
            }
        }
    }

    let y1_out = y1
        .and_then(|s| s.get(raw_index).copied())
        .filter(|v| v.is_finite());
    Some(SampledSeriesValue { y0: y, y1: y1_out })
}

fn request_ordinal_indices_for_axis_pointer(
    ordinal_indices: &mut OrdinalIndexStage,
    model: &ChartModel,
    datasets: &DatasetStore,
    participation: &crate::engine::stages::ParticipationState,
) {
    for series in model.series_in_order() {
        if !series.visible {
            continue;
        }

        let bar_mapping = crate::engine::bar::bar_mapping_for_series(model, series.id)
            .filter(|_| series.kind == crate::spec::SeriesKind::Bar);
        let category_axis = bar_mapping
            .map(|m| m.category_axis)
            .unwrap_or(series.x_axis);

        let category_len = model.axes.get(&category_axis).and_then(|a| match &a.scale {
            crate::scale::AxisScale::Category(scale) => Some(scale.len()),
            _ => None,
        });
        let Some(category_len) = category_len else {
            continue;
        };

        let Some(table) = datasets.dataset(model.root_dataset_id(series.dataset)) else {
            continue;
        };
        let Some(dataset) = model.datasets.get(&series.dataset) else {
            continue;
        };
        let ordinal_col = if let Some(mapping) = bar_mapping {
            dataset.fields.get(&mapping.category_field).copied()
        } else {
            dataset.fields.get(&series.encode.x).copied()
        };
        let Some(ordinal_col) = ordinal_col else {
            continue;
        };

        let contract = participation.series_contract(series.id, table.row_count);
        let selection_range = contract.selection_range;
        let filter = contract.x_policy.filter;
        let selection = contract.selection;

        if matches!(selection, RowSelection::Indices(_)) {
            continue;
        }

        let filter_for_index = if category_axis == series.x_axis {
            filter
        } else {
            crate::engine::window_policy::AxisFilter1D::default()
        };
        let root_dataset = model.root_dataset_id(series.dataset);
        let key = crate::engine::stages::OrdinalIndexKey::new(
            series.dataset,
            root_dataset,
            ordinal_col,
            category_len,
            selection_range,
            filter_for_index,
        );
        ordinal_indices.request(key);
    }
}

const MAX_UNSORTED_AXIS_SCAN_POINTS: usize = 200_000;

fn sample_nearest_at_x_view(
    model: &ChartModel,
    datasets: &DatasetStore,
    stack_dims: &StackDimsStage,
    model_rev: Revision,
    table_rev: Revision,
    series_id: crate::ids::SeriesId,
    empty_mask: crate::view::SeriesEmptyMask,
    x_value: f64,
    x: &[f64],
    y0: &[f64],
    y1: Option<&[f64]>,
    x_filter: crate::engine::window_policy::AxisFilter1D,
    table_view: &crate::data::DataTableView<'_>,
    nearest_index: Option<&[crate::engine::stages::NearestXIndexItem]>,
) -> Option<SampledSeriesValue> {
    if !empty_mask.allows_axis_x_value(x_value) {
        return None;
    }

    if let Some(index) = nearest_index
        && let Some((raw_index, _x_raw)) =
            crate::engine::stages::nearest_raw_index_in_sorted_x_index(index, x_value)
    {
        let x_raw = x.get(raw_index).copied().unwrap_or(f64::NAN);
        if x_raw.is_finite()
            && x_filter.contains(x_raw)
            && raw_index_is_visible_under_empty_mask(empty_mask, raw_index, x, y0, y1)
        {
            return sample_at_raw_index_with_empty_mask(
                series_id, empty_mask, model, datasets, stack_dims, model_rev, table_rev,
                raw_index, x, y0, y1,
            );
        }
    }

    let view_len = table_view.len();
    if view_len == 0 {
        return None;
    }
    if view_len > MAX_UNSORTED_AXIS_SCAN_POINTS {
        return None;
    }

    let mut best_raw_index: Option<usize> = None;
    let mut best_dist = f64::INFINITY;

    for view_index in 0..view_len {
        let Some(raw_index) = table_view.get_raw_index(view_index) else {
            continue;
        };
        let x_raw = x.get(raw_index).copied().unwrap_or(f64::NAN);
        if !x_raw.is_finite() {
            continue;
        }
        if !x_filter.contains(x_raw) {
            continue;
        }
        if !raw_index_is_visible_under_empty_mask(empty_mask, raw_index, x, y0, y1) {
            continue;
        }
        let dist = (x_value - x_raw).abs();
        if dist < best_dist {
            best_dist = dist;
            best_raw_index = Some(raw_index);
        }
    }

    let raw_index = best_raw_index?;
    sample_at_raw_index_with_empty_mask(
        series_id, empty_mask, model, datasets, stack_dims, model_rev, table_rev, raw_index, x, y0,
        y1,
    )
}

fn sample_scatter_at_x_view(
    model: &ChartModel,
    datasets: &DatasetStore,
    stack_dims: &StackDimsStage,
    model_rev: Revision,
    table_rev: Revision,
    series_id: crate::ids::SeriesId,
    empty_mask: crate::view::SeriesEmptyMask,
    x_value: f64,
    x: &[f64],
    y0: &[f64],
    x_filter: crate::engine::window_policy::AxisFilter1D,
    table_view: &crate::data::DataTableView<'_>,
    nearest_index: Option<&[crate::engine::stages::NearestXIndexItem]>,
) -> Option<SampledSeriesValue> {
    if !empty_mask.allows_axis_x_value(x_value) {
        return None;
    }

    let len = x.len().min(y0.len());
    if len == 0 {
        return None;
    }

    let selection = table_view.selection();
    let (start, end) = match selection {
        RowSelection::All => (0usize, len),
        RowSelection::Range(range) => {
            let r = range.as_std_range(len);
            (r.start, r.end)
        }
        RowSelection::Indices(_) => (0usize, 0usize),
    };

    if start < end
        && crate::transform::is_probably_monotonic_in_range(x, RowRange { start, end }).is_some()
    {
        let xs = &x[start..end];
        if xs.len() == 1 {
            if !raw_index_is_visible_under_empty_mask(empty_mask, start, x, y0, None) {
                return None;
            }
            return Some(SampledSeriesValue {
                y0: y0[start],
                y1: None,
            });
        }

        let idx = lower_bound(xs, x_value);
        let i1 = (start + idx).min(end - 1);
        let i0 = i1.saturating_sub(1).max(start);

        let x0 = x.get(i0).copied().unwrap_or(f64::NAN);
        let x1 = x.get(i1).copied().unwrap_or(f64::NAN);
        if !x0.is_finite() || !x1.is_finite() {
            return None;
        }

        let d0 = (x_value - x0).abs();
        let d1 = (x_value - x1).abs();
        let i = if d1 < d0 { i1 } else { i0 };

        if !raw_index_is_visible_under_empty_mask(empty_mask, i, x, y0, None) {
            return None;
        }

        let y = y0.get(i).copied()?;
        if !y.is_finite() {
            return None;
        }

        let y = if model
            .series
            .get(&series_id)
            .is_some_and(|s| s.stack.is_some())
        {
            y + stack_base_cached(
                model, datasets, stack_dims, model_rev, table_rev, series_id, i, y,
            )
        } else {
            y
        };

        return Some(SampledSeriesValue { y0: y, y1: None });
    }

    sample_nearest_at_x_view(
        model,
        datasets,
        stack_dims,
        model_rev,
        table_rev,
        series_id,
        empty_mask,
        x_value,
        x,
        y0,
        None,
        x_filter,
        table_view,
        nearest_index,
    )
}

fn sample_series_at_x_view(
    model: &ChartModel,
    datasets: &DatasetStore,
    stack_dims: &StackDimsStage,
    model_rev: Revision,
    table_rev: Revision,
    series_id: crate::ids::SeriesId,
    empty_mask: crate::view::SeriesEmptyMask,
    x_value: f64,
    x: &[f64],
    y0: &[f64],
    y1: Option<&[f64]>,
    x_filter: crate::engine::window_policy::AxisFilter1D,
    table_view: &crate::data::DataTableView<'_>,
    nearest_index: Option<&[crate::engine::stages::NearestXIndexItem]>,
) -> Option<SampledSeriesValue> {
    if !empty_mask.allows_axis_x_value(x_value) {
        return None;
    }

    let len = x.len().min(y0.len());
    if len == 0 {
        return None;
    }

    let selection = table_view.selection();
    let (start, end) = match selection {
        RowSelection::All => (0usize, len),
        RowSelection::Range(range) => {
            let r = range.as_std_range(len);
            (r.start, r.end)
        }
        RowSelection::Indices(_) => (0usize, 0usize),
    };

    if start < end
        && crate::transform::is_probably_monotonic_in_range(x, RowRange { start, end }).is_some()
    {
        let xs = &x[start..end];
        if xs.len() == 1 {
            let raw_index = start;
            if !raw_index_is_visible_under_empty_mask(empty_mask, raw_index, x, y0, y1) {
                return None;
            }
            return Some(SampledSeriesValue {
                y0: y0[raw_index],
                y1: y1.and_then(|s| s.get(raw_index).copied()),
            });
        }

        let idx = lower_bound(xs, x_value);
        let mut i1 = (start + idx).min(end - 1);
        let mut i0 = i1.saturating_sub(1).max(start);
        if i0 == i1 {
            // `lower_bound` can return the start index when sampling at/before the first point.
            // Ensure we have a non-degenerate segment for interpolation.
            if i1 + 1 < end {
                i1 += 1;
            } else if i0 > start {
                i0 -= 1;
            }
        }

        let x0 = x[i0];
        let x1v = x[i1];
        if !x0.is_finite() || !x1v.is_finite() || x1v <= x0 {
            return None;
        }

        let t = ((x_value - x0) / (x1v - x0)).clamp(0.0, 1.0);
        let y0a = y0.get(i0).copied()?;
        let y0b = y0.get(i1).copied()?;
        if !y0a.is_finite() || !y0b.is_finite() {
            return None;
        }

        if !raw_index_is_visible_under_empty_mask(empty_mask, i0, x, y0, y1)
            || !raw_index_is_visible_under_empty_mask(empty_mask, i1, x, y0, y1)
        {
            return sample_nearest_at_x_view(
                model,
                datasets,
                stack_dims,
                model_rev,
                table_rev,
                series_id,
                empty_mask,
                x_value,
                x,
                y0,
                y1,
                x_filter,
                table_view,
                nearest_index,
            );
        }

        let y = if model
            .series
            .get(&series_id)
            .is_some_and(|s| s.stack.is_some())
        {
            let base0 = stack_base_cached(
                model, datasets, stack_dims, model_rev, table_rev, series_id, i0, y0a,
            );
            let base1 = stack_base_cached(
                model, datasets, stack_dims, model_rev, table_rev, series_id, i1, y0b,
            );
            let y_eff0 = y0a + base0;
            let y_eff1 = y0b + base1;
            y_eff0 + t * (y_eff1 - y_eff0)
        } else {
            y0a + t * (y0b - y0a)
        };

        let y1_out = if let Some(y1) = y1 {
            let y1a = y1.get(i0).copied()?;
            let y1b = y1.get(i1).copied()?;
            if !y1a.is_finite() || !y1b.is_finite() {
                return None;
            }
            Some(y1a + t * (y1b - y1a))
        } else {
            None
        };

        return Some(SampledSeriesValue { y0: y, y1: y1_out });
    }

    sample_nearest_at_x_view(
        model,
        datasets,
        stack_dims,
        model_rev,
        table_rev,
        series_id,
        empty_mask,
        x_value,
        x,
        y0,
        y1,
        x_filter,
        table_view,
        nearest_index,
    )
}

fn stack_base_cached(
    model: &ChartModel,
    datasets: &DatasetStore,
    stack_dims: &StackDimsStage,
    model_rev: Revision,
    table_rev: Revision,
    series_id: crate::ids::SeriesId,
    raw_index: usize,
    y: f64,
) -> f64 {
    let Some(stack) = model.series.get(&series_id).and_then(|s| s.stack) else {
        return 0.0;
    };

    stack_dims
        .stack_base(stack, series_id, raw_index, model_rev, table_rev)
        .unwrap_or_else(|| {
            stack_base_at_index(model, datasets, series_id, raw_index, y)
                .map(|b| b.base)
                .unwrap_or(0.0)
        })
}

fn lower_bound(xs: &[f64], value: f64) -> usize {
    let mut lo = 0usize;
    let mut hi = xs.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if xs[mid] < value {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}
