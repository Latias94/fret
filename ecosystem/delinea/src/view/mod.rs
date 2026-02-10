use crate::data::{DataTable, DatasetStore};
use crate::engine::ChartState;
use crate::engine::model::ChartModel;
use crate::engine::window_policy::AxisFilter1D;
use crate::ids::{AxisId, DatasetId, Revision, SeriesId};
use crate::spec::{FilterMode, SeriesKind};
use crate::transform::{RowRange, RowSelection, SeriesXPolicy, data_zoom_x_node, data_zoom_y_node};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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
    pub selection: RowSelection,
    pub x_policy: SeriesXPolicy,
    pub x_filter_mode: FilterMode,
    pub y_filter_mode: FilterMode,
    pub y_filter: AxisFilter1D,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SeriesEmptyMask {
    pub x_active: bool,
    pub x_filter: AxisFilter1D,
    pub y_active: bool,
    pub y_filter: AxisFilter1D,
    pub y_is_interval: bool,
}

impl SeriesEmptyMask {
    pub fn allows_axis_x_value(self, axis_x: f64) -> bool {
        if !self.x_active {
            return true;
        }
        axis_x.is_finite() && self.x_filter.contains(axis_x)
    }

    pub fn allows_raw_index(
        self,
        raw_index: usize,
        x: &[f64],
        y0: &[f64],
        y1: Option<&[f64]>,
    ) -> bool {
        if self.x_active {
            let x_raw = x.get(raw_index).copied().unwrap_or(f64::NAN);
            if !x_raw.is_finite() || !self.x_filter.contains(x_raw) {
                return false;
            }
        }

        if self.y_active {
            if self.y_is_interval {
                let Some(y1) = y1 else {
                    return false;
                };
                let y0_raw = y0.get(raw_index).copied().unwrap_or(f64::NAN);
                let y1_raw = y1.get(raw_index).copied().unwrap_or(f64::NAN);
                if !y0_raw.is_finite()
                    || !y1_raw.is_finite()
                    || !self.y_filter.intersects_interval(y0_raw, y1_raw)
                {
                    return false;
                }
            } else {
                let y_raw = y0.get(raw_index).copied().unwrap_or(f64::NAN);
                if !y_raw.is_finite() || !self.y_filter.contains(y_raw) {
                    return false;
                }
            }
        }

        true
    }
}

