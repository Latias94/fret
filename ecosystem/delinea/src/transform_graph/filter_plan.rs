use crate::engine::model::ChartModel;
use crate::ids::{GridId, SeriesId};
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone)]
pub struct FilterPlan {
    pub grids: BTreeMap<GridId, Vec<SeriesId>>,
    pub steps: Vec<FilterPlanStep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilterPlanStep {
    pub grid: GridId,
    pub kind: FilterPlanStepKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterPlanStepKind {
    XYWeakFilter,
    XRange,
    XIndices,
    YPercent,
    YIndices,
}

pub fn build_filter_plan(model: &ChartModel) -> FilterPlan {
    let mut grids: BTreeMap<GridId, Vec<SeriesId>> = BTreeMap::new();

    for series_id in &model.series_order {
        let Some(series_model) = model.series.get(series_id) else {
            continue;
        };
        let Some(axis) = model.axes.get(&series_model.x_axis) else {
            continue;
        };

        grids.entry(axis.grid).or_default().push(*series_id);
    }

    let mut steps: Vec<FilterPlanStep> = Vec::with_capacity(grids.len() * 5);
    for grid in grids.keys().copied() {
        steps.push(FilterPlanStep {
            grid,
            kind: FilterPlanStepKind::XYWeakFilter,
        });
        steps.push(FilterPlanStep {
            grid,
            kind: FilterPlanStepKind::XRange,
        });
        steps.push(FilterPlanStep {
            grid,
            kind: FilterPlanStepKind::XIndices,
        });
        steps.push(FilterPlanStep {
            grid,
            kind: FilterPlanStepKind::YPercent,
        });
        steps.push(FilterPlanStep {
            grid,
            kind: FilterPlanStepKind::YIndices,
        });
    }

    FilterPlan { grids, steps }
}
