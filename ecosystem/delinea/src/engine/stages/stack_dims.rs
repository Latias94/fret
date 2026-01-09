use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::data::DatasetStore;
use crate::engine::model::ChartModel;
use crate::ids::{DatasetId, Revision, SeriesId, StackId};
use crate::scheduler::WorkBudget;
use crate::spec::StackStrategy;

#[derive(Debug, Default, Clone)]
pub struct StackDimsStage {
    requested: Vec<StackId>,
    requested_set: BTreeSet<StackId>,
    cursor: usize,
    cache: BTreeMap<StackId, StackGroupEntry>,
}

#[derive(Debug, Clone)]
struct StackSeriesInput {
    series: SeriesId,
    y_col: usize,
}

#[derive(Debug, Clone)]
enum StackGroupEntry {
    Ready {
        model_rev: Revision,
        data_rev: Revision,
        dataset: DatasetId,
        row_count: usize,
        strategy: StackStrategy,
        bases: BTreeMap<SeriesId, Arc<[f64]>>,
    },
    Building {
        model_rev: Revision,
        data_rev: Revision,
        dataset: DatasetId,
        row_count: usize,
        strategy: StackStrategy,
        series: Vec<StackSeriesInput>,
        series_index: usize,
        next_row: usize,
        accum_pos: Vec<f64>,
        accum_neg: Vec<f64>,
        current_base: Vec<f64>,
        bases: BTreeMap<SeriesId, Arc<[f64]>>,
    },
}

impl StackDimsStage {
    pub fn begin_frame(&mut self) {
        self.requested.clear();
        self.requested_set.clear();
        self.cursor = 0;
    }

    pub fn request(&mut self, stack: StackId) -> bool {
        if !self.requested_set.insert(stack) {
            return false;
        }
        self.requested.push(stack);
        true
    }

    pub fn request_for_visible_stacks(&mut self, model: &ChartModel) {
        for series in model.series_in_order() {
            if series.visible
                && let Some(stack) = series.stack
            {
                self.request(stack);
            }
        }
    }

    pub fn prepare_requests(&mut self, model: &ChartModel, datasets: &DatasetStore) {
        self.cache.retain(|k, _| self.requested_set.contains(k));

        for &stack in &self.requested {
            let model_rev = model.revs.marks;

            let Some((dataset_id, strategy, series_inputs)) =
                stack_group_inputs(model, stack, datasets)
            else {
                self.cache.remove(&stack);
                continue;
            };

            let Some(table) = datasets.dataset(dataset_id) else {
                self.cache.remove(&stack);
                continue;
            };

            let data_rev = table.revision;
            let row_count = table.row_count;

            match self.cache.get(&stack) {
                Some(StackGroupEntry::Ready {
                    model_rev: mr,
                    data_rev: dr,
                    dataset,
                    row_count: rc,
                    strategy: st,
                    ..
                }) if *mr == model_rev
                    && *dr == data_rev
                    && *dataset == dataset_id
                    && *rc == row_count
                    && *st == strategy => {}
                _ => {
                    self.cache.insert(
                        stack,
                        StackGroupEntry::Building {
                            model_rev,
                            data_rev,
                            dataset: dataset_id,
                            row_count,
                            strategy,
                            series: series_inputs,
                            series_index: 0,
                            next_row: 0,
                            accum_pos: vec![0.0; row_count],
                            accum_neg: if strategy == StackStrategy::SameSign {
                                vec![0.0; row_count]
                            } else {
                                Vec::new()
                            },
                            current_base: vec![0.0; row_count],
                            bases: BTreeMap::new(),
                        },
                    );
                }
            }
        }
    }

