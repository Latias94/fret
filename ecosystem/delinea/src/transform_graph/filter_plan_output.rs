use super::TransformGraph;
use crate::engine::model::ChartModel;
use crate::engine::stages::{FilterPlanOutput, GridFilterOutput, SeriesFilterOutput};
use crate::ids::{AxisId, GridId, Revision};
use crate::view::ViewState;
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone)]
pub(super) struct CachedFilterPlanOutput {
    pub view_rev: Revision,
    pub model_rev: Revision,
    pub y_percent_sig: u64,
}

impl TransformGraph {
    pub fn refresh_filter_plan_output(
        &mut self,
        model: &ChartModel,
        view: &ViewState,
        y_percent_extents_by_grid: &BTreeMap<GridId, BTreeMap<AxisId, (f64, f64)>>,
    ) {
        let sig = y_percent_extents_signature(y_percent_extents_by_grid);
        let view_rev = view.revision;
        let model_rev = model.revs.spec;

        if let Some(cache) = &self.filter_plan_output_cache
            && cache.view_rev == view_rev
            && cache.model_rev == model_rev
            && cache.y_percent_sig == sig
        {
            return;
        }

        let plan = self.filter_plan(model);

        let mut grids: Vec<GridFilterOutput> = Vec::with_capacity(plan.grids.len());
        for (grid, series) in &plan.grids {
            grids.push(GridFilterOutput {
                grid: *grid,
                series: series.clone(),
                y_percent_extents: y_percent_extents_by_grid
                    .get(grid)
                    .cloned()
                    .unwrap_or_default(),
            });
        }

        let mut series: Vec<SeriesFilterOutput> = Vec::with_capacity(model.series_order.len());
        for series_id in &model.series_order {
            let Some(series_model) = model.series.get(series_id) else {
                series.push(SeriesFilterOutput {
                    series: *series_id,
                    ..Default::default()
                });
                continue;
            };

            let grid = model
                .axes
                .get(&series_model.x_axis)
                .map(|a| a.grid)
                .unwrap_or_default();

            let Some(v) = view.series_view(*series_id) else {
                series.push(SeriesFilterOutput {
                    series: *series_id,
                    dataset: series_model.dataset,
                    grid,
                    data_revision: Revision::default(),
                    ..Default::default()
                });
                continue;
            };

            let empty_mask = v.empty_mask(series_model.kind, series_model.stack.is_some());
            series.push(SeriesFilterOutput {
                series: *series_id,
                dataset: series_model.dataset,
                grid,
                data_revision: v.data_revision,
                selection: v.selection.clone(),
                x_policy: v.x_policy,
                x_filter_mode: v.x_filter_mode,
                y_filter_mode: v.y_filter_mode,
                y_filter: v.y_filter,
                empty_mask,
            });
        }

        self.filter_plan_output = FilterPlanOutput {
            revision: view.revision,
            grids,
            series,
        };
        self.filter_plan_output_cache = Some(CachedFilterPlanOutput {
            view_rev,
            model_rev,
            y_percent_sig: sig,
        });
    }
}

const FNV1A_OFFSET: u64 = 0xcbf29ce484222325;
const FNV1A_PRIME: u64 = 0x00000100000001B3;

fn fnv1a_step(hash: u64, value: u64) -> u64 {
    (hash ^ value).wrapping_mul(FNV1A_PRIME)
}

fn y_percent_extents_signature(map: &BTreeMap<GridId, BTreeMap<AxisId, (f64, f64)>>) -> u64 {
    let mut h = FNV1A_OFFSET;
    for (grid, axes) in map {
        h = fnv1a_step(h, grid.0);
        for (axis, (min, max)) in axes {
            h = fnv1a_step(h, axis.0);
            h = fnv1a_step(h, min.to_bits());
            h = fnv1a_step(h, max.to_bits());
        }
    }
    h
}
