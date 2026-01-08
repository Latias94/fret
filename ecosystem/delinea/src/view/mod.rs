use crate::data::{DataTable, DatasetStore};
use crate::engine::ChartState;
use crate::engine::model::ChartModel;
use crate::engine::window::DataWindowX;
use crate::ids::{AxisId, DatasetId, Revision, SeriesId};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RowRange {
    pub start: usize,
    pub end: usize,
}

impl RowRange {
    pub fn clamp_to_len(&mut self, len: usize) {
        self.start = self.start.min(len);
        self.end = self.end.min(len);
        if self.end < self.start {
            self.end = self.start;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DatasetView {
    pub dataset: DatasetId,
    pub revision: Revision,
    pub data_revision: Revision,
    pub row_range: RowRange,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SeriesView {
    pub series: SeriesId,
    pub dataset: DatasetId,
    pub x_axis: AxisId,
    pub revision: Revision,
    pub data_revision: Revision,
    pub row_range: RowRange,
}

#[derive(Debug, Default, Clone)]
pub struct ViewState {
    pub revision: Revision,
    pub datasets: Vec<DatasetView>,
    pub series: Vec<SeriesView>,
    last_model_rev: Revision,
    last_data_rev: Revision,
    last_state_rev: Revision,
}

impl ViewState {
    pub fn sync_inputs(&mut self, model: &ChartModel, datasets: &DatasetStore, state: &ChartState) {
        let model_rev = model.revs.spec;
        let data_rev = datasets
            .datasets
            .iter()
            .next()
            .map(|(_, t)| t.revision)
            .unwrap_or_default();

        let state_rev = state.revision;

        if model_rev != self.last_model_rev
            || data_rev != self.last_data_rev
            || state_rev != self.last_state_rev
        {
            self.revision.bump();
            self.last_model_rev = model_rev;
            self.last_data_rev = data_rev;
            self.last_state_rev = state_rev;
        }
    }

    pub fn rebuild(&mut self, model: &ChartModel, datasets: &DatasetStore, state: &ChartState) {
        self.datasets.clear();
        self.series.clear();
        for (id, _dataset_model) in &model.datasets {
            let table = datasets
                .datasets
                .iter()
                .find_map(|(did, t)| (*did == *id).then_some(t));
            let Some(table) = table else { continue };
            let mut row_range = state
                .dataset_row_ranges
                .get(id)
                .copied()
                .unwrap_or(RowRange {
                    start: 0,
                    end: table.row_count,
                });
            row_range.clamp_to_len(table.row_count);
            self.datasets.push(DatasetView {
                dataset: *id,
                revision: self.revision,
                data_revision: table.revision,
                row_range,
            });
        }

        for series_id in &model.series_order {
            let Some(series) = model.series.get(series_id) else {
                continue;
            };
            let table = datasets
                .datasets
                .iter()
                .find_map(|(did, t)| (*did == series.dataset).then_some(t));
            let Some(table) = table else { continue };
            let Some(dataset) = model.datasets.get(&series.dataset) else {
                continue;
            };
            let Some(x_col) = dataset.fields.get(&series.encode.x).copied() else {
                continue;
            };
            let Some(x) = table.column_f64(x_col) else {
                continue;
            };

            let mut base_range = state
                .dataset_row_ranges
                .get(&series.dataset)
                .copied()
                .unwrap_or(RowRange {
                    start: 0,
                    end: table.row_count,
                });
            base_range.clamp_to_len(table.row_count);

            let window = state.data_window_x.get(&series.x_axis).copied();
            let row_range = if let Some(window) = window {
                row_range_for_x_window(x, base_range, window)
            } else {
                base_range
            };

            self.series.push(SeriesView {
                series: *series_id,
                dataset: series.dataset,
                x_axis: series.x_axis,
                revision: self.revision,
                data_revision: table.revision,
                row_range,
            });
        }
    }

    pub fn dataset_view(&self, dataset: DatasetId) -> Option<&DatasetView> {
        self.datasets.iter().find(|v| v.dataset == dataset)
    }

    pub fn series_view(&self, series: SeriesId) -> Option<&SeriesView> {
        self.series.iter().find(|v| v.series == series)
    }
}

pub fn table_row_range<'a>(
    table: &'a DataTable,
    view: Option<&DatasetView>,
) -> core::ops::Range<usize> {
    let mut range = RowRange {
        start: 0,
        end: table.row_count,
    };
    if let Some(view) = view {
        range = view.row_range;
    }
    range.clamp_to_len(table.row_count);
    range.start..range.end
}

fn row_range_for_x_window(values: &[f64], base: RowRange, window: DataWindowX) -> RowRange {
    let mut base = base;
    base.clamp_to_len(values.len());
    if base.is_empty() {
        return base;
    }

    let slice = &values[base.start..base.end];
    let Some(first) = slice.first().copied() else {
        return base;
    };
    let Some(last) = slice.last().copied() else {
        return base;
    };
    if !first.is_finite() || !last.is_finite() {
        return row_range_for_x_window_linear(values, base, window);
    }

    let ascending = first <= last;
    if !is_probably_monotonic(slice, ascending) {
        return row_range_for_x_window_linear(values, base, window);
    }

    let (min, max) = if window.min <= window.max {
        (window.min, window.max)
    } else {
        (window.max, window.min)
    };

    let (start, end) = if ascending {
        let lo = slice.partition_point(|&v| v < min);
        let hi = slice.partition_point(|&v| v <= max);
        (base.start + lo, base.start + hi)
    } else {
        let lo = slice.partition_point(|&v| v > max);
        let hi = slice.partition_point(|&v| v >= min);
        (base.start + lo, base.start + hi)
    };

    RowRange { start, end }
}

fn is_probably_monotonic(values: &[f64], ascending: bool) -> bool {
    let len = values.len();
    if len <= 2 {
        return true;
    }

    let samples = 8usize.min(len);
    let mut prev = values[0];
    if !prev.is_finite() {
        return false;
    }

    for s in 1..samples {
        let i = s * (len - 1) / (samples - 1);
        let cur = values[i];
        if !cur.is_finite() {
            return false;
        }
        if ascending {
            if cur < prev {
                return false;
            }
        } else if cur > prev {
            return false;
        }
        prev = cur;
    }
    true
}

fn row_range_for_x_window_linear(values: &[f64], base: RowRange, window: DataWindowX) -> RowRange {
    let (min, max) = if window.min <= window.max {
        (window.min, window.max)
    } else {
        (window.max, window.min)
    };

    let mut first: Option<usize> = None;
    let mut last: Option<usize> = None;
    for i in base.start..base.end {
        let v = values.get(i).copied().unwrap_or(f64::NAN);
        if !v.is_finite() {
            continue;
        }
        if v >= min && v <= max {
            first.get_or_insert(i);
            last = Some(i);
        }
    }

    match (first, last) {
        (Some(a), Some(b)) if b >= a => RowRange {
            start: a,
            end: b + 1,
        },
        _ => RowRange {
            start: base.start,
            end: base.start,
        },
    }
}
