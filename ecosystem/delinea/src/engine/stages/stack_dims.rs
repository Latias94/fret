use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::data::DatasetStore;
use crate::engine::bar::bar_mapping_for_series;
use crate::engine::model::ChartModel;
use crate::ids::{DatasetId, Revision, SeriesId, StackId};
use crate::scale::AxisScale;
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
    value_col: usize,
}

#[derive(Debug, Clone)]
enum StackIndexing {
    ByRowIndex {
        accum_pos: Vec<f64>,
        accum_neg: Vec<f64>,
    },
    ByOrdinal {
        category_col: usize,
        accum_pos: Vec<f64>,
        accum_neg: Vec<f64>,
    },
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
        stacked: BTreeMap<SeriesId, Arc<[f64]>>,
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
        indexing: StackIndexing,
        current_base: Vec<f64>,
        current_stacked: Vec<f64>,
        bases: BTreeMap<SeriesId, Arc<[f64]>>,
        stacked: BTreeMap<SeriesId, Arc<[f64]>>,
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

            let Some(inputs) = stack_group_inputs(model, stack, datasets) else {
                self.cache.remove(&stack);
                continue;
            };

            let Some(table) = datasets.dataset(model.root_dataset_id(inputs.dataset_id)) else {
                self.cache.remove(&stack);
                continue;
            };

