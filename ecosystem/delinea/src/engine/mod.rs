use fret_core::Rect;

use crate::action::Action;
use crate::data::DatasetStore;
use crate::engine::lod::LodScratch;
use crate::engine::model::{ChartModel, ModelError};
use crate::engine::stages::MarksStage;
use crate::ids::{ChartId, Revision};
use crate::link::{LinkConfig, LinkEvent};
use crate::marks::MarkTree;
use crate::scheduler::{StepResult, WorkBudget};
use crate::stats::EngineStats;
use crate::text::TextMeasurer;
use crate::view::ViewState;
use fret_core::Point;
use std::collections::BTreeMap;

pub mod hit_test;
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
    pub data_window_x: BTreeMap<crate::ids::AxisId, window::DataWindowX>,
    pub data_window_y: BTreeMap<crate::ids::AxisId, window::DataWindowY>,
    pub hover_px: Option<Point>,
    pub dataset_row_ranges: BTreeMap<crate::ids::DatasetId, crate::view::RowRange>,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChartOutput {
    pub revision: Revision,
    pub viewport: Option<Rect>,
    pub marks: MarkTree,
    pub link_events: Vec<LinkEvent>,
    pub hover: Option<HoverHit>,
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
    marks_stage: MarksStage,
    lod_scratch: LodScratch,
}

impl ChartEngine {
    pub fn new(spec: crate::spec::ChartSpec) -> Result<Self, ModelError> {
        let id = spec.id;
        let model = ChartModel::from_spec(spec)?;
        Ok(Self {
            id,
            model,
            datasets: DatasetStore::default(),
            state: ChartState::default(),
            output: ChartOutput::default(),
            stats: EngineStats::default(),
            view: ViewState::default(),
            marks_stage: MarksStage::default(),
            lod_scratch: LodScratch::default(),
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
                self.state.revision.bump();
            }
            Action::SetDataWindowX { axis, window } => {
                if let Some(mut window) = window {
                    window.clamp_non_degenerate();
                    self.state.data_window_x.insert(axis, window);
                } else {
                    self.state.data_window_x.remove(&axis);
                }
                self.state.revision.bump();
                self.marks_stage.mark_dirty();
            }
            Action::SetDataWindowY { axis, window } => {
                if let Some(mut window) = window {
                    window.clamp_non_degenerate();
                    self.state.data_window_y.insert(axis, window);
                } else {
                    self.state.data_window_y.remove(&axis);
                }
                self.state.revision.bump();
                self.marks_stage.mark_dirty();
            }
            Action::SetViewWindow2D {
                x_axis,
                y_axis,
                x,
                y,
            } => {
                if let Some(mut x) = x {
                    x.clamp_non_degenerate();
                    self.state.data_window_x.insert(x_axis, x);
                } else {
                    self.state.data_window_x.remove(&x_axis);
                }

                if let Some(mut y) = y {
                    y.clamp_non_degenerate();
                    self.state.data_window_y.insert(y_axis, y);
                } else {
                    self.state.data_window_y.remove(&y_axis);
                }

                self.state.revision.bump();
                self.marks_stage.mark_dirty();
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
        self.output.hover = None;

        self.view
            .sync_inputs(&self.model, &self.datasets, &self.state);
        // P0: Always rebuild the view when inputs change. This will become incremental when we add
        // transforms like dataZoom/filter/aggregate.
        self.view.rebuild(&self.model, &self.datasets, &self.state);

        self.marks_stage
            .sync_inputs(&self.model, &self.datasets, &self.view);
        if self.marks_stage.is_dirty() {
            self.output.marks.clear();
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
            viewport,
            &mut budget,
            &mut self.lod_scratch,
            &mut self.output.marks,
            &mut self.stats,
        );

        let unfinished = !done;

        if let Some(hover_px) = self.state.hover_px {
            self.output.hover =
                hit_test::hover_hit_test(&self.model, &self.datasets, &self.output.marks, hover_px);
        }

        self.output.revision.bump();
        Ok(StepResult { unfinished })
    }
}
