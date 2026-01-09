use std::collections::{BTreeMap, BTreeSet};

use crate::data::DatasetStore;
use crate::engine::model::ChartModel;
use crate::ids::{DatasetId, Revision, SeriesId};
use crate::scheduler::WorkBudget;
use crate::spec::SeriesKind;

#[derive(Debug, Default, Clone)]
pub struct BarLayoutStage {
    requested: Vec<BarLayoutGroupKey>,
    requested_set: BTreeSet<BarLayoutGroupKey>,
    cursor: usize,
    cache: BTreeMap<BarLayoutGroupKey, BarLayoutGroupEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BarLayoutGroupKey {
    dataset: DatasetId,
    x_axis: crate::ids::AxisId,
    y_axis: crate::ids::AxisId,
    x_col: u32,
}

#[derive(Debug, Clone)]
enum BarLayoutGroupEntry {
    Ready {
        model_rev: Revision,
        layouts: BTreeMap<SeriesId, BarSeriesLayout>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarSeriesLayout {
    /// X offset in data space, relative to the category ordinal center.
    pub offset_x: f64,
    /// Bar width in data space (category bands have width 1.0).
    pub width_x: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum BarSlotKey {
    Stack(crate::ids::StackId),
    Series(SeriesId),
}

impl BarLayoutStage {
    pub fn begin_frame(&mut self) {
        self.requested.clear();
        self.requested_set.clear();
        self.cursor = 0;
    }

    pub fn request_for_visible_bars(&mut self, model: &ChartModel) {
        for series in model.series_in_order() {
            if !series.visible || series.kind != SeriesKind::Bar {
                continue;
            }

            let Some(dataset) = model.datasets.get(&series.dataset) else {
                continue;
            };
            let Some(x_col) = dataset.fields.get(&series.encode.x).copied() else {
                continue;
            };

            let key = BarLayoutGroupKey {
                dataset: series.dataset,
                x_axis: series.x_axis,
                y_axis: series.y_axis,
                x_col: x_col.min(u32::MAX as usize) as u32,
            };
            if self.requested_set.insert(key) {
                self.requested.push(key);
            }
        }
    }

    pub fn prepare_requests(&mut self) {
        self.cache.retain(|k, _| self.requested_set.contains(k));
    }

    pub fn step(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        budget: &mut WorkBudget,
    ) -> bool {
        while self.cursor < self.requested.len() {
            let key = self.requested[self.cursor];
            let model_rev = model.revs.marks;

            if let Some(BarLayoutGroupEntry::Ready { model_rev: r, .. }) = self.cache.get(&key)
                && *r == model_rev
            {
                self.cursor += 1;
                continue;
            }

            if budget.take_marks(1) == 0 {
                return false;
            }

            let layouts = build_layouts_for_group(model, datasets, key);
            self.cache
                .insert(key, BarLayoutGroupEntry::Ready { model_rev, layouts });
            self.cursor += 1;
        }

        true
    }

    pub fn layout_for_series(
        &self,
        model: &ChartModel,
        series_id: SeriesId,
        x_col: usize,
    ) -> Option<BarSeriesLayout> {
        let series = model.series.get(&series_id)?;
        let key = BarLayoutGroupKey {
            dataset: series.dataset,
            x_axis: series.x_axis,
            y_axis: series.y_axis,
            x_col: x_col.min(u32::MAX as usize) as u32,
        };

        match self.cache.get(&key) {
            Some(BarLayoutGroupEntry::Ready { model_rev, layouts })
                if *model_rev == model.revs.marks =>
            {
                layouts.get(&series_id).copied()
            }
            _ => None,
        }
    }
}

fn build_layouts_for_group(
    model: &ChartModel,
    datasets: &DatasetStore,
    key: BarLayoutGroupKey,
) -> BTreeMap<SeriesId, BarSeriesLayout> {
    let mut series_in_group = Vec::new();
    for series in model.series_in_order() {
        if !series.visible || series.kind != SeriesKind::Bar {
            continue;
        }
        if series.dataset != key.dataset
            || series.x_axis != key.x_axis
            || series.y_axis != key.y_axis
        {
            continue;
        }
        let Some(dataset) = model.datasets.get(&series.dataset) else {
            continue;
        };
        let Some(x_col) = dataset.fields.get(&series.encode.x).copied() else {
            continue;
        };
        if x_col != key.x_col as usize {
            continue;
        }
        if datasets.dataset(series.dataset).is_none() {
            continue;
        }
        series_in_group.push(series.id);
    }

    let mut slots = Vec::<BarSlotKey>::new();
    let mut slot_for_series = BTreeMap::<SeriesId, usize>::new();

    for series_id in &series_in_group {
        let Some(series) = model.series.get(series_id) else {
            continue;
        };
        let slot_key = series
            .stack
            .map(BarSlotKey::Stack)
            .unwrap_or(BarSlotKey::Series(series.id));
        let slot_index = match slots.iter().position(|k| *k == slot_key) {
            Some(i) => i,
            None => {
                slots.push(slot_key);
                slots.len() - 1
            }
        };
        slot_for_series.insert(series.id, slot_index);
    }

    let slot_count = slots.len();
    if slot_count == 0 {
        return BTreeMap::new();
    }

    // ECharts-inspired defaults:
    // - `barCategoryGap`: 20% of the band reserved as outer padding
    // - `barGap`: 30% of bar width as spacing between slots
    let category_gap = 0.2_f64.clamp(0.0, 0.95);
    let bar_gap = 0.3_f64.clamp(0.0, 4.0);

    let group_width = (1.0 - category_gap).clamp(0.0, 1.0);
    let denom = slot_count as f64 + (slot_count.saturating_sub(1) as f64) * bar_gap;
    let bar_width = if denom > 0.0 {
        group_width / denom
    } else {
        group_width
    };
    let gap_width = bar_width * bar_gap;

    let group_left = -0.5 * group_width;

    let mut layouts = BTreeMap::<SeriesId, BarSeriesLayout>::new();
    for series_id in series_in_group {
        let slot_index = slot_for_series.get(&series_id).copied().unwrap_or(0);
        let slot_left = group_left + (slot_index as f64) * (bar_width + gap_width);
        let offset_x = slot_left + 0.5 * bar_width;
        layouts.insert(
            series_id,
            BarSeriesLayout {
                offset_x,
                width_x: bar_width,
            },
        );
    }

    layouts
}
