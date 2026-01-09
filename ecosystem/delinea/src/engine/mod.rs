use fret_core::Rect;

use crate::action::Action;
use crate::data::DatasetStore;
use crate::engine::interaction::AxisInteractionLocks;
use crate::engine::lod::LodScratch;
use crate::engine::model::{ChartModel, ModelError};
use crate::engine::stages::{MarksStage, SelectionStage};
use crate::ids::{ChartId, Revision};
use crate::link::{LinkConfig, LinkEvent};
use crate::marks::MarkTree;
use crate::scheduler::{StepResult, WorkBudget};
use crate::spec::AxisPointerTrigger;
use crate::stats::EngineStats;
use crate::text::TextMeasurer;
use crate::tooltip::{TooltipLine, TooltipOutput};
use crate::transform::stack_base_at_index;
use crate::view::ViewState;
use fret_core::Point;
use std::collections::BTreeMap;

pub mod axis;
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
    pub axis_locks: BTreeMap<crate::ids::AxisId, AxisInteractionLocks>,
    pub hover_px: Option<Point>,
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
    pub marks: MarkTree,
    pub axis_windows: BTreeMap<crate::ids::AxisId, window::DataWindow>,
    pub link_events: Vec<LinkEvent>,
    pub hover: Option<HoverHit>,
    pub axis_pointer: Option<AxisPointerOutput>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AxisPointerOutput {
    pub crosshair_px: Point,
    pub hit: Option<HoverHit>,
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
}

impl core::fmt::Display for EngineError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingViewport => write!(f, "missing viewport"),
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
    selection_stage: SelectionStage,
    marks_stage: MarksStage,
    lod_scratch: LodScratch,
    axis_pointer_cache: AxisPointerCache,
}

#[derive(Debug, Default, Clone)]
struct AxisPointerCache {
    last_hover_px: Option<Point>,
    last_marks_rev: Revision,
    hit: Option<HoverHit>,
    output: Option<AxisPointerOutput>,
}

impl ChartEngine {
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
        Ok(Self {
            id,
            model,
            datasets: DatasetStore::default(),
            state,
            output: ChartOutput::default(),
            stats: EngineStats::default(),
            view: ViewState::default(),
            selection_stage: SelectionStage::default(),
            marks_stage: MarksStage::default(),
            lod_scratch: LodScratch::default(),
            axis_pointer_cache: AxisPointerCache::default(),
        })
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

    pub fn view(&self) -> &ViewState {
        &self.view
    }

    pub fn stats(&self) -> &EngineStats {
        &self.stats
    }