impl SeriesView {
    pub fn empty_mask(&self, kind: SeriesKind, stacked: bool) -> SeriesEmptyMask {
        let x_active = self.x_filter_mode == FilterMode::Empty
            && (self.x_policy.filter.min.is_some() || self.x_policy.filter.max.is_some());
        let y_active = self.y_filter_mode == FilterMode::Empty
            && !stacked
            && (self.y_filter.min.is_some() || self.y_filter.max.is_some())
            && matches!(
                kind,
                SeriesKind::Line
                    | SeriesKind::Area
                    | SeriesKind::Band
                    | SeriesKind::Scatter
                    | SeriesKind::Bar
            );

        SeriesEmptyMask {
            x_active,
            x_filter: self.x_policy.filter,
            y_active,
            y_filter: self.y_filter,
            y_is_interval: kind == SeriesKind::Band,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ViewState {
    pub revision: Revision,
    pub datasets: Vec<DatasetView>,
    pub series: Vec<SeriesView>,
    last_model_rev: Revision,
    last_data_sig: u64,
    last_state_sig: u64,
}

impl ViewState {
    pub fn sync_inputs(
        &mut self,
        model: &ChartModel,
        datasets: &DatasetStore,
        state: &ChartState,
    ) -> bool {
        let model_rev = model.revs.spec;
        let data_sig = dataset_store_signature(model, datasets);
        let state_sig = view_state_signature(model, state);

        if model_rev != self.last_model_rev
            || data_sig != self.last_data_sig
            || state_sig != self.last_state_sig
        {
            self.revision.bump();
            self.last_model_rev = model_rev;
            self.last_data_sig = data_sig;
            self.last_state_sig = state_sig;
            return true;
        }

        false
    }

    pub fn rebuild(&mut self, model: &ChartModel, datasets: &DatasetStore, state: &ChartState) {
        self.datasets.clear();
        self.series.clear();
        for (id, _dataset_model) in &model.datasets {
            let table = datasets.dataset(model.root_dataset_id(*id));
            let Some(table) = table else { continue };
            let mut row_range = state
                .dataset_row_ranges
                .get(id)
                .copied()
                .unwrap_or(RowRange {
                    start: 0,
                    end: table.row_count(),
                });
            row_range.clamp_to_len(table.row_count());
            self.datasets.push(DatasetView {
                dataset: *id,
                revision: self.revision,
                data_revision: table.revision(),
                row_range,
            });
        }

        for series_id in &model.series_order {
            let Some(series) = model.series.get(series_id) else {
                continue;
            };
            let table = datasets.dataset(model.root_dataset_id(series.dataset));
            let Some(table) = table else { continue };
            let Some(dataset) = model.datasets.get(&series.dataset) else {
                continue;
            };
            let Some(x_col) = dataset.fields.get(&series.encode.x).copied() else {
                continue;
            };
            let Some(_x) = table.column_f64(x_col) else {
                continue;
            };

            let mut base_range = state
                .dataset_row_ranges
                .get(&series.dataset)
                .copied()
                .unwrap_or(RowRange {
                    start: 0,
                    end: table.row_count(),
                });
            base_range.clamp_to_len(table.row_count());

            let x_axis_range = model
                .axes
                .get(&series.x_axis)
                .map(|a| a.range)
                .unwrap_or_default();
            let x_node = data_zoom_x_node(model, state, series.x_axis, x_axis_range);
            let x_filter_mode = x_node.filter_mode();
            let mut x_policy = x_node.x_policy();
            if model
                .axes
                .get(&series.x_axis)
                .is_some_and(|a| matches!(a.scale, crate::scale::AxisScale::Category(_)))
            {
                x_policy.mapping_window = expand_category_bands_window(x_policy.mapping_window);
            }
            let selection = RowSelection::Range(base_range);

            let y_filter_mode = model
                .data_zoom_y_by_axis
                .get(&series.y_axis)
                .and_then(|id| model.data_zoom_y.get(id))
                .map(|z| z.filter_mode)
                .unwrap_or(FilterMode::None);

            let y_axis_range = model
                .axes
                .get(&series.y_axis)
                .map(|a| a.range)
                .unwrap_or_default();
            let y_node = data_zoom_y_node(model, state, series.y_axis, y_axis_range);
            let y_filter = y_node.y_filter();

            self.series.push(SeriesView {
                series: *series_id,
                dataset: series.dataset,
                x_axis: series.x_axis,
                revision: self.revision,
                data_revision: table.revision(),
                selection,
                x_policy,
                x_filter_mode,
                y_filter_mode,
                y_filter,
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

fn expand_category_bands_window(
    window: Option<crate::engine::window::DataWindowX>,
) -> Option<crate::engine::window::DataWindowX> {
    window.map(|mut w| {
        let min = w.min;
        if min.is_finite() && is_near_integer(min) {
            w.min -= 0.5;
        }
        let max = w.max;
        if max.is_finite() && is_near_integer(max) {
            w.max += 0.5;
        }
        w.clamp_non_degenerate();
        w
    })
}

fn is_near_integer(v: f64) -> bool {
    (v - v.round()).abs() < 1e-9
}

fn dataset_store_signature(model: &ChartModel, datasets: &DatasetStore) -> u64 {
    let mut hash = 1469598103934665603u64;
    hash = fnv1a_step(hash, model.datasets.len() as u64);
    for dataset_id in model.datasets.keys() {
        hash = fnv1a_step(hash, dataset_id.0);
        if let Some(table) = datasets.dataset(model.root_dataset_id(*dataset_id)) {
            hash = fnv1a_step(hash, table.revision().0);
            hash = fnv1a_step(hash, table.row_count() as u64);
            hash = fnv1a_step(hash, table.column_count() as u64);
        }
    }
    hash
}

fn fnv1a_step(hash: u64, value: u64) -> u64 {
    (hash ^ value).wrapping_mul(1099511628211u64)
}

fn filter_mode_tag(mode: crate::spec::FilterMode) -> u64 {
    match mode {
        crate::spec::FilterMode::Filter => 1,
        crate::spec::FilterMode::WeakFilter => 2,
        crate::spec::FilterMode::Empty => 3,
        crate::spec::FilterMode::None => 4,
    }
}

fn f64_tag(v: Option<f64>) -> u64 {
    match v {
        None => 0,
        Some(v) => v.to_bits(),
    }
}

fn view_state_signature(model: &ChartModel, state: &ChartState) -> u64 {
    // This signature intentionally ignores state that does not affect data/row participation:
    // hover position, brush selection, axis locks, visualMap state, etc.
    //
    // The intent is to avoid rebuilding the data view pipeline for purely visual/interaction state
    // changes (aligns with ECharts `dataZoomProcessor` which is part of the data pipeline).
    let mut hash = 1469598103934665603u64;

    hash = fnv1a_step(hash, model.data_zoom_x_by_axis.len() as u64);
    for axis in model.data_zoom_x_by_axis.keys() {
        hash = fnv1a_step(hash, axis.0);
        let st = state.data_zoom_x.get(axis).copied();
        hash = fnv1a_step(hash, st.is_some() as u64);
        if let Some(st) = st {
            hash = fnv1a_step(hash, filter_mode_tag(st.filter_mode));
            hash = fnv1a_step(hash, st.window.is_some() as u64);
            if let Some(w) = st.window {
                hash = fnv1a_step(hash, f64_tag(Some(w.min)));
                hash = fnv1a_step(hash, f64_tag(Some(w.max)));
            }
        }
    }

    hash = fnv1a_step(hash, model.data_zoom_y_by_axis.len() as u64);
    for axis in model.data_zoom_y_by_axis.keys() {
        hash = fnv1a_step(hash, axis.0);
        let w = state.data_window_y.get(axis).copied();
        hash = fnv1a_step(hash, w.is_some() as u64);
        if let Some(w) = w {
            hash = fnv1a_step(hash, f64_tag(Some(w.min)));
            hash = fnv1a_step(hash, f64_tag(Some(w.max)));
        }
    }

    // Percent windows are an input to the data participation pipeline even when the derived
    // value windows are computed later in the staged filter plan (order-sensitive).
    hash = fnv1a_step(hash, state.axis_percent_windows.len() as u64);
    for (axis, (a, b)) in &state.axis_percent_windows {
        hash = fnv1a_step(hash, axis.0);
        hash = fnv1a_step(hash, a.to_bits());
        hash = fnv1a_step(hash, b.to_bits());
    }

    hash = fnv1a_step(hash, model.datasets.len() as u64);
    for dataset in model.datasets.keys() {
        hash = fnv1a_step(hash, dataset.0);
        let r = state.dataset_row_ranges.get(dataset).copied();
        hash = fnv1a_step(hash, r.is_some() as u64);
        if let Some(r) = r {
            hash = fnv1a_step(hash, r.start as u64);
            hash = fnv1a_step(hash, r.end as u64);
        }
    }

    hash
}

pub fn table_row_range<'a>(
    table: &'a DataTable,
    view: Option<&DatasetView>,
) -> core::ops::Range<usize> {
    let mut range = RowRange {
        start: 0,
        end: table.row_count(),
    };
    if let Some(view) = view {
        range = view.row_range;
    }
    range.as_std_range(table.row_count())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Column, DataTable, DatasetStore};
    use crate::engine::ChartState;
    use crate::engine::model::ChartModel;
    use crate::engine::window::DataWindow;
    use crate::ids::{AxisId, ChartId, DataZoomId, DatasetId, FieldId, GridId, SeriesId};
    use crate::scale::AxisScale;
    use crate::spec::{
        AxisKind, AxisRange, AxisSpec, DatasetSpec, FieldSpec, FilterMode, GridSpec, SeriesEncode,
        SeriesKind, SeriesSpec,
    };

    fn build_model() -> ChartModel {
        let dataset_id = DatasetId::new(1);
        let grid_id = GridId::new(1);
        let x_axis = AxisId::new(1);
        let y_axis = AxisId::new(2);
        let x_field = FieldId::new(1);
        let y_field = FieldId::new(2);

        let spec = crate::spec::ChartSpec {
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
                        id: y_field,
                        column: 1,
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
                    scale: AxisScale::default(),
                    range: Some(AxisRange::Auto),
                },
                AxisSpec {
                    id: y_axis,
                    name: None,
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: None,
                    scale: AxisScale::default(),
                    range: Some(AxisRange::Auto),
                },
            ],
            data_zoom_x: vec![crate::spec::DataZoomXSpec {
                id: DataZoomId::new(1),
                axis: x_axis,
                filter_mode: FilterMode::Filter,
                min_value_span: None,
                max_value_span: None,
            }],
            data_zoom_y: vec![crate::spec::DataZoomYSpec {
                id: DataZoomId::new(2),
                axis: y_axis,
                filter_mode: FilterMode::Filter,
                min_value_span: None,
                max_value_span: None,
            }],
            tooltip: None,
            axis_pointer: None,
            visual_maps: vec![],
            series: vec![SeriesSpec {
                id: SeriesId::new(1),
                name: None,
                kind: SeriesKind::Scatter,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_field,
                    y2: None,
                },
                x_axis,
                y_axis,
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            }],
        };

        ChartModel::from_spec(spec).expect("valid chart spec")
    }

    fn build_datasets() -> DatasetStore {
        let mut table = DataTable::default();
        table.push_column(Column::F64(vec![0.0, 1.0, 2.0, 3.0]));
        table.push_column(Column::F64(vec![10.0, 20.0, 30.0, 40.0]));
        let mut store = DatasetStore::default();
        store.insert(DatasetId::new(1), table);
        store
    }

    #[test]
    fn view_sync_ignores_hover_and_brush_state() {
        let model = build_model();
        let datasets = build_datasets();
        let mut state = ChartState::default();
        let mut view = ViewState::default();

        assert!(view.sync_inputs(&model, &datasets, &state));
        assert!(!view.sync_inputs(&model, &datasets, &state));

        state.hover_px = Some(fret_core::Point::new(
            fret_core::Px(1.0),
            fret_core::Px(2.0),
        ));
        state.revision.bump();
        assert!(
            !view.sync_inputs(&model, &datasets, &state),
            "hover must not invalidate the data view pipeline"
        );

        state.brush_selection_2d = Some(crate::selection::BrushSelection2D {
            grid: None,
            x_axis: AxisId::new(1),
            y_axis: AxisId::new(2),
            x: DataWindow { min: 0.0, max: 1.0 },
            y: DataWindow { min: 0.0, max: 1.0 },
        });
        state.revision.bump();
        assert!(
            !view.sync_inputs(&model, &datasets, &state),
            "brush must not invalidate the data view pipeline"
        );
    }

    #[test]
    fn view_sync_rebuilds_on_datazoom_and_dataset_row_range_changes() {
        let model = build_model();
        let datasets = build_datasets();
        let mut state = ChartState::default();
        let mut view = ViewState::default();

        assert!(view.sync_inputs(&model, &datasets, &state));

        state.data_zoom_x.insert(
            AxisId::new(1),
            crate::engine::DataZoomXState {
                window: Some(DataWindow { min: 0.0, max: 2.0 }),
                filter_mode: FilterMode::Filter,
            },
        );
        state.revision.bump();
        assert!(view.sync_inputs(&model, &datasets, &state));

        state.dataset_row_ranges.insert(
            DatasetId::new(1),
            crate::transform::RowRange { start: 1, end: 3 },
        );
        state.revision.bump();
        assert!(view.sync_inputs(&model, &datasets, &state));
    }
}
