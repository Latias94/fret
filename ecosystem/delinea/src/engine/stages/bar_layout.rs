use std::collections::{BTreeMap, BTreeSet};

use crate::data::DatasetStore;
use crate::engine::bar::bar_mapping_for_series;
use crate::engine::model::ChartModel;
use crate::ids::{DatasetId, Revision, SeriesId};
use crate::scheduler::WorkBudget;
use crate::spec::{BarWidthSpec, SeriesKind};

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
    category_axis: crate::ids::AxisId,
    value_axis: crate::ids::AxisId,
    category_col: u32,
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
    /// Offset in category data space, relative to the category ordinal center.
    pub offset_cat: f64,
    /// Bar thickness in category data space (category bands have width 1.0).
    pub width_cat: f64,
    /// Slot index within the group (0..slot_count-1).
    pub slot_index: u16,
    /// Slot count within the group.
    pub slot_count: u16,
    /// Resolved bar width preference for the group.
    ///
    /// `None` means auto width.
    pub bar_width: Option<BarWidthSpec>,
    /// Resolved bar gap ratio for the group.
    pub bar_gap: f64,
    /// Resolved category gap ratio for the group.
    pub bar_category_gap: f64,
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

            let Some(mapping) = bar_mapping_for_series(model, series.id) else {
                continue;
            };
            let Some(dataset) = model.datasets.get(&series.dataset) else {
                continue;
            };
            let Some(category_col) = dataset.fields.get(&mapping.category_field).copied() else {
                continue;
            };

            let key = BarLayoutGroupKey {
                dataset: series.dataset,
                category_axis: mapping.category_axis,
                value_axis: mapping.value_axis,
                category_col: category_col.min(u32::MAX as usize) as u32,
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
    ) -> Option<BarSeriesLayout> {
        let series = model.series.get(&series_id)?;
        let mapping = bar_mapping_for_series(model, series_id)?;
        let dataset = model.datasets.get(&series.dataset)?;
        let category_col = *dataset.fields.get(&mapping.category_field)?;
        let key = BarLayoutGroupKey {
            dataset: series.dataset,
            category_axis: mapping.category_axis,
            value_axis: mapping.value_axis,
            category_col: category_col.min(u32::MAX as usize) as u32,
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
    let mut requested_bar_width: Option<BarWidthSpec> = None;
    let mut requested_bar_gap: Option<f64> = None;
    let mut requested_category_gap: Option<f64> = None;
    for series in model.series_in_order() {
        if !series.visible || series.kind != SeriesKind::Bar {
            continue;
        }
        if series.dataset != key.dataset {
            continue;
        }
        let Some(mapping) = bar_mapping_for_series(model, series.id) else {
            continue;
        };
        if mapping.category_axis != key.category_axis || mapping.value_axis != key.value_axis {
            continue;
        }
        let Some(dataset) = model.datasets.get(&series.dataset) else {
            continue;
        };
        let Some(category_col) = dataset.fields.get(&mapping.category_field).copied() else {
            continue;
        };
        if category_col != key.category_col as usize {
            continue;
        }
        if datasets
            .dataset(model.root_dataset_id(series.dataset))
            .is_none()
        {
            continue;
        }
        series_in_group.push(series.id);

        if requested_bar_width.is_none() {
            requested_bar_width = sanitize_bar_width(series.bar_layout.bar_width);
        }
        if requested_bar_gap.is_none() {
            requested_bar_gap = sanitize_gap_ratio(series.bar_layout.bar_gap);
        }
        if requested_category_gap.is_none() {
            requested_category_gap = sanitize_band_padding(series.bar_layout.bar_category_gap);
        }
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
    // - `barGap`: 30% of bar width as spacing between slots (can be negative for overlap)
    //
    // In v1, we treat these as group-level settings. If multiple series in the group specify
    // different values, the first specified one in series order wins (deterministic).
    let category_gap = requested_category_gap.unwrap_or(0.2).clamp(0.0, 0.95);
    let bar_gap = requested_bar_gap.unwrap_or(0.3).clamp(-1.0, 4.0);

    let group_width = (1.0 - category_gap).clamp(0.0, 1.0);
    let (bar_width, gap_width) =
        compute_slot_widths(group_width, slot_count, bar_gap, requested_bar_width);

    let group_left = -0.5 * group_width;

    let mut layouts = BTreeMap::<SeriesId, BarSeriesLayout>::new();
    for series_id in series_in_group {
        let slot_index = slot_for_series.get(&series_id).copied().unwrap_or(0);
        let slot_left = group_left + (slot_index as f64) * (bar_width + gap_width);
        let offset_cat = slot_left + 0.5 * bar_width;
        layouts.insert(
            series_id,
            BarSeriesLayout {
                offset_cat,
                width_cat: bar_width,
                slot_index: slot_index.min(u16::MAX as usize) as u16,
                slot_count: slot_count.min(u16::MAX as usize) as u16,
                bar_width: requested_bar_width,
                bar_gap,
                bar_category_gap: category_gap,
            },
        );
    }

    layouts
}

fn sanitize_bar_width(value: Option<BarWidthSpec>) -> Option<BarWidthSpec> {
    let v = value?;
    match v {
        BarWidthSpec::Px(px) => {
            if px.is_finite() && px > 0.0 {
                Some(BarWidthSpec::Px(px))
            } else {
                None
            }
        }
        BarWidthSpec::Band(r) => {
            if r.is_finite() && r > 0.0 {
                Some(BarWidthSpec::Band(r.min(1.0)))
            } else {
                None
            }
        }
    }
}

fn sanitize_band_padding(value: Option<f64>) -> Option<f64> {
    let v = value?;
    if v.is_finite() && v >= 0.0 {
        Some(v.min(0.95))
    } else {
        None
    }
}

fn sanitize_gap_ratio(value: Option<f64>) -> Option<f64> {
    let v = value?;
    if v.is_finite() && v >= -1.0 {
        Some(v.min(4.0))
    } else {
        None
    }
}

fn compute_slot_widths(
    group_width: f64,
    slot_count: usize,
    bar_gap: f64,
    requested_bar_width: Option<BarWidthSpec>,
) -> (f64, f64) {
    if slot_count == 0 {
        return (0.0, 0.0);
    }

    let requested_band = match requested_bar_width {
        Some(BarWidthSpec::Band(r)) => Some(r),
        _ => None,
    };

    let requested = requested_band.unwrap_or(f64::NAN);
    let mut bar_width = if requested.is_finite() {
        requested
    } else {
        let denom = slot_count as f64 + (slot_count.saturating_sub(1) as f64) * bar_gap;
        if denom > 0.0 {
            group_width / denom
        } else {
            group_width
        }
    };

    // If an explicit width would overflow the group, scale down to fit.
    let denom = slot_count as f64 + (slot_count.saturating_sub(1) as f64) * bar_gap;
    let total = bar_width * denom;
    if total.is_finite() && total > group_width && group_width > 0.0 {
        bar_width *= group_width / total;
    }

    let gap_width = bar_width * bar_gap;
    (bar_width, gap_width)
}

#[cfg(test)]
mod tests {
    use crate::data::{Column, DataTable};
    use crate::engine::model::ChartModel;
    use crate::ids::{AxisId, ChartId, DatasetId, FieldId, GridId, SeriesId, StackId};
    use crate::scheduler::WorkBudget;
    use crate::spec::{
        AxisKind, AxisSpec, BarLayoutSpec, ChartSpec, DatasetSpec, FieldSpec, GridSpec,
        SeriesEncode, SeriesKind, SeriesSpec,
    };

    use super::*;

    #[test]
    fn bar_layout_groups_stacked_series_into_one_slot() {
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
        let x_field = FieldId::new(1);
        let y_a_field = FieldId::new(2);
        let y_b_field = FieldId::new(3);
        let y_c_field = FieldId::new(4);
        let stack = StackId::new(1);
        let a = SeriesId::new(1);
        let b = SeriesId::new(2);
        let c = SeriesId::new(3);

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
                    FieldSpec {
                        id: y_c_field,
                        column: 3,
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
                        categories: vec!["A".into()],
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
                SeriesSpec {
                    id: c,
                    name: None,
                    kind: SeriesKind::Bar,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_c_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: None,
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
        table.push_column(Column::F64(vec![0.0]));
        table.push_column(Column::F64(vec![1.0]));
        table.push_column(Column::F64(vec![2.0]));
        table.push_column(Column::F64(vec![3.0]));
        datasets.insert(dataset_id, table);

        let mut stage = BarLayoutStage::default();
        stage.begin_frame();
        stage.request_for_visible_bars(&model);
        stage.prepare_requests();
        assert!(stage.step(&model, &datasets, &mut WorkBudget::new(1_000_000, 0, 64)));

        let layout_a = stage.layout_for_series(&model, a).unwrap();
        let layout_b = stage.layout_for_series(&model, b).unwrap();
        let layout_c = stage.layout_for_series(&model, c).unwrap();

        assert_eq!(layout_a, layout_b);
        assert_ne!(layout_a, layout_c);
    }

    #[test]
    fn bar_layout_respects_explicit_width_and_zero_gaps() {
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
        let x_field = FieldId::new(1);
        let y_a_field = FieldId::new(2);
        let y_b_field = FieldId::new(3);
        let a = SeriesId::new(1);
        let b = SeriesId::new(2);

        let bar_layout = BarLayoutSpec {
            bar_width: Some(crate::spec::BarWidthSpec::Band(0.4)),
            bar_gap: Some(0.0),
            bar_category_gap: Some(0.0),
        };

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
                        categories: vec!["A".into()],
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
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout,
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
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout,
                    area_baseline: None,
                    lod: None,
                },
            ],
        };

        let model = ChartModel::from_spec(spec).unwrap();

        let mut datasets = DatasetStore::default();
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![0.0]));
        table.push_column(Column::F64(vec![1.0]));
        table.push_column(Column::F64(vec![2.0]));
        datasets.insert(dataset_id, table);

        let mut stage = BarLayoutStage::default();
        stage.begin_frame();
        stage.request_for_visible_bars(&model);
        stage.prepare_requests();
        assert!(stage.step(&model, &datasets, &mut WorkBudget::new(1_000_000, 0, 64)));

        let layout_a = stage.layout_for_series(&model, a).unwrap();
        let layout_b = stage.layout_for_series(&model, b).unwrap();

        assert!((layout_a.width_cat - 0.4).abs() < 1e-6);
        assert!((layout_b.width_cat - 0.4).abs() < 1e-6);
        assert!(layout_a.offset_cat < layout_b.offset_cat);
    }

    #[test]
    fn bar_layout_allows_negative_bar_gap_overlap() {
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
        let x_field = FieldId::new(1);
        let y_a_field = FieldId::new(2);
        let y_b_field = FieldId::new(3);
        let a = SeriesId::new(1);
        let b = SeriesId::new(2);

        let bar_layout = BarLayoutSpec {
            bar_width: None,
            bar_gap: Some(-1.0),
            bar_category_gap: Some(0.0),
        };

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
                        categories: vec!["A".into()],
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
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout,
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
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout,
                    area_baseline: None,
                    lod: None,
                },
            ],
        };

        let model = ChartModel::from_spec(spec).unwrap();

        let mut datasets = DatasetStore::default();
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![0.0]));
        table.push_column(Column::F64(vec![1.0]));
        table.push_column(Column::F64(vec![2.0]));
        datasets.insert(dataset_id, table);

        let mut stage = BarLayoutStage::default();
        stage.begin_frame();
        stage.request_for_visible_bars(&model);
        stage.prepare_requests();
        assert!(stage.step(&model, &datasets, &mut WorkBudget::new(1_000_000, 0, 64)));

        let layout_a = stage.layout_for_series(&model, a).unwrap();
        let layout_b = stage.layout_for_series(&model, b).unwrap();

        assert!((layout_a.width_cat - 1.0).abs() < 1e-9);
        assert!((layout_b.width_cat - 1.0).abs() < 1e-9);
        assert!((layout_a.offset_cat - layout_b.offset_cat).abs() < 1e-9);
    }

    #[test]
    fn bar_layout_keeps_px_width_as_group_preference() {
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
        let x_field = FieldId::new(1);
        let y_a_field = FieldId::new(2);
        let y_b_field = FieldId::new(3);
        let a = SeriesId::new(1);
        let b = SeriesId::new(2);

        let bar_layout = BarLayoutSpec {
            bar_width: Some(crate::spec::BarWidthSpec::Px(12.0)),
            bar_gap: Some(0.3),
            bar_category_gap: Some(0.2),
        };

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
                        categories: vec!["A".into()],
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
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout,
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
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout,
                    area_baseline: None,
                    lod: None,
                },
            ],
        };

        let model = ChartModel::from_spec(spec).unwrap();

        let mut datasets = DatasetStore::default();
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![0.0]));
        table.push_column(Column::F64(vec![1.0]));
        table.push_column(Column::F64(vec![2.0]));
        datasets.insert(dataset_id, table);

        let mut stage = BarLayoutStage::default();
        stage.begin_frame();
        stage.request_for_visible_bars(&model);
        stage.prepare_requests();
        assert!(stage.step(&model, &datasets, &mut WorkBudget::new(1_000_000, 0, 64)));

        let layout_a = stage.layout_for_series(&model, a).unwrap();
        let layout_b = stage.layout_for_series(&model, b).unwrap();

        assert_eq!(
            layout_a.bar_width,
            Some(crate::spec::BarWidthSpec::Px(12.0))
        );
        assert_eq!(
            layout_b.bar_width,
            Some(crate::spec::BarWidthSpec::Px(12.0))
        );
    }
}
