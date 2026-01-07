use fret_core::Rect;

use crate::action::Action;
use crate::data::DatasetStore;
use crate::engine::lod::LodScratch;
use crate::engine::stages::MarksStage;
use crate::ids::{ChartId, Revision};
use crate::link::{LinkConfig, LinkEvent};
use crate::marks::MarkTree;
use crate::scheduler::{StepResult, WorkBudget};
use crate::spec::ChartSpec;
use crate::stats::EngineStats;
use crate::text::TextMeasurer;
use fret_core::Point;

pub mod hit_test;
pub mod lod;
pub mod stages;
pub mod window;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChartState {
    pub revision: Revision,
    pub link: LinkConfig,
    pub data_window_x: Option<window::DataWindowX>,
    pub hover_px: Option<Point>,
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
    spec: ChartSpec,
    datasets: DatasetStore,
    state: ChartState,
    output: ChartOutput,
    stats: EngineStats,
    marks_stage: MarksStage,
    lod_scratch: LodScratch,
}

impl ChartEngine {
    pub fn new(spec: ChartSpec) -> Self {
        let id = spec.id;
        Self {
            id,
            spec,
            datasets: DatasetStore::default(),
            state: ChartState::default(),
            output: ChartOutput::default(),
            stats: EngineStats::default(),
            marks_stage: MarksStage::default(),
            lod_scratch: LodScratch::default(),
        }
    }

    pub fn id(&self) -> ChartId {
        self.id
    }

    pub fn spec(&self) -> &ChartSpec {
        &self.spec
    }

    pub fn spec_mut(&mut self) -> &mut ChartSpec {
        &mut self.spec
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

    pub fn stats(&self) -> &EngineStats {
        &self.stats
    }

    pub fn apply_action(&mut self, action: Action) {
        match action {
            Action::HoverAt { point } => {
                self.state.hover_px = Some(point);
                self.state.revision.bump();
            }
            Action::SetDataWindowX { window } => {
                self.state.data_window_x = window;
                self.state.revision.bump();
            }
            Action::SetLinkGroup { group } => {
                self.state.link.group = group;
                self.state.revision.bump();
            }
            _ => {
                self.state.revision.bump();
            }
        }
    }

    pub fn step(
        &mut self,
        _measurer: &mut dyn TextMeasurer,
        mut budget: WorkBudget,
    ) -> Result<StepResult, EngineError> {
        self.output.viewport = self.spec.viewport;
        if self.output.viewport.is_none() {
            return Err(EngineError::MissingViewport);
        }

        self.output.link_events.clear();
        self.output.hover = None;

        self.marks_stage.sync_inputs(&self.spec, &self.datasets);
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
            &self.spec,
            &self.datasets,
            &self.state,
            viewport,
            &mut budget,
            &mut self.lod_scratch,
            &mut self.output.marks,
            &mut self.stats,
        );

        let unfinished = !done;

        if let Some(hover_px) = self.state.hover_px {
            self.output.hover =
                hit_test::hover_hit_test(&self.spec, &self.datasets, &self.output.marks, hover_px);
        }

        self.output.revision.bump();
        Ok(StepResult { unfinished })
    }
}