    pub fn step(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        budget: &mut WorkBudget,
    ) -> bool {
        if self.requested.is_empty() {
            return true;
        }

        while self.cursor < self.requested.len() {
            let stack = self.requested[self.cursor];

            let Some(entry) = self.cache.get_mut(&stack) else {
                self.cursor += 1;
                continue;
            };

            let StackGroupEntry::Building {
                model_rev,
                data_rev,
                dataset,
                row_count,
                strategy,
                series,
                series_index,
                next_row,
                accum_pos,
                accum_neg,
                current_base,
                bases,
            } = entry
            else {
                self.cursor += 1;
                continue;
            };

            if *model_rev != model.revs.marks {
                self.cursor += 1;
                continue;
            }

            let Some(table) = datasets.dataset(*dataset) else {
                self.cursor += 1;
                continue;
            };

            if table.revision != *data_rev || table.row_count != *row_count {
                self.cursor += 1;
                continue;
            }

            while *series_index < series.len() {
                let points_budget = budget.take_points(8192) as usize;
                if points_budget == 0 {
                    return false;
                }

                let input = series[*series_index].clone();
                let Some(y) = table.column_f64(input.y_col) else {
                    self.cursor += 1;
                    break;
                };

                let end = (*next_row + points_budget).min(*row_count);
                for i in *next_row..end {
                    let yi = y.get(i).copied().unwrap_or(f64::NAN);
                    if !yi.is_finite() {
                        current_base[i] = 0.0;
                        continue;
                    }

                    let base = stack_base_for_row(*strategy, accum_pos, accum_neg, yi, i);
                    current_base[i] = base;
                    stack_apply_row(*strategy, accum_pos, accum_neg, yi, i);
                }

                *next_row = end;

                if *next_row >= *row_count {
                    let mut finished = Vec::new();
                    std::mem::swap(&mut finished, current_base);
                    bases.insert(input.series, Arc::from(finished.into_boxed_slice()));

                    *next_row = 0;
                    *series_index += 1;
                    current_base.resize(*row_count, 0.0);
                }
            }

            if *series_index >= series.len() {
                let ready = StackGroupEntry::Ready {
                    model_rev: *model_rev,
                    data_rev: *data_rev,
                    dataset: *dataset,
                    row_count: *row_count,
                    strategy: *strategy,
                    bases: bases.clone(),
                };
                self.cache.insert(stack, ready);
            }

            self.cursor += 1;
        }

        true
    }

    pub fn stack_base(
        &self,
        stack: StackId,
        series: SeriesId,
        raw_index: usize,
        model_rev: Revision,
        data_rev: Revision,
    ) -> Option<f64> {
        match self.cache.get(&stack) {
            Some(StackGroupEntry::Ready {
                model_rev: mr,
                data_rev: dr,
                bases,
                ..
            }) if *mr == model_rev && *dr == data_rev => {
                bases.get(&series).and_then(|b| b.get(raw_index).copied())
            }
            _ => None,
        }
    }
}

fn stack_group_inputs(
    model: &ChartModel,
    stack: StackId,
    datasets: &DatasetStore,
) -> Option<(DatasetId, StackStrategy, Vec<StackSeriesInput>)> {
    let mut dataset_id: Option<DatasetId> = None;
    let mut strategy: Option<StackStrategy> = None;
    let mut inputs = Vec::<StackSeriesInput>::new();

    for series in model.series_in_order() {
        if !series.visible || series.stack != Some(stack) {
            continue;
        }
        dataset_id.get_or_insert(series.dataset);
        strategy.get_or_insert(series.stack_strategy);

        if dataset_id != Some(series.dataset) || strategy != Some(series.stack_strategy) {
            return None;
        }

        let Some(table) = datasets.dataset(series.dataset) else {
            return None;
        };
        let Some(dataset) = model.datasets.get(&series.dataset) else {
            return None;
        };
        let y_col = *dataset.fields.get(&series.encode.y)?;
        if table.column_f64(y_col).is_none() {
            return None;
        }

        inputs.push(StackSeriesInput {
            series: series.id,
            y_col,
        });
    }

    let dataset_id = dataset_id?;
    let strategy = strategy.unwrap_or_default();
    Some((dataset_id, strategy, inputs))
}