    pub fn apply_action(&mut self, action: Action) {
        match action {
            Action::HoverAt { point } => {
                self.state.hover_px = Some(point);
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
                self.apply_pan_from_base(axis, base, delta_px, viewport_span_px);
            }
            Action::PanDataWindowYFromBase {
                axis,
                base,
                delta_px,
                viewport_span_px,
            } => {
                self.apply_pan_from_base(axis, base, delta_px, viewport_span_px);
            }
            Action::ZoomDataWindowXFromBase {
                axis,
                base,
                center_px,
                log2_scale,
                viewport_span_px,
            } => {
                self.apply_zoom_from_base(axis, base, center_px, log2_scale, viewport_span_px);
            }
            Action::ZoomDataWindowYFromBase {
                axis,
                base,
                center_px,
                log2_scale,
                viewport_span_px,
            } => {
                self.apply_zoom_from_base(axis, base, center_px, log2_scale, viewport_span_px);
            }
            Action::SetDataWindowXFromZoom { axis, window } => {
                self.apply_zoom_set_window(axis, window);
            }
            Action::SetDataWindowYFromZoom { axis, window } => {
                self.apply_zoom_set_window(axis, window);
            }
            Action::SetDataWindowX { axis, window } => {
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
            Action::SetLinkGroup { group } => {
                self.state.link.group = group;
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

        base.clamp_non_degenerate();
        let mut window = base.zoom_by_px(center_px, log2_scale, viewport_span_px);

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

    fn apply_zoom_set_window(&mut self, axis: crate::ids::AxisId, mut window: window::DataWindow) {
        let Some(axis_model) = self.model.axes.get(&axis) else {
            return;
        };
        if self.axis_is_fixed(axis) {
            return;
        }
        if self.axis_locks(axis).zoom_locked {
            return;
        }

        window.clamp_non_degenerate();
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
        if self.output.viewport.is_none() {
            return Err(EngineError::MissingViewport);
        }

        self.output.link_events.clear();

        let view_changed = self
            .view
            .sync_inputs(&self.model, &self.datasets, &self.state);
        if view_changed {
            self.view.rebuild(&self.model, &self.datasets, &self.state);
        }

        self.selection_stage
            .sync_inputs(&self.model, &self.datasets, &self.view);
        let selection_done = self.selection_stage.step(&self.datasets, &mut budget);

        self.marks_stage
            .sync_inputs(&self.model, &self.datasets, &self.view);
        if self.marks_stage.is_dirty() {
            self.output.marks.clear();
            self.output.axis_windows.clear();
            self.marks_stage.reset();
        }

        self.stats.stage_data_runs += 1;
        self.stats.stage_layout_runs += 1;
        self.stats.stage_visual_runs += 1;
        self.stats.stage_marks_runs += 1;

        let viewport = self.output.viewport.unwrap();
        let done = self.marks_stage.step(
            &self.model,
            &self.datasets,
            &self.state,
            &self.view,
            &self.selection_stage,
            viewport,
            &mut budget,
            &mut self.lod_scratch,
            &mut self.output.marks,
            &mut self.stats,
        );

        self.output
            .axis_windows
            .clone_from(self.marks_stage.axis_windows());
        for axis in self.model.axes.values() {
            if let crate::scale::AxisScale::Category(scale) = &axis.scale
                && !scale.categories.is_empty()
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

        let unfinished = !done || !selection_done;

        let hover_px = self.state.hover_px;
        let marks_rev = self.output.marks.revision;
        if self.axis_pointer_cache.last_marks_rev != marks_rev {
            self.axis_pointer_cache.last_marks_rev = marks_rev;
            self.axis_pointer_cache.last_hover_px = None;
            self.axis_pointer_cache.hit = None;
            self.axis_pointer_cache.output = None;
        }

        let axis_pointer = self.model.axis_pointer.filter(|p| p.enabled);

        if let Some(hover_px) = hover_px {
            let should_recompute = axis_pointer.is_some_and(|p| {
                should_recompute_hover(
                    self.axis_pointer_cache.last_hover_px,
                    hover_px,
                    p.throttle_px,
                )
            });
            if should_recompute {
                self.axis_pointer_cache.last_hover_px = Some(hover_px);
                self.axis_pointer_cache.hit = None;
                self.axis_pointer_cache.output = None;

                if let Some(spec) = axis_pointer {
                    let viewport = self.output.viewport.unwrap_or_default();
                    if rect_contains_point(viewport, hover_px) {
                        match spec.trigger {
                            AxisPointerTrigger::Item => {
                                let hit = hit_test::hover_hit_test(
                                    &self.model,
                                    &self.datasets,
                                    &self.output.marks,
                                    hover_px,
                                );
                                self.axis_pointer_cache.hit = hit;
                                self.axis_pointer_cache.output = compute_item_axis_pointer_output(
                                    &self.model,
                                    &self.output.axis_windows,
                                    hover_px,
                                    hit,
                                    spec,
                                );
                            }
                            AxisPointerTrigger::Axis => {
                                let hit = hit_test::hover_hit_test(
                                    &self.model,
                                    &self.datasets,
                                    &self.output.marks,
                                    hover_px,
                                );
                                self.axis_pointer_cache.hit = hit;
                                self.axis_pointer_cache.output = compute_axis_axis_pointer_output(
                                    &self.model,
                                    &self.datasets,
                                    &self.view,
                                    &self.output.axis_windows,
                                    viewport,
                                    hover_px,
                                    hit,
                                    spec,
                                );
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

        self.output.revision.bump();
        Ok(StepResult { unfinished })
    }
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

fn compute_item_axis_pointer_output(
    model: &ChartModel,
    axis_windows: &BTreeMap<crate::ids::AxisId, window::DataWindow>,
    hover_px: Point,
    hit: Option<HoverHit>,
    spec: crate::engine::model::AxisPointerModel,
) -> Option<AxisPointerOutput> {
    let hit = hit?;
    let trigger2 = spec.trigger_distance_px.max(0.0) * spec.trigger_distance_px.max(0.0);
    if hit.dist2_px > trigger2 {
        return None;
    }

    let crosshair_px = if spec.snap { hit.point_px } else { hover_px };

    let series = model.series.get(&hit.series);
    let (x_axis, y_axis) = series.map(|s| (s.x_axis, s.y_axis)).unwrap_or_default();

    let series_value = series
        .and_then(|s| s.name.as_deref())
        .map(|n| n.to_string())
        .unwrap_or_else(|| hit.series.0.to_string());

    let x_axis_label = model
        .axes
        .get(&x_axis)
        .and_then(|a| a.name.as_deref())
        .map(|n| format!("x ({n})"))
        .unwrap_or_else(|| "x".to_string());
    let y_axis_label = model
        .axes
        .get(&y_axis)
        .and_then(|a| a.name.as_deref())
        .map(|n| format!("y ({n})"))
        .unwrap_or_else(|| "y".to_string());

    let x_window = axis_windows.get(&x_axis).copied().unwrap_or_default();
    let y_window = axis_windows.get(&y_axis).copied().unwrap_or_default();
    let x_value = crate::engine::axis::format_value_for(model, x_axis, x_window, hit.x_value);
    let y_value = crate::engine::axis::format_value_for(model, y_axis, y_window, hit.y_value);

    let mut tooltip = TooltipOutput::default();
    tooltip.lines.reserve(3);
    tooltip.lines.push(TooltipLine {
        label: "series".to_string(),
        value: series_value,
    });
    tooltip.lines.push(TooltipLine {
        label: x_axis_label,
        value: x_value,
    });
    tooltip.lines.push(TooltipLine {
        label: y_axis_label,
        value: y_value,
    });

    Some(AxisPointerOutput {
        crosshair_px,
        hit: Some(hit),
        tooltip,
    })
}

fn compute_axis_axis_pointer_output(
    model: &ChartModel,
    datasets: &DatasetStore,
    view: &ViewState,
    axis_windows: &BTreeMap<crate::ids::AxisId, window::DataWindow>,
    viewport: Rect,
    hover_px: Point,
    hit: Option<HoverHit>,
    spec: crate::engine::model::AxisPointerModel,
) -> Option<AxisPointerOutput> {
    let primary = model.series_in_order().find(|s| s.visible)?;
    let x_axis = primary.x_axis;

    let x_window = axis_windows.get(&x_axis).copied().unwrap_or_default();
    let trigger2 = spec.trigger_distance_px.max(0.0) * spec.trigger_distance_px.max(0.0);
    let hit_for_marker = hit.filter(|h| h.dist2_px <= trigger2);

    let x_value = if spec.snap
        && let Some(hit) = hit_for_marker
    {
        hit.x_value
    } else {
        crate::engine::axis::data_at_x_in_rect(x_window, hover_px.x.0, viewport)
    };
    if !x_value.is_finite() {
        return None;
    }

    let crosshair_px = if spec.snap
        && let Some(hit) = hit_for_marker
    {
        hit.point_px
    } else {
        hover_px
    };

    let mut tooltip = TooltipOutput::default();

    let x_label = model
        .axes
        .get(&x_axis)
        .and_then(|a| a.name.as_deref())
        .map(|n| format!("x ({n})"))
        .unwrap_or_else(|| "x".to_string());
    tooltip.lines.push(TooltipLine {
        label: x_label,
        value: crate::engine::axis::format_value_for(model, x_axis, x_window, x_value),
    });

    for series in model.series_in_order() {
        if !series.visible || series.x_axis != x_axis {
            continue;
        }

        let Some(table) = datasets
            .datasets
            .iter()
            .find_map(|(id, t)| (*id == series.dataset).then_some(t))
        else {
            continue;
        };
        let Some(dataset) = model.datasets.get(&series.dataset) else {
            continue;
        };
        let Some(x_col) = dataset.fields.get(&series.encode.x).copied() else {
            continue;
        };
        let Some(y0_col) = dataset.fields.get(&series.encode.y).copied() else {
            continue;
        };

        let Some(x) = table.column_f64(x_col) else {
            continue;
        };
        let Some(y0) = table.column_f64(y0_col) else {
            continue;
        };

        let y1 = if series.kind == crate::spec::SeriesKind::Band {
            if let Some(y2_field) = series.encode.y2
                && let Some(y2_col) = dataset.fields.get(&y2_field).copied()
                && let Some(y1) = table.column_f64(y2_col)
            {
                Some(y1)
            } else {
                None
            }
        } else {
            None
        };

        let selection = view
            .series_view(series.id)
            .map(|v| v.selection.clone())
            .unwrap_or_default();
        let row_range = selection.as_range(table.row_count);

        let Some(sample) = (if series.kind == crate::spec::SeriesKind::Scatter {
            sample_scatter_at_x(model, datasets, series.id, x_value, x, y0, row_range)
        } else {
            sample_series_at_x(model, datasets, series.id, x_value, x, y0, y1, row_range)
        }) else {
            continue;
        };

        let label = series
            .name
            .as_deref()
            .map(|n| n.to_string())
            .unwrap_or_else(|| format!("Series {}", series.id.0));

        let y_window = axis_windows
            .get(&series.y_axis)
            .copied()
            .unwrap_or_default();
        let value = if let Some(y1) = sample.y1 {
            let a =
                crate::engine::axis::format_value_for(model, series.y_axis, y_window, sample.y0);
            let b = crate::engine::axis::format_value_for(model, series.y_axis, y_window, y1);
            format!("{a} .. {b}")
        } else {
            crate::engine::axis::format_value_for(model, series.y_axis, y_window, sample.y0)
        };

        tooltip.lines.push(TooltipLine { label, value });
    }

    Some(AxisPointerOutput {
        crosshair_px,
        hit: hit_for_marker,
        tooltip,
    })
}

#[derive(Debug, Clone, Copy)]
struct SampledSeriesValue {
    y0: f64,
    y1: Option<f64>,
}

fn sample_scatter_at_x(
    model: &ChartModel,
    datasets: &DatasetStore,
    series_id: crate::ids::SeriesId,
    x_value: f64,
    x: &[f64],
    y0: &[f64],
    row_range: core::ops::Range<usize>,
) -> Option<SampledSeriesValue> {
    let len = x.len().min(y0.len());
    if len == 0 {
        return None;
    }
    let start = row_range.start.min(len);
    let end = row_range.end.min(len);
    if end <= start {
        return None;
    }

    let xs = &x[start..end];
    if xs.len() == 1 {
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

    let y = y0.get(i).copied()?;
    if !y.is_finite() {
        return None;
    }

    let y = if model
        .series
        .get(&series_id)
        .is_some_and(|s| s.stack.is_some())
    {
        let base = stack_base_at_index(model, datasets, series_id, i, y)?.base;
        y + base
    } else {
        y
    };

    Some(SampledSeriesValue { y0: y, y1: None })
}

fn sample_series_at_x(
    model: &ChartModel,
    datasets: &DatasetStore,
    series_id: crate::ids::SeriesId,
    x_value: f64,
    x: &[f64],
    y0: &[f64],
    y1: Option<&[f64]>,
    row_range: core::ops::Range<usize>,
) -> Option<SampledSeriesValue> {
    let len = x.len().min(y0.len());
    if len == 0 {
        return None;
    }
    let start = row_range.start.min(len);
    let end = row_range.end.min(len);
    if end <= start {
        return None;
    }

    let xs = &x[start..end];
    if xs.len() == 1 {
        return Some(SampledSeriesValue {
            y0: y0[start],
            y1: y1.and_then(|s| s.get(start).copied()),
        });
    }

    let idx = lower_bound(xs, x_value);
    let i1 = (start + idx).min(end - 1);
    let i0 = i1.saturating_sub(1).max(start);

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

    let y = if model
        .series
        .get(&series_id)
        .is_some_and(|s| s.stack.is_some())
    {
        let base0 = stack_base_at_index(model, datasets, series_id, i0, y0a)?.base;
        let base1 = stack_base_at_index(model, datasets, series_id, i1, y0b)?.base;
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

    Some(SampledSeriesValue { y0: y, y1: y1_out })
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
