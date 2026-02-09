use crate::data::DatasetStore;
use crate::engine::model::ChartModel;
use crate::ids::{DatasetId, SeriesId, StackId};
use crate::spec::StackStrategy;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StackBaseAtIndex {
    pub base: f64,
    pub strategy: StackStrategy,
}

pub fn stack_group_for_series(model: &ChartModel, series: SeriesId) -> Option<StackId> {
    model.series.get(&series).and_then(|s| s.stack)
}

pub fn stacked_y_at_index(
    model: &ChartModel,
    datasets: &DatasetStore,
    series: SeriesId,
    index: usize,
    y: f64,
) -> Option<f64> {
    let base = stack_base_at_index(model, datasets, series, index, y)?.base;
    Some(y + base)
}

pub fn stack_base_at_index(
    model: &ChartModel,
    datasets: &DatasetStore,
    series_id: SeriesId,
    index: usize,
    y: f64,
) -> Option<StackBaseAtIndex> {
    let series = model.series.get(&series_id)?;
    let Some(stack) = series.stack else {
        return Some(StackBaseAtIndex {
            base: 0.0,
            strategy: StackStrategy::SameSign,
        });
    };

    let strategy = series.stack_strategy;

    let mut sum = 0.0f64;
    let mut pos = 0.0f64;
    let mut neg = 0.0f64;

    for id in &model.series_order {
        if *id == series_id {
            break;
        }
        let Some(s) = model.series.get(id) else {
            continue;
        };
        if !s.visible {
            continue;
        }
        if s.stack != Some(stack) {
            continue;
        }
        if s.dataset != series.dataset
            || s.x_axis != series.x_axis
            || s.y_axis != series.y_axis
            || s.encode.x != series.encode.x
            || s.stack_strategy != series.stack_strategy
        {
            continue;
        }

        let yi = series_y_at_index(model, datasets, s.dataset, *id, index)?;
        if !yi.is_finite() {
            continue;
        }

        match strategy {
            StackStrategy::All => sum += yi,
            StackStrategy::SameSign => {
                if yi >= 0.0 {
                    pos += yi;
                } else {
                    neg += yi;
                }
            }
        }
    }

    let base = match strategy {
        StackStrategy::All => sum,
        StackStrategy::SameSign => {
            if y >= 0.0 {
                pos
            } else {
                neg
            }
        }
    };

    Some(StackBaseAtIndex { base, strategy })
}

fn series_y_at_index(
    model: &ChartModel,
    datasets: &DatasetStore,
    dataset_id: DatasetId,
    series_id: SeriesId,
    index: usize,
) -> Option<f64> {
    let series = model.series.get(&series_id)?;
    let dataset = model.datasets.get(&dataset_id)?;
    let y_col = *dataset.fields.get(&series.encode.y)?;

    let table = datasets.dataset(model.root_dataset_id(dataset_id))?;
    let y = table.column_f64(y_col)?;
    y.get(index).copied()
}