            let data_rev = table.revision();
            let row_count = table.row_count();

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
                    && *dataset == inputs.dataset_id
                    && *rc == row_count
                    && *st == inputs.strategy => {}
                _ => {
                    let indexing = stack_indexing_for_group(model, datasets, &inputs, row_count);
                    self.cache.insert(
                        stack,
                        StackGroupEntry::Building {
                            model_rev,
                            data_rev,
                            dataset: inputs.dataset_id,
                            row_count,
                            strategy: inputs.strategy,
                            series: inputs.series,
                            series_index: 0,
                            next_row: 0,
                            indexing,
                            current_base: vec![0.0; row_count],
                            current_stacked: vec![0.0; row_count],
                            bases: BTreeMap::new(),
                            stacked: BTreeMap::new(),
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
                indexing,
                current_base,
                current_stacked,
                bases,
                stacked,
            } = entry
            else {
                self.cursor += 1;
                continue;
            };

            if *model_rev != model.revs.marks {
                self.cursor += 1;
                continue;
            }

            let Some(table) = datasets.dataset(model.root_dataset_id(*dataset)) else {
                self.cursor += 1;
                continue;
            };

            if table.revision() != *data_rev || table.row_count() != *row_count {
                self.cursor += 1;
                continue;
            }

            while *series_index < series.len() {
                let points_budget = budget.take_points(8192) as usize;
                if points_budget == 0 {
                    return false;
                }

                let input = series[*series_index].clone();
                let Some(y) = table.column_f64(input.value_col) else {
                    self.cursor += 1;
                    break;
                };

                let x = match indexing {
                    StackIndexing::ByRowIndex { .. } => None,
                    StackIndexing::ByOrdinal { category_col, .. } => {
                        table.column_f64(*category_col)
                    }
                };

                let end = (*next_row + points_budget).min(*row_count);
                for i in *next_row..end {
                    let yi = y.get(i).copied().unwrap_or(f64::NAN);
                    if !yi.is_finite() {
                        current_base[i] = f64::NAN;
                        current_stacked[i] = f64::NAN;
                        continue;
                    }

                    let index = match indexing {
                        StackIndexing::ByRowIndex { .. } => Some(i),
                        StackIndexing::ByOrdinal { accum_pos, .. } => {
                            if let Some(x) = x {
                                let xv = x.get(i).copied().unwrap_or(f64::NAN);
                                if !xv.is_finite() {
                                    None
                                } else {
                                    let ord = xv.round() as i64;
                                    if ord < 0 || ord as usize >= accum_pos.len() {
                                        None
                                    } else {
                                        Some(ord as usize)
                                    }
                                }
                            } else {
                                None
                            }
                        }
                    };

                    let Some(index) = index else {
                        current_base[i] = 0.0;
                        current_stacked[i] = yi;
                        continue;
                    };

                    let base = match indexing {
                        StackIndexing::ByRowIndex {
                            accum_pos,
                            accum_neg,
                        } => stack_base_for_row(*strategy, accum_pos, accum_neg, yi, index),
                        StackIndexing::ByOrdinal {
                            accum_pos,
                            accum_neg,
                            ..
                        } => stack_base_for_row(*strategy, accum_pos, accum_neg, yi, index),
                    };

                    current_base[i] = base;
                    current_stacked[i] = yi + base;

                    match indexing {
                        StackIndexing::ByRowIndex {
                            accum_pos,
                            accum_neg,
                        } => stack_apply_row(*strategy, accum_pos, accum_neg, yi, index),
                        StackIndexing::ByOrdinal {
                            accum_pos,
                            accum_neg,
                            ..
                        } => stack_apply_row(*strategy, accum_pos, accum_neg, yi, index),
                    }
                }

                *next_row = end;

                if *next_row >= *row_count {
                    let mut finished_base = Vec::new();
                    std::mem::swap(&mut finished_base, current_base);
                    bases.insert(input.series, Arc::from(finished_base.into_boxed_slice()));

                    let mut finished_stacked = Vec::new();
                    std::mem::swap(&mut finished_stacked, current_stacked);
                    stacked.insert(input.series, Arc::from(finished_stacked.into_boxed_slice()));

                    *next_row = 0;
                    *series_index += 1;
                    current_base.resize(*row_count, 0.0);
                    current_stacked.resize(*row_count, 0.0);
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
                    stacked: stacked.clone(),
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

    pub fn stack_arrays(
        &self,
        stack: StackId,
        series: SeriesId,
        model_rev: Revision,
        data_rev: Revision,
    ) -> Option<StackSeriesArrays> {
        match self.cache.get(&stack) {
            Some(StackGroupEntry::Ready {
                model_rev: mr,
                data_rev: dr,
                bases,
                stacked,
                ..
            }) if *mr == model_rev && *dr == data_rev => {
                let base = bases.get(&series)?.clone();
                let stacked = stacked.get(&series)?.clone();
                Some(StackSeriesArrays { base, stacked })
            }
            _ => None,
        }
    }

    pub fn stacked_y(
        &self,
        stack: StackId,
        series: SeriesId,
        raw_index: usize,
        model_rev: Revision,
        data_rev: Revision,
    ) -> Option<f64> {
        self.stacked_value(stack, series, raw_index, model_rev, data_rev)
    }

    pub fn stacked_value(
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
                stacked,
                ..
            }) if *mr == model_rev && *dr == data_rev => {
                stacked.get(&series).and_then(|b| b.get(raw_index).copied())
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StackSeriesArrays {
    pub base: Arc<[f64]>,
    pub stacked: Arc<[f64]>,
}

#[derive(Debug, Clone)]
struct StackGroupInputs {
    dataset_id: DatasetId,
    category_axis: crate::ids::AxisId,
    category_col: usize,
    strategy: StackStrategy,
    series: Vec<StackSeriesInput>,
}

fn stack_group_inputs(
    model: &ChartModel,
    stack: StackId,
    datasets: &DatasetStore,
) -> Option<StackGroupInputs> {
    let mut dataset_id: Option<DatasetId> = None;
    let mut strategy: Option<StackStrategy> = None;
    let mut category_axis: Option<crate::ids::AxisId> = None;
    let mut value_axis: Option<crate::ids::AxisId> = None;
    let mut category_col: Option<usize> = None;
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

        let Some(table) = datasets.dataset(model.root_dataset_id(series.dataset)) else {
            return None;
        };
        let Some(dataset) = model.datasets.get(&series.dataset) else {
            return None;
        };

        let (category_axis_id, value_axis_id, series_category_col, series_value_col) =
            if series.kind == crate::spec::SeriesKind::Bar {
                let mapping = bar_mapping_for_series(model, series.id)?;
                (
                    mapping.category_axis,
                    mapping.value_axis,
                    *dataset.fields.get(&mapping.category_field)?,
                    *dataset.fields.get(&mapping.value_field)?,
                )
            } else {
                (
                    series.x_axis,
                    series.y_axis,
                    *dataset.fields.get(&series.encode.x)?,
                    *dataset.fields.get(&series.encode.y)?,
                )
            };

        category_axis.get_or_insert(category_axis_id);
        value_axis.get_or_insert(value_axis_id);
        category_col.get_or_insert(series_category_col);

        if category_axis != Some(category_axis_id)
            || value_axis != Some(value_axis_id)
            || category_col != Some(series_category_col)
        {
            return None;
        }

        if table.column_f64(series_category_col).is_none()
            || table.column_f64(series_value_col).is_none()
        {
            return None;
        }

        inputs.push(StackSeriesInput {
            series: series.id,
            value_col: series_value_col,
        });
    }

    let dataset_id = dataset_id?;
    let strategy = strategy.unwrap_or_default();
    let category_axis = category_axis?;
    let category_col = category_col?;
    Some(StackGroupInputs {
        dataset_id,
        category_axis,
        category_col,
        strategy,
        series: inputs,
    })
}

fn stack_indexing_for_group(
    model: &ChartModel,
    datasets: &DatasetStore,
    inputs: &StackGroupInputs,
    row_count: usize,
) -> StackIndexing {
    let Some(table) = datasets.dataset(model.root_dataset_id(inputs.dataset_id)) else {
        return StackIndexing::ByRowIndex {
            accum_pos: vec![0.0; row_count],
            accum_neg: if inputs.strategy == StackStrategy::SameSign {
                vec![0.0; row_count]
            } else {
                Vec::new()
            },
        };
    };

    let axis = model.axes.get(&inputs.category_axis);
    let ordinal_len = axis.and_then(|a| match &a.scale {
        AxisScale::Category(scale) => Some(scale.len()),
        _ => None,
    });

    if let Some(ordinal_len) = ordinal_len
        && ordinal_len > 0
        && table.column_f64(inputs.category_col).is_some()
    {
        return StackIndexing::ByOrdinal {
            category_col: inputs.category_col,
            accum_pos: vec![0.0; ordinal_len],
            accum_neg: if inputs.strategy == StackStrategy::SameSign {
                vec![0.0; ordinal_len]
            } else {
                Vec::new()
            },
        };
    }

    StackIndexing::ByRowIndex {
        accum_pos: vec![0.0; row_count],
        accum_neg: if inputs.strategy == StackStrategy::SameSign {
            vec![0.0; row_count]
        } else {
            Vec::new()
        },
    }
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

                from: None,
                transforms: Vec::new(),
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
            data_zoom_y: vec![],
            tooltip: None,
            axis_pointer: None,
            visual_maps: vec![],
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
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
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
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
            ],
        };

        let model = ChartModel::from_spec(spec).unwrap();

        let mut datasets = DatasetStore::default();
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![0.0, 1.0, 2.0]));
        table.push_column(Column::F64(vec![1.0, 2.0, 3.0]));
        table.push_column(Column::F64(vec![10.0, 20.0, 30.0]));
        let data_rev = table.revision();
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