fn stack_base_for_row(
    strategy: StackStrategy,
    accum_pos: &[f64],
    accum_neg: &[f64],
    y: f64,
    index: usize,
) -> f64 {
    match strategy {
        StackStrategy::All => accum_pos.get(index).copied().unwrap_or(0.0),
        StackStrategy::SameSign => {
            if y >= 0.0 {
                accum_pos.get(index).copied().unwrap_or(0.0)
            } else {
                accum_neg.get(index).copied().unwrap_or(0.0)
            }
        }
    }
}

fn stack_apply_row(
    strategy: StackStrategy,
    accum_pos: &mut [f64],
    accum_neg: &mut [f64],
    y: f64,
    index: usize,
) {
    match strategy {
        StackStrategy::All => {
            if let Some(sum) = accum_pos.get_mut(index) {
                *sum += y;
            }
        }
        StackStrategy::SameSign => {
            if y >= 0.0 {
                if let Some(sum) = accum_pos.get_mut(index) {
                    *sum += y;
                }
            } else if let Some(sum) = accum_neg.get_mut(index) {
                *sum += y;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::data::{Column, DataTable};
    use crate::engine::model::ChartModel;
    use crate::ids::{AxisId, ChartId, DatasetId, FieldId, GridId, SeriesId, StackId};
    use crate::spec::{
        AxisKind, AxisSpec, ChartSpec, DatasetSpec, FieldSpec, GridSpec, SeriesEncode, SeriesKind,
        SeriesSpec,
    };

    #[test]
    fn stack_dims_stage_computes_series_base_arrays() {
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
        let x_field = FieldId::new(1);
        let y_a_field = FieldId::new(2);
        let y_b_field = FieldId::new(3);
        let stack = StackId::new(1);
        let a = SeriesId::new(1);
        let b = SeriesId::new(2);

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
                        id: y_a_field,
                        column: 1,
                    },
                    FieldSpec {
                        id: y_b_field,
                        column: 2,
                    },
                ],
            }],
            grids: vec![GridSpec { id: grid_id }],
            axes: vec![
                AxisSpec {
                    id: x_axis,
                    name: None,
                    kind: AxisKind::X,
                    grid: grid_id,
                    position: None,
                    scale: Default::default(),
                    range: None,
                },
                AxisSpec {
                    id: y_axis,
                    name: None,
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: None,
                    scale: Default::default(),
                    range: None,
                },
            ],
            data_zoom_x: vec![],
            axis_pointer: None,
            series: vec![
                SeriesSpec {
                    id: a,
                    name: None,
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_a_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: Some(stack),
                    stack_strategy: Default::default(),
                    area_baseline: None,
                },
                SeriesSpec {
                    id: b,
                    name: None,
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_b_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: Some(stack),
                    stack_strategy: Default::default(),
                    area_baseline: None,
                },
            ],
        };

        let model = ChartModel::from_spec(spec).unwrap();

        let mut datasets = DatasetStore::default();
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![0.0, 1.0, 2.0]));
        table.push_column(Column::F64(vec![1.0, 2.0, 3.0]));
        table.push_column(Column::F64(vec![10.0, 20.0, 30.0]));
        let data_rev = table.revision;
        datasets.insert(dataset_id, table);

        let mut stage = StackDimsStage::default();
        stage.begin_frame();
        stage.request_for_visible_stacks(&model);
        stage.prepare_requests(&model, &datasets);

        let mut budget = WorkBudget::new(1_000_000, 0, 0);
        assert!(stage.step(&model, &datasets, &mut budget));

        // For series A, base is always 0.
        assert_eq!(
            stage.stack_base(stack, a, 0, model.revs.marks, data_rev),
            Some(0.0)
        );
        assert_eq!(
            stage.stack_base(stack, a, 2, model.revs.marks, data_rev),
            Some(0.0)
        );

        // For series B, base equals A's y at each index.
        assert_eq!(
            stage.stack_base(stack, b, 0, model.revs.marks, data_rev),
            Some(1.0)
        );
        assert_eq!(
            stage.stack_base(stack, b, 1, model.revs.marks, data_rev),
            Some(2.0)
        );
        assert_eq!(
            stage.stack_base(stack, b, 2, model.revs.marks, data_rev),
            Some(3.0)
        );
    }
}
