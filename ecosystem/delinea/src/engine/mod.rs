use fret_core::Rect;

use crate::action::Action;
use crate::data::DatasetStore;
use crate::engine::interaction::AxisInteractionLocks;
use crate::engine::lod::LodScratch;
use crate::engine::model::{ChartModel, ModelError};
use crate::engine::stages::{
    BarLayoutStage, DataViewStage, MarksStage, OrdinalIndexStage, StackDimsStage,
};
use crate::ids::{ChartId, Revision};
use crate::link::{LinkConfig, LinkEvent};
use crate::marks::MarkTree;
use crate::scheduler::{StepResult, WorkBudget};
use crate::spec::AxisPointerTrigger;
use crate::stats::EngineStats;
use crate::text::TextMeasurer;
use crate::tooltip::{TooltipLine, TooltipOutput};
use crate::transform::stack_base_at_index;
use crate::transform::{RowRange, RowSelection};
use crate::view::ViewState;
use fret_core::Point;
use std::collections::BTreeMap;

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

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BrushSelection2D {
    pub x_axis: crate::ids::AxisId,
    pub y_axis: crate::ids::AxisId,
    pub x: window::DataWindowX,
    pub y: window::DataWindowY,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChartState {
    pub revision: Revision,
    pub link: LinkConfig,
    pub data_zoom_x: BTreeMap<crate::ids::AxisId, DataZoomXState>,
    pub data_window_y: BTreeMap<crate::ids::AxisId, window::DataWindowY>,
    pub axis_locks: BTreeMap<crate::ids::AxisId, AxisInteractionLocks>,
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
    pub marks: MarkTree,
    pub axis_windows: BTreeMap<crate::ids::AxisId, window::DataWindow>,
    pub link_events: Vec<LinkEvent>,
    pub hover: Option<HoverHit>,
    pub axis_pointer: Option<AxisPointerOutput>,
    pub brush_selection_2d: Option<BrushSelection2D>,
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
    data_view_stage: DataViewStage,
    ordinal_index_stage: OrdinalIndexStage,
    bar_layout_stage: BarLayoutStage,
    stack_dims_stage: StackDimsStage,
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
            data_view_stage: DataViewStage::default(),
            ordinal_index_stage: OrdinalIndexStage::default(),
            bar_layout_stage: BarLayoutStage::default(),
            stack_dims_stage: StackDimsStage::default(),
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
            Action::SetBrushSelection2D {
                x_axis,
                y_axis,
                x,
                y,
            } => {
                let next = BrushSelection2D {
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
            Action::SetDataWindowXFromZoom {
                axis,
                base,
                window,
                anchor,
            } => {
                self.apply_zoom_set_window(axis, base, window, anchor);
            }
            Action::SetDataWindowYFromZoom {
                axis,
                base,
                window,
                anchor,
            } => {
                self.apply_zoom_set_window(axis, base, window, anchor);
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
            Action::SetViewWindow2DFromZoom {
                x_axis,
                y_axis,
                base_x,
                base_y,
                x,
                y,
            } => {
                let mut changed = false;

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
        if self.output.viewport.is_none() {
            return Err(EngineError::MissingViewport);
        }

        self.output.link_events.clear();
        self.output.brush_selection_2d = self.state.brush_selection_2d;

        let view_changed = self
            .view
            .sync_inputs(&self.model, &self.datasets, &self.state);
        if view_changed {
            self.view.rebuild(&self.model, &self.datasets, &self.state);
        }

        self.ordinal_index_stage.begin_frame();
        if self
            .model
            .axis_pointer
            .is_some_and(|p| p.enabled && p.trigger == AxisPointerTrigger::Axis)
        {
            request_ordinal_indices_for_axis_pointer(
                &mut self.ordinal_index_stage,
                &self.model,
                &self.datasets,
                &self.view,
            );
        }
        self.ordinal_index_stage.prepare_requests(&self.datasets);
        let ordinal_indices_done = self.ordinal_index_stage.step(&self.datasets, &mut budget);

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

        self.data_view_stage.begin_frame();
        self.marks_stage.request_data_views(
            &self.model,
            &self.datasets,
            &self.view,
            &mut self.data_view_stage,
        );
        self.data_view_stage.prepare_requests(&self.datasets);
        let selection_done = self.data_view_stage.step(&self.datasets, &mut budget);

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
            &self.data_view_stage,
            &self.stack_dims_stage,
            &self.bar_layout_stage,
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

        let unfinished = !done
            || !selection_done
            || !stack_dims_done
            || !ordinal_indices_done
            || !bar_layout_done;

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
                                    &self.stack_dims_stage,
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
                                    &self.stack_dims_stage,
                                );
                                self.axis_pointer_cache.hit = hit;
                                self.axis_pointer_cache.output = compute_axis_axis_pointer_output(
                                    &self.model,
                                    &self.datasets,
                                    &self.view,
                                    &self.data_view_stage,
                                    &self.stack_dims_stage,
                                    &self.ordinal_index_stage,
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
    data_views: &DataViewStage,
    stack_dims: &StackDimsStage,
    ordinal_indices: &OrdinalIndexStage,
    axis_windows: &BTreeMap<crate::ids::AxisId, window::DataWindow>,
    viewport: Rect,
    hover_px: Point,
    hit: Option<HoverHit>,
    spec: crate::engine::model::AxisPointerModel,
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

    let trigger_window = axis_windows.get(&trigger_axis).copied().unwrap_or_default();
    let trigger2 = spec.trigger_distance_px.max(0.0) * spec.trigger_distance_px.max(0.0);
    let hit_for_marker = hit.filter(|h| h.dist2_px <= trigger2);

    let axis_value = if spec.snap
        && let Some(hit) = hit_for_marker
    {
        match trigger_axis_kind {
            crate::spec::AxisKind::X => hit.x_value,
            crate::spec::AxisKind::Y => hit.y_value,
        }
    } else {
        match trigger_axis_kind {
            crate::spec::AxisKind::X => {
                crate::engine::axis::data_at_x_in_rect(trigger_window, hover_px.x.0, viewport)
            }
            crate::spec::AxisKind::Y => {
                crate::engine::axis::data_at_y_in_rect(trigger_window, hover_px.y.0, viewport)
            }
        }
    };
    if !axis_value.is_finite() {
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

    let axis_label = model
        .axes
        .get(&trigger_axis)
        .and_then(|a| a.name.as_deref())
        .map(|n| match trigger_axis_kind {
            crate::spec::AxisKind::X => format!("x ({n})"),
            crate::spec::AxisKind::Y => format!("y ({n})"),
        })
        .unwrap_or_else(|| match trigger_axis_kind {
            crate::spec::AxisKind::X => "x".to_string(),
            crate::spec::AxisKind::Y => "y".to_string(),
        });
    tooltip.lines.push(TooltipLine {
        label: axis_label,
        value: crate::engine::axis::format_value_for(
            model,
            trigger_axis,
            trigger_window,
            axis_value,
        ),
    });

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

        let Some(table) = datasets.dataset(series.dataset) else {
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

        let (selection_range, filter, base_selection) = match view.series_view(series.id) {
            Some(series_view) => {
                let selection_range = series_view.selection.as_range(table.row_count);
                let selection_range = RowRange {
                    start: selection_range.start,
                    end: selection_range.end,
                };
                (
                    selection_range,
                    series_view.x_policy.filter,
                    series_view.selection.clone(),
                )
            }
            None => (
                RowRange {
                    start: 0,
                    end: table.row_count,
                },
                crate::engine::window_policy::AxisFilter1D::default(),
                RowSelection::default(),
            ),
        };

        let selection_for_index = base_selection.clone();
        let filter_for_index = if series_trigger_axis == series.x_axis {
            filter
        } else {
            crate::engine::window_policy::AxisFilter1D::default()
        };

        let table_view = x_col.map(|x_col| {
            data_views.table_view_for(
                table,
                series.dataset,
                x_col,
                selection_range,
                filter,
                base_selection,
            )
        });

        let model_rev = model.revs.marks;
        let table_rev = table.revision;

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
                    ordinal_col,
                    category_len,
                    selection_range,
                    filter_for_index,
                );
                if let Some(raw_index) =
                    ordinal_indices.raw_index_of_ordinal(key, ordinal, table_rev)
                {
                    if let Some(mapping) = bar_mapping {
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
                    } else if let (Some(y0), Some(_x)) = (y0, x) {
                        sample = sample_at_raw_index(
                            model, datasets, stack_dims, model_rev, table_rev, series.id,
                            raw_index, y0, y1,
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
                    model, datasets, stack_dims, model_rev, table_rev, series.id, axis_value, x,
                    y0, table_view,
                )
            } else {
                sample_series_at_x_view(
                    model, datasets, stack_dims, model_rev, table_rev, series.id, axis_value, x,
                    y0, y1, table_view,
                )
            }
        });

        let label = series
            .name
            .as_deref()
            .map(|n| n.to_string())
            .unwrap_or_else(|| format!("Series {}", series.id.0));

        let value_axis = bar_mapping.map(|m| m.value_axis).unwrap_or(series.y_axis);
        let value_window = axis_windows.get(&value_axis).copied().unwrap_or_default();
        let value = match sample {
            Some(sample) => {
                if let Some(y1) = sample.y1 {
                    let a = crate::engine::axis::format_value_for(
                        model,
                        value_axis,
                        value_window,
                        sample.y0,
                    );
                    let b =
                        crate::engine::axis::format_value_for(model, value_axis, value_window, y1);
                    format!("{a} .. {b}")
                } else {
                    crate::engine::axis::format_value_for(
                        model,
                        value_axis,
                        value_window,
                        sample.y0,
                    )
                }
            }
            None => "-".to_string(),
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
    view: &ViewState,
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

        let Some(table) = datasets.dataset(series.dataset) else {
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

        let (selection_range, filter, selection) = match view.series_view(series.id) {
            Some(series_view) => {
                let selection_range = series_view.selection.as_range(table.row_count);
                let selection_range = RowRange {
                    start: selection_range.start,
                    end: selection_range.end,
                };
                (
                    selection_range,
                    series_view.x_policy.filter,
                    series_view.selection.clone(),
                )
            }
            None => (
                RowRange {
                    start: 0,
                    end: table.row_count,
                },
                crate::engine::window_policy::AxisFilter1D::default(),
                RowSelection::default(),
            ),
        };

        if matches!(selection, RowSelection::Indices(_)) {
            continue;
        }

        let filter_for_index = if category_axis == series.x_axis {
            filter
        } else {
            crate::engine::window_policy::AxisFilter1D::default()
        };
        let key = crate::engine::stages::OrdinalIndexKey::new(
            series.dataset,
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
    x_value: f64,
    x: &[f64],
    y0: &[f64],
    y1: Option<&[f64]>,
    table_view: &crate::data::DataTableView<'_>,
) -> Option<SampledSeriesValue> {
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
        let dist = (x_value - x_raw).abs();
        if dist < best_dist {
            best_dist = dist;
            best_raw_index = Some(raw_index);
        }
    }

    let raw_index = best_raw_index?;

    let mut y = y0.get(raw_index).copied()?;
    if !y.is_finite() {
        return None;
    }

    if model
        .series
        .get(&series_id)
        .is_some_and(|s| s.stack.is_some())
    {
        y += stack_base_cached(
            model, datasets, stack_dims, model_rev, table_rev, series_id, raw_index, y,
        );
    }

    let y1_out = y1.and_then(|s| s.get(raw_index).copied());
    Some(SampledSeriesValue { y0: y, y1: y1_out })
}

fn sample_scatter_at_x_view(
    model: &ChartModel,
    datasets: &DatasetStore,
    stack_dims: &StackDimsStage,
    model_rev: Revision,
    table_rev: Revision,
    series_id: crate::ids::SeriesId,
    x_value: f64,
    x: &[f64],
    y0: &[f64],
    table_view: &crate::data::DataTableView<'_>,
) -> Option<SampledSeriesValue> {
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
            y + stack_base_cached(
                model, datasets, stack_dims, model_rev, table_rev, series_id, i, y,
            )
        } else {
            y
        };

        return Some(SampledSeriesValue { y0: y, y1: None });
    }

    sample_nearest_at_x_view(
        model, datasets, stack_dims, model_rev, table_rev, series_id, x_value, x, y0, None,
        table_view,
    )
}

fn sample_series_at_x_view(
    model: &ChartModel,
    datasets: &DatasetStore,
    stack_dims: &StackDimsStage,
    model_rev: Revision,
    table_rev: Revision,
    series_id: crate::ids::SeriesId,
    x_value: f64,
    x: &[f64],
    y0: &[f64],
    y1: Option<&[f64]>,
    table_view: &crate::data::DataTableView<'_>,
) -> Option<SampledSeriesValue> {
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
        model, datasets, stack_dims, model_rev, table_rev, series_id, x_value, x, y0, y1,
        table_view,
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