    #[test]
    fn stack_dims_stage_uses_ordinal_accumulation_on_category_axes() {
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

                from: None,
                transforms: Vec::new(),
            }],
            grids: vec![GridSpec { id: grid_id }],
            axes: vec![
                AxisSpec {
                    id: x_axis,
                    name: None,
                    kind: AxisKind::X,
                    grid: grid_id,
                    position: None,
                    scale: crate::scale::AxisScale::Category(crate::scale::CategoryAxisScale {
                        categories: vec!["a".into(), "b".into(), "c".into()],
                    }),
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
            data_zoom_y: vec![],
            tooltip: None,
            axis_pointer: None,
            visual_maps: vec![],
            series: vec![
                SeriesSpec {
                    id: a,
                    name: None,
                    kind: SeriesKind::Bar,
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
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
                SeriesSpec {
                    id: b,
                    name: None,
                    kind: SeriesKind::Bar,
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
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
            ],
        };

        let model = ChartModel::from_spec(spec).unwrap();

        let mut datasets = DatasetStore::default();
        let mut table = DataTable::default();
        // Category ordinals with a shuffled raw order.
        table.push_column(Column::F64(vec![0.0, 2.0, 1.0]));
        table.push_column(Column::F64(vec![1.0, 2.0, 3.0]));
        table.push_column(Column::F64(vec![10.0, 20.0, 30.0]));
        let data_rev = table.revision();
        datasets.insert(dataset_id, table);

        let mut stage = StackDimsStage::default();
        stage.begin_frame();
        stage.request_for_visible_stacks(&model);
        stage.prepare_requests(&model, &datasets);

        let mut budget = WorkBudget::new(1_000_000, 0, 0);
        assert!(stage.step(&model, &datasets, &mut budget));

        assert_eq!(
            stage.stack_base(stack, a, 0, model.revs.marks, data_rev),
            Some(0.0)
        );
        assert_eq!(
            stage.stack_base(stack, a, 2, model.revs.marks, data_rev),
            Some(0.0)
        );

        // For series B, the base should match series A's y per category ordinal.
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

        assert_eq!(
            stage.stacked_y(stack, b, 2, model.revs.marks, data_rev),
            Some(33.0)
        );
    }
}
